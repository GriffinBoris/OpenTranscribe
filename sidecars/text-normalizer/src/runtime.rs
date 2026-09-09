use std::num::NonZeroU32;
use std::path::Path;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

const SYSTEM_PROMPT: &str = "You are a text normalizer for speech-to-text transcripts. The input begins with a control line specifying the styling, structure, and context settings; clean the transcript to match those settings and output only the cleaned text.";

pub struct Normalizer {
    model: Option<LlamaModel>,
    backend: LlamaBackend,
}

impl Normalizer {
    pub fn new() -> Result<Self, String> {
        let mut backend = LlamaBackend::init().map_err(|error| error.to_string())?;
        backend.void_logs();
        Ok(Self {
            model: None,
            backend,
        })
    }

    pub fn load(&mut self, path: &Path, use_gpu: bool) -> Result<(), String> {
        let params = LlamaModelParams::default().with_n_gpu_layers(if use_gpu { 99 } else { 0 });
        self.model = Some(
            LlamaModel::load_from_file(&self.backend, path, &params)
                .map_err(|error| error.to_string())?,
        );
        Ok(())
    }

    pub fn unload(&mut self) {
        self.model = None;
    }

    pub fn normalize(&self, text: &str) -> Result<String, String> {
        let model = self
            .model
            .as_ref()
            .ok_or("Load S1-mini before cleaning text.")?;
        let input_tokens = model
            .str_to_token(text, AddBos::Never)
            .map_err(|error| error.to_string())?;
        if input_tokens.len() > 1_000 {
            return Err("S1-mini supports up to 1,000 tokens per dictation. Your original transcript is preserved.".to_owned());
        }
        let tokens = model
            .str_to_token(&prompt(text), AddBos::Never)
            .map_err(|error| error.to_string())?;
        let output_limit = input_tokens.len() * 13 / 10 + 32;
        let params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(4096))
            .with_n_batch(2048);
        let mut context = model
            .new_context(&self.backend, params)
            .map_err(|error| error.to_string())?;
        let mut batch = LlamaBatch::new(2048, 1);
        batch
            .add_sequence(&tokens, 0, false)
            .map_err(|error| error.to_string())?;
        context
            .decode(&mut batch)
            .map_err(|error| error.to_string())?;
        let mut sampler = LlamaSampler::greedy();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        for index in 0..output_limit {
            let token = sampler.sample(&context, -1);
            if model.is_eog_token(token) {
                return Ok(output.trim().to_owned());
            }
            output.push_str(
                &model
                    .token_to_piece(token, &mut decoder, false, None)
                    .map_err(|error| error.to_string())?,
            );
            batch.clear();
            batch
                .add(token, (tokens.len() + index) as i32, &[0], true)
                .map_err(|error| error.to_string())?;
            context
                .decode(&mut batch)
                .map_err(|error| error.to_string())?;
        }
        Err("S1-mini did not finish within its output limit. Your original transcript is preserved.".to_owned())
    }
}

fn prompt(text: &str) -> String {
    // S1-mini's trained template requires an empty think block, not reasoning-budget suppression.
    format!(
        "<|im_start|>system\n{SYSTEM_PROMPT}<|im_end|>\n<|im_start|>user\n[Styling: semi-formal] [Structure: prose] [Context: general]\n{text}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n"
    )
}

#[cfg(test)]
mod tests {
    use super::prompt;
    #[test]
    fn applies_the_trained_control_line_and_non_thinking_prefix() {
        let rendered = prompt("uh send it friday no thursday");
        assert!(rendered.contains("[Styling: semi-formal] [Structure: prose] [Context: general]\nuh send it friday no thursday<|im_end|>"));
        assert!(rendered.ends_with("<|im_start|>assistant\n<think>\n\n</think>\n\n"));
    }
}
