use std::process::{Command as ProcessCommand, Stdio};

use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, PROTOCOL_VERSION, read_frame, write_frame,
};

#[test]
fn starts_and_completes_the_protocol_handshake() {
    let mut child = ProcessCommand::new(env!("CARGO_BIN_EXE_opentranscribe-local-transcriber"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("local transcriber should start");
    let mut stdin = child
        .stdin
        .take()
        .expect("sidecar stdin should be available");
    let mut stdout = child
        .stdout
        .take()
        .expect("sidecar stdout should be available");

    write_frame(
        &mut stdin,
        &Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id: "hello".to_owned(),
            body: Command::Hello,
        },
    )
    .expect("hello command should write");
    let response: Envelope<Event> =
        read_frame(&mut stdout).expect("ready event should be readable");

    assert_eq!(response.protocol_version, PROTOCOL_VERSION);
    assert_eq!(response.request_id, "hello");
    assert!(matches!(response.body, Event::Ready { .. }));

    write_frame(
        &mut stdin,
        &Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id: "shutdown".to_owned(),
            body: Command::Shutdown,
        },
    )
    .expect("shutdown command should write");
    drop(stdin);

    assert!(
        child
            .wait()
            .expect("local transcriber should exit")
            .success()
    );
}
