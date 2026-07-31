use std::path::PathBuf;
use std::time::Duration;

use reqwest::blocking::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

use super::openai_audio::prepare_audio;

const TRANSCRIPTION_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const SUPPORTED_MODELS: [&str; 3] = [
    "gpt-transcribe",
    "gpt-4o-transcribe",
    "gpt-4o-mini-transcribe",
];
const DIARIZATION_MODEL: &str = "gpt-4o-transcribe-diarize";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAiDiarizedSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub speaker: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DetectedLanguage {
    pub code: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenAiChunkTranscript {
    pub text: String,
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
    #[serde(default)]
    pub languages: Vec<DetectedLanguage>,
    #[serde(default)]
    pub segments: Vec<OpenAiDiarizedSegment>,
    #[serde(skip)]
    pub duration_ms: u64,
}

pub struct OpenAiFileTranscriber {
    client: reqwest::blocking::Client,
    endpoint: String,
}

impl OpenAiFileTranscriber {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .connect_timeout(Duration::from_secs(20))
                .timeout(Duration::from_secs(30 * 60))
                .build()
                .expect("static OpenAI client configuration must be valid"),
            endpoint: TRANSCRIPTION_URL.to_owned(),
        }
    }

    pub fn test_connection(&self, api_key: &str) -> AppResult<()> {
        let response = self
            .client
            .get("https://api.openai.com/v1/models/gpt-transcribe")
            .bearer_auth(api_key)
            .send()
            .map_err(|error| AppError::Provider(error.to_string()))?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(AppError::Provider(format!(
            "OpenAI returned HTTP {}",
            response.status()
        )))
    }

    pub fn transcribe(
        &self,
        api_key: &str,
        model: &str,
        files: &[PathBuf],
        language_hint: Option<&str>,
        mut on_progress: impl FnMut(usize, usize),
        should_cancel: impl Fn() -> bool,
    ) -> AppResult<Vec<OpenAiChunkTranscript>> {
        if !SUPPORTED_MODELS.contains(&model) && model != DIARIZATION_MODEL {
            return Err(AppError::Provider(format!(
                "unsupported OpenAI transcription model: {model}"
            )));
        }

        let prepared = prepare_audio(files)?;
        let mut transcripts = Vec::new();

        for (index, chunk) in prepared.chunks.iter().enumerate() {
            if should_cancel() {
                return Err(AppError::JobCanceled);
            }

            let mut form = Form::new()
                .text("model", model.to_owned())
                .file("file", &chunk.path)?;

            if model == DIARIZATION_MODEL {
                form = form
                    .text("response_format", "diarized_json")
                    .text("chunking_strategy", "auto");
            }

            if let Some(language_hint) = language_hint {
                form = form.text("language", language_hint.to_owned());
            }
            let response = self
                .client
                .post(&self.endpoint)
                .bearer_auth(api_key)
                .multipart(form)
                .send()
                .map_err(|error| AppError::Provider(error.to_string()))?;

            if !response.status().is_success() {
                return Err(AppError::Provider(format!(
                    "OpenAI returned HTTP {}",
                    response.status()
                )));
            }

            let mut transcript: OpenAiChunkTranscript = response
                .json()
                .map_err(|error| AppError::Provider(error.to_string()))?;
            transcript.duration_ms = chunk.duration_ms;
            transcripts.push(transcript);
            on_progress(index + 1, prepared.chunks.len());

            if should_cancel() {
                return Err(AppError::JobCanceled);
            }
        }

        Ok(transcripts)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::PathBuf;
    use std::sync::mpsc::{self, Receiver};
    use std::thread::{self, JoinHandle};
    use std::time::Duration;

    use hound::{SampleFormat, WavSpec, WavWriter};
    use tempfile::TempDir;

    use super::{DIARIZATION_MODEL, OpenAiFileTranscriber, SUPPORTED_MODELS};

    #[test]
    fn keeps_quality_and_economy_models_explicit() {
        assert!(SUPPORTED_MODELS.contains(&"gpt-transcribe"));
        assert!(SUPPORTED_MODELS.contains(&"gpt-4o-mini-transcribe"));
    }

    #[test]
    fn requests_and_parses_diarized_segments() {
        let audio = TestAudio::new();
        let response = r#"{
            "text":"Hello there. Welcome.",
            "segments":[
                {"start":0.1,"end":0.4,"text":"Hello there.","speaker":"A"},
                {"start":0.5,"end":0.9,"text":"Welcome.","speaker":"B"}
            ]
        }"#;
        let (endpoint, requests, server) = mock_server("200 OK", response);
        let transcriber = OpenAiFileTranscriber {
            client: reqwest::blocking::Client::new(),
            endpoint,
        };

        let transcripts = transcriber
            .transcribe(
                "test-key",
                DIARIZATION_MODEL,
                &[audio.path()],
                None,
                |_, _| {},
                || false,
            )
            .expect("diarized transcription should succeed");
        let request = requests
            .recv_timeout(Duration::from_secs(5))
            .expect("mock server should capture one request");
        server.join().expect("mock server should finish");
        let body = String::from_utf8_lossy(&request.body);

        assert!(body.contains("name=\"model\"\r\n\r\ngpt-4o-transcribe-diarize"));
        assert!(body.contains("name=\"response_format\"\r\n\r\ndiarized_json"));
        assert!(body.contains("name=\"chunking_strategy\"\r\n\r\nauto"));
        assert_eq!(transcripts[0].segments.len(), 2);
        assert_eq!(transcripts[0].segments[0].speaker, "A");
        assert_eq!(transcripts[0].segments[1].text, "Welcome.");
    }

    #[test]
    fn sends_the_openai_file_transcription_contract() {
        let audio = TestAudio::new();
        let (endpoint, requests, server) = mock_server(
            "200 OK",
            r#"{
                "text":"hola",
                "languages":[{"code":"es"}],
                "usage":{"type":"duration","seconds":1}
            }"#,
        );
        let transcriber = OpenAiFileTranscriber {
            client: reqwest::blocking::Client::new(),
            endpoint,
        };
        let mut progress = Vec::new();

        let transcripts = transcriber
            .transcribe(
                "test-key",
                "gpt-transcribe",
                &[audio.path()],
                Some("es"),
                |completed, total| progress.push((completed, total)),
                || false,
            )
            .expect("mocked transcription should succeed");
        let request = requests
            .recv_timeout(Duration::from_secs(5))
            .expect("mock server should capture one request");
        server.join().expect("mock server should finish");
        let headers = request.headers.to_ascii_lowercase();
        let body = String::from_utf8_lossy(&request.body);

        assert!(headers.starts_with("post /v1/audio/transcriptions http/1.1"));
        assert!(headers.contains("authorization: bearer test-key"));
        assert!(headers.contains("content-type: multipart/form-data"));
        assert!(body.contains("name=\"model\"\r\n\r\ngpt-transcribe"));
        assert!(body.contains("name=\"language\"\r\n\r\nes"));
        assert!(body.contains("name=\"file\"; filename=\"recording.wav\""));
        assert_eq!(progress, vec![(1, 1)]);
        assert_eq!(transcripts.len(), 1);
        assert_eq!(transcripts[0].text, "hola");
        assert_eq!(transcripts[0].languages[0].code, "es");
        assert_eq!(transcripts[0].usage.as_ref().unwrap()["seconds"], 1);
        assert_eq!(transcripts[0].duration_ms, 1_000);
    }

    #[test]
    fn surfaces_openai_http_failures_without_fake_progress() {
        let audio = TestAudio::new();
        let (endpoint, requests, server) =
            mock_server("429 Too Many Requests", r#"{"error":"rate limited"}"#);
        let transcriber = OpenAiFileTranscriber {
            client: reqwest::blocking::Client::new(),
            endpoint,
        };
        let mut progress = Vec::new();

        let error = transcriber
            .transcribe(
                "test-key",
                "gpt-transcribe",
                &[audio.path()],
                None,
                |completed, total| progress.push((completed, total)),
                || false,
            )
            .expect_err("provider error should fail the job");
        requests
            .recv_timeout(Duration::from_secs(5))
            .expect("mock server should capture one request");
        server.join().expect("mock server should finish");

        assert!(error.to_string().contains("HTTP 429"));
        assert!(progress.is_empty());
    }

    struct TestAudio {
        directory: TempDir,
    }

    impl TestAudio {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("temporary directory should exist");
            let path = directory.path().join("recording.wav");
            let mut writer = WavWriter::create(
                path,
                WavSpec {
                    channels: 1,
                    sample_rate: 10,
                    bits_per_sample: 24,
                    sample_format: SampleFormat::Int,
                },
            )
            .expect("test audio should be created");

            for _ in 0..10 {
                writer
                    .write_sample(0_i32)
                    .expect("test sample should be written");
            }
            writer.finalize().expect("test audio should finalize");

            Self { directory }
        }

        fn path(&self) -> PathBuf {
            self.directory.path().join("recording.wav")
        }
    }

    struct CapturedRequest {
        headers: String,
        body: Vec<u8>,
    }

    fn mock_server(
        status: &'static str,
        response_body: &'static str,
    ) -> (String, Receiver<CapturedRequest>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("mock server should bind");
        let address = listener
            .local_addr()
            .expect("mock server should have an address");
        let (sender, receiver) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("mock server should accept");
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("mock server should set a timeout");
            let request = read_request(&mut stream);
            sender
                .send(request)
                .expect("captured request should be delivered");
            write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
                response_body.len()
            )
            .expect("mock response should be written");
        });

        (
            format!("http://{address}/v1/audio/transcriptions"),
            receiver,
            server,
        )
    }

    fn read_request(stream: &mut TcpStream) -> CapturedRequest {
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 4_096];

        loop {
            let count = stream
                .read(&mut buffer)
                .expect("request should be readable");
            assert!(count > 0, "request ended before its body was complete");
            bytes.extend_from_slice(&buffer[..count]);

            let Some(header_end) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
                continue;
            };
            let headers = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length").then(|| {
                        value
                            .trim()
                            .parse::<usize>()
                            .expect("content length is numeric")
                    })
                })
                .expect("multipart request should declare its content length");
            let body_start = header_end + 4;

            if bytes.len() >= body_start + content_length {
                return CapturedRequest {
                    headers,
                    body: bytes[body_start..body_start + content_length].to_vec(),
                };
            }
        }
    }
}
