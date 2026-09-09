use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, ModelDescriptor, PROTOCOL_VERSION, read_frame, write_frame,
};
use std::io::BufReader;
use std::process::{Command as ProcessCommand, Stdio};

#[test]
#[ignore = "requires OPENTRANSCRIBE_S1_MODEL pointing to the catalog GGUF"]
fn normalizes_multiple_utterances_in_one_loaded_process() {
    let model_path = std::env::var("OPENTRANSCRIBE_S1_MODEL").expect("model path");
    let mut child = ProcessCommand::new(env!("CARGO_BIN_EXE_opentranscribe-text-normalizer"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("start normalizer");
    let mut input = child.stdin.take().expect("stdin");
    let mut output = BufReader::new(child.stdout.take().expect("stdout"));
    let mut request = |command| {
        write_frame(
            &mut input,
            &Envelope {
                protocol_version: PROTOCOL_VERSION,
                request_id: "test".to_owned(),
                body: command,
            },
        )
        .expect("send command");
        read_frame::<Envelope<Event>>(&mut output)
            .expect("read event")
            .body
    };
    assert!(matches!(request(Command::Hello), Event::Ready { .. }));
    assert!(matches!(
        request(Command::LoadModel(ModelDescriptor {
            model_id: "s1-mini-q4_k_m".to_owned(),
            path: model_path,
            use_gpu: cfg!(target_os = "macos")
        })),
        Event::Status { .. }
    ));
    let started = std::time::Instant::now();
    let result = request(Command::NormalizeText {
        job_id: "first".to_owned(),
        text: "um send the draft on monday no tuesday".to_owned(),
    });
    let Event::NormalizedText { text, .. } = result else {
        panic!("normalization failed: {result:?}");
    };
    assert!(text.to_lowercase().contains("tuesday"), "{text}");
    assert!(!text.to_lowercase().contains("monday"), "{text}");
    eprintln!("First cleanup inference: {:?}", started.elapsed());
    let result = request(Command::NormalizeText {
        job_id: "second".to_owned(),
        text: "um".to_owned(),
    });
    assert!(matches!(result, Event::NormalizedText { text, .. } if text.is_empty()));
    write_frame(
        &mut input,
        &Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id: "exit".to_owned(),
            body: Command::Shutdown,
        },
    )
    .expect("shutdown");
    assert!(child.wait().expect("wait").success());
}
