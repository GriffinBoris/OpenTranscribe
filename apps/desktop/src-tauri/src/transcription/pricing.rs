use serde_json::{Value, json};

pub const OPENAI_PRICING_VERIFIED_AT: &str = "2026-07-31";

const GPT_TRANSCRIBE_USD_PER_MINUTE: f64 = 0.0045;
const GPT_4O_TRANSCRIBE_USD_PER_MINUTE: f64 = 0.006;
const GPT_4O_MINI_TRANSCRIBE_USD_PER_MINUTE: f64 = 0.003;
const GPT_LIVE_TRANSCRIBE_USD_PER_MINUTE: f64 = 0.017;

pub fn estimate_openai_cost(
    model_id: &str,
    duration_ms: u64,
    live_stream_count: u8,
) -> Option<f64> {
    let file_rate = file_rate_per_minute(model_id)?;
    let live_rate = GPT_LIVE_TRANSCRIBE_USD_PER_MINUTE * f64::from(live_stream_count);

    Some(duration_ms as f64 / 60_000.0 * (file_rate + live_rate))
}

pub fn openai_usage(
    model_id: &str,
    duration_ms: u64,
    live_stream_count: u8,
    provider_usage: Vec<Value>,
) -> Value {
    json!({
        "audio_duration_ms": duration_ms,
        "provider_usage": provider_usage,
        "pricing": {
            "currency": "USD",
            "file_rate_per_minute": file_rate_per_minute(model_id),
            "live_rate_per_minute": (live_stream_count > 0)
                .then_some(GPT_LIVE_TRANSCRIBE_USD_PER_MINUTE),
            "live_stream_count": live_stream_count,
            "verified_at": OPENAI_PRICING_VERIFIED_AT,
        }
    })
}

fn file_rate_per_minute(model_id: &str) -> Option<f64> {
    match model_id {
        "gpt-transcribe" => Some(GPT_TRANSCRIBE_USD_PER_MINUTE),
        "gpt-4o-transcribe" => Some(GPT_4O_TRANSCRIBE_USD_PER_MINUTE),
        "gpt-4o-mini-transcribe" => Some(GPT_4O_MINI_TRANSCRIBE_USD_PER_MINUTE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{estimate_openai_cost, openai_usage};

    #[test]
    fn estimates_file_transcription_from_audio_duration() {
        assert_eq!(
            estimate_openai_cost("gpt-transcribe", 120_000, 0),
            Some(0.009)
        );
    }

    #[test]
    fn includes_both_live_and_saved_transcript_passes() {
        assert_eq!(
            estimate_openai_cost("gpt-4o-mini-transcribe", 60_000, 1),
            Some(0.02)
        );
    }

    #[test]
    fn counts_microphone_and_system_live_streams_separately() {
        let estimate = estimate_openai_cost("gpt-4o-mini-transcribe", 60_000, 2)
            .expect("published model pricing should produce an estimate");

        assert!((estimate - 0.037).abs() < f64::EPSILON * 2.0);
    }

    #[test]
    fn leaves_unpublished_model_pricing_unknown() {
        assert_eq!(
            estimate_openai_cost("gpt-4o-transcribe-diarize", 60_000, 0),
            None
        );
        assert_eq!(
            openai_usage("gpt-4o-transcribe-diarize", 60_000, 0, Vec::new())["pricing"]["file_rate_per_minute"],
            serde_json::Value::Null
        );
    }
}
