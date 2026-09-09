pub struct ModelDefinition {
    pub id: &'static str,
    pub preset: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub filename: &'static str,
    pub byte_count: u64,
    pub sha256: &'static str,
    pub download_url: &'static str,
}

pub const MODELS: [ModelDefinition; 4] = [
    ModelDefinition {
        id: "whisper-small-q5_1",
        preset: "fast",
        label: "Fast",
        description: "Whisper Small · Multilingual · Q5",
        filename: "ggml-small-q5_1.bin",
        byte_count: 190_085_487,
        sha256: "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb",
        download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/87cd18b47b941d2f65d09981dad23bb7d0481c77/ggml-small-q5_1.bin?download=true",
    },
    ModelDefinition {
        id: "whisper-medium-q5_0",
        preset: "balanced",
        label: "Balanced",
        description: "Whisper Medium · Multilingual · Q5",
        filename: "ggml-medium-q5_0.bin",
        byte_count: 539_212_467,
        sha256: "19fea4b380c3a618ec4723c3eef2eb785ffba0d0538cf43f8f235e7b3b34220f",
        download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/87cd18b47b941d2f65d09981dad23bb7d0481c77/ggml-medium-q5_0.bin?download=true",
    },
    ModelDefinition {
        id: "whisper-large-v3-turbo-q5_0",
        preset: "best",
        label: "Best",
        description: "Whisper Large v3 Turbo · Multilingual · Q5",
        filename: "ggml-large-v3-turbo-q5_0.bin",
        byte_count: 574_041_195,
        sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/98aa99a0a9db05ae2342309f5096248665f7cba3/ggml-large-v3-turbo-q5_0.bin?download=true",
    },
    ModelDefinition {
        id: "s1-mini-q4_k_m",
        preset: "cleanup",
        label: "S1-mini by Superwhisper",
        description: "English dictation cleanup · Q4 · runs locally after transcription",
        filename: "s1-mini-q4_k_m.gguf",
        byte_count: 484_219_808,
        sha256: "3b41ebe2502cbd03e811d5d16b022f5ab551eda58d62597d152f89535003c634",
        download_url: "https://huggingface.co/superwhisper/s1-mini-GGUF/resolve/34add00a48a2e5d24e5a4ee5405a99620a3a240c/s1-mini-q4_k_m.gguf?download=true",
    },
];

pub fn find(model_id: &str) -> Option<&'static ModelDefinition> {
    MODELS.iter().find(|model| model.id == model_id)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{MODELS, find};

    #[test]
    fn catalog_ids_are_unique_and_findable() {
        let ids: HashSet<_> = MODELS.iter().map(|model| model.id).collect();

        assert_eq!(ids.len(), MODELS.len());
        assert!(MODELS.iter().all(|model| find(model.id).is_some()));
        assert!(MODELS.iter().all(|model| model.sha256.len() == 64));
        assert_eq!(MODELS[0].id, "whisper-small-q5_1");
        assert_eq!(MODELS[1].id, "whisper-medium-q5_0");
        assert_eq!(MODELS[2].id, "whisper-large-v3-turbo-q5_0");
    }
}
