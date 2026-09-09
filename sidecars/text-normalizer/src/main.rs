mod runtime;

use opentranscribe_transcriber_protocol::{
    Command, Envelope, Event, PROTOCOL_VERSION, read_frame, write_frame,
};
use runtime::Normalizer;
use std::io::BufReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(std::io::stdin().lock());
    let mut runtime = Normalizer::new()?;
    loop {
        let envelope: Envelope<Command> = read_frame(&mut reader)?;
        let result = if envelope.protocol_version != PROTOCOL_VERSION {
            Err("The desktop and normalizer protocol versions differ.".to_owned())
        } else {
            match envelope.body {
                Command::Hello => Ok(Event::Ready {
                    runtime_version: env!("CARGO_PKG_VERSION").to_owned(),
                }),
                Command::LoadModel(model) => runtime
                    .load(Path::new(&model.path), model.use_gpu)
                    .map(|()| Event::Status {
                        model_id: Some(model.model_id),
                        backlog_ms: 0,
                    }),
                Command::NormalizeText { job_id, text } => runtime
                    .normalize(&text)
                    .map(|text| Event::NormalizedText { job_id, text }),
                Command::UnloadModel => {
                    runtime.unload();
                    Ok(Event::Status {
                        model_id: None,
                        backlog_ms: 0,
                    })
                }
                Command::TranscribeFile(_) => {
                    Err("Use the speech transcriber for audio.".to_owned())
                }
                Command::Shutdown => break,
            }
        };
        write_frame(
            &mut std::io::stdout().lock(),
            &Envelope {
                protocol_version: PROTOCOL_VERSION,
                request_id: envelope.request_id,
                body: result.unwrap_or_else(|message| Event::Error {
                    code: "normalization_failed".to_owned(),
                    message,
                }),
            },
        )?;
    }
    Ok(())
}
