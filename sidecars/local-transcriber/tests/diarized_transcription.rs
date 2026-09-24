use std::collections::HashSet;
use std::process::{Command as ProcessCommand, Stdio};

use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, FileTranscription, ModelDescriptor, PROTOCOL_VERSION, read_frame,
    write_frame,
};

#[test]
#[ignore = "requires downloaded Whisper/Nemotron models and a two-speaker WAV; see docs/local-diarization.md"]
fn transcribes_two_speakers_through_the_packaged_protocol() {
    let whisper = std::env::var("OPENTRANSCRIBE_TEST_WHISPER_MODEL").expect("Whisper model path");
    let nemotron =
        std::env::var("OPENTRANSCRIBE_TEST_NEMOTRON_MODEL").expect("Nemotron model path");
    let audio = std::env::var("OPENTRANSCRIBE_TEST_AUDIO").expect("two-speaker audio path");
    let mut child = ProcessCommand::new(env!("CARGO_BIN_EXE_opentranscribe-local-transcriber"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sidecar starts");
    let mut stdin = child.stdin.take().expect("stdin");
    let mut stdout = child.stdout.take().expect("stdout");
    for command in [
        Command::Hello,
        Command::LoadModel(ModelDescriptor {
            model_id: "whisper-validation".to_owned(),
            path: whisper,
            use_gpu: cfg!(target_os = "macos"),
        }),
        Command::TranscribeFile(FileTranscription {
            job_id: "diarization-test".to_owned(),
            path: audio,
            language_hint: Some("en".to_owned()),
            prompt: None,
            diarization_model_path: Some(nemotron),
        }),
        Command::Shutdown,
    ] {
        write_frame(
            &mut stdin,
            &Envelope {
                protocol_version: PROTOCOL_VERSION,
                request_id: "test".to_owned(),
                body: command,
            },
        )
        .expect("command encodes");
    }
    drop(stdin);
    let mut speakers = HashSet::new();
    let mut text = String::new();
    let mut last_start = 0;
    loop {
        let response: Envelope<Event> = read_frame(&mut stdout).expect("event decodes");
        assert_eq!(response.protocol_version, PROTOCOL_VERSION);
        match response.body {
            Event::Segment {
                start_ms,
                end_ms,
                text: words,
                speaker_label,
                ..
            } => {
                assert!(start_ms >= last_start && end_ms >= start_ms);
                last_start = start_ms;
                text.push_str(&words);
                if let Some(speaker) = speaker_label {
                    speakers.insert(speaker.clone());
                    eprintln!("{start_ms}..{end_ms} speaker {speaker}: {words}");
                }
            }
            Event::Error { message, .. } => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("inference failed: {message}");
            }
            Event::Completed { .. } => break,
            _ => {}
        }
    }
    assert!(child.wait().expect("sidecar exits").success());
    assert!(!text.trim().is_empty());
    assert!(
        speakers.len() >= 2,
        "expected multiple speakers, found {speakers:?}"
    );
}

#[test]
#[ignore = "requires Nemotron model and two-speaker WAV; no Whisper model required"]
fn diarizes_repeatedly_without_loading_a_speech_model() {
    let model_path =
        std::env::var("OPENTRANSCRIBE_TEST_NEMOTRON_MODEL").expect("Nemotron model path");
    let path = std::env::var("OPENTRANSCRIBE_TEST_AUDIO").expect("audio path");
    let mut child = ProcessCommand::new(env!("CARGO_BIN_EXE_opentranscribe-local-transcriber"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    for job_id in ["first", "rerun"] {
        write_frame(
            &mut stdin,
            &Envelope {
                protocol_version: PROTOCOL_VERSION,
                request_id: job_id.to_owned(),
                body: Command::DiarizeFile(opentranscribe_transcriber_protocol::FileDiarization {
                    job_id: job_id.to_owned(),
                    path: path.clone(),
                    model_path: model_path.clone(),
                }),
            },
        )
        .unwrap();
        let mut speakers = HashSet::new();
        let mut last_start = 0;
        loop {
            let response: Envelope<Event> = read_frame(&mut stdout).unwrap();
            match response.body {
                Event::SpeakerTurn {
                    job_id: response_job_id,
                    turn,
                } => {
                    assert_eq!(response_job_id, job_id);
                    assert!(turn.start_ms >= last_start && turn.end_ms > turn.start_ms);
                    last_start = turn.start_ms;
                    speakers.insert(turn.speaker_label);
                }
                Event::Completed {
                    job_id: response_job_id,
                } => {
                    assert_eq!(response_job_id, job_id);
                    break;
                }
                Event::Error { message, .. } => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("{message}");
                }
                Event::Segment { .. } => panic!("diarization must not emit replacement text"),
                _ => {}
            }
        }
        assert!(speakers.len() >= 2, "{speakers:?}");
    }
    write_frame(
        &mut stdin,
        &Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id: "shutdown".to_owned(),
            body: Command::Shutdown,
        },
    )
    .unwrap();
    drop(stdin);
    assert!(child.wait().unwrap().success());
}
