use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

pub const APP_SETTINGS_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    RecordOnly,
    LocalAfterRecording,
    OpenAiLive,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum DictationProvider {
    #[default]
    Local,
    OpenAi,
}

impl<'de> Deserialize<'de> for RecordingMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "record_only" => Ok(Self::RecordOnly),
            "local_after_recording" | "local_live" => Ok(Self::LocalAfterRecording),
            "open_ai_live" => Ok(Self::OpenAiLive),
            value => Err(D::Error::unknown_variant(
                value,
                &["record_only", "local_after_recording", "open_ai_live"],
            )),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum OpenAiTranscriptionModel {
    #[default]
    GptTranscribe,
    #[serde(rename = "gpt_4o_transcribe")]
    #[ts(rename = "gpt_4o_transcribe")]
    Gpt4oTranscribe,
    #[serde(rename = "gpt_4o_transcribe_diarize")]
    #[ts(rename = "gpt_4o_transcribe_diarize")]
    Gpt4oTranscribeDiarize,
    #[serde(rename = "gpt_4o_mini_transcribe")]
    #[ts(rename = "gpt_4o_mini_transcribe")]
    Gpt4oMiniTranscribe,
}

impl OpenAiTranscriptionModel {
    pub fn model_id(&self) -> &'static str {
        match self {
            Self::GptTranscribe => "gpt-transcribe",
            Self::Gpt4oTranscribe => "gpt-4o-transcribe",
            Self::Gpt4oTranscribeDiarize => "gpt-4o-transcribe-diarize",
            Self::Gpt4oMiniTranscribe => "gpt-4o-mini-transcribe",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordingProjectSelection {
    #[default]
    Automatic,
    Inbox,
    Project {
        project_id: String,
    },
}

const DEFAULT_GLOBAL_SHORTCUT: &str = "CommandOrControl+Shift+R";
const DEFAULT_DICTATION_SHORTCUT: &str = "Alt+Space";

#[derive(Clone, Debug, PartialEq, TS)]
#[ts(export)]
pub struct GlobalShortcut(pub String);

impl Default for GlobalShortcut {
    fn default() -> Self {
        Self(DEFAULT_GLOBAL_SHORTCUT.to_owned())
    }
}

impl<'de> Deserialize<'de> for GlobalShortcut {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        Ok(Self(match value.as_str() {
            "command_or_control_shift_r" => DEFAULT_GLOBAL_SHORTCUT.to_owned(),
            "command_or_control_shift_space" => "CommandOrControl+Shift+Space".to_owned(),
            "alt_shift_r" => "Alt+Shift+R".to_owned(),
            _ => value,
        }))
    }
}

impl Serialize for GlobalShortcut {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Appearance {
    pub theme: ThemePreference,
    pub reduced_motion: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct AppSettings {
    #[serde(default = "default_settings_schema_version")]
    pub schema_version: u32,
    #[ts(type = "number")]
    pub revision: u64,
    #[serde(default)]
    pub setup_completed: bool,
    pub recording_mode: RecordingMode,
    #[serde(default)]
    pub openai_transcription_model: OpenAiTranscriptionModel,
    #[serde(default)]
    pub microphone_device_id: Option<String>,
    #[serde(default = "default_capture_system_audio")]
    pub capture_system_audio: bool,
    #[serde(default)]
    pub microphone_echo_cancellation: bool,
    #[serde(default)]
    pub local_models_directory: Option<String>,
    #[serde(default)]
    pub recording_project_selection: RecordingProjectSelection,
    #[serde(default)]
    pub global_shortcut_enabled: bool,
    #[serde(default)]
    pub global_shortcut: GlobalShortcut,
    #[serde(default = "default_dictation_shortcut_enabled")]
    pub dictation_shortcut_enabled: bool,
    #[serde(default = "default_dictation_shortcut")]
    pub dictation_shortcut: GlobalShortcut,
    #[serde(default)]
    pub dictation_provider: DictationProvider,
    #[serde(default)]
    pub dictation_local_model_id: Option<String>,
    #[serde(default)]
    pub dictation_openai_model: OpenAiTranscriptionModel,
    #[serde(default = "default_dictation_auto_paste")]
    pub dictation_auto_paste: bool,
    #[serde(default)]
    pub dictation_cleanup_enabled: bool,
    pub appearance: Appearance,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: APP_SETTINGS_SCHEMA_VERSION,
            revision: 1,
            setup_completed: false,
            recording_mode: RecordingMode::RecordOnly,
            openai_transcription_model: OpenAiTranscriptionModel::default(),
            microphone_device_id: None,
            capture_system_audio: default_capture_system_audio(),
            microphone_echo_cancellation: false,
            local_models_directory: None,
            recording_project_selection: RecordingProjectSelection::default(),
            global_shortcut_enabled: false,
            global_shortcut: GlobalShortcut::default(),
            dictation_shortcut_enabled: default_dictation_shortcut_enabled(),
            dictation_shortcut: default_dictation_shortcut(),
            dictation_provider: DictationProvider::default(),
            dictation_local_model_id: None,
            dictation_openai_model: OpenAiTranscriptionModel::default(),
            dictation_auto_paste: true,
            dictation_cleanup_enabled: false,
            appearance: Appearance {
                theme: ThemePreference::System,
                reduced_motion: false,
            },
        }
    }
}

fn default_dictation_shortcut() -> GlobalShortcut {
    GlobalShortcut(DEFAULT_DICTATION_SHORTCUT.to_owned())
}

fn default_capture_system_audio() -> bool {
    true
}

fn default_dictation_shortcut_enabled() -> bool {
    false
}

fn default_dictation_auto_paste() -> bool {
    true
}

fn default_settings_schema_version() -> u32 {
    APP_SETTINGS_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::{
        APP_SETTINGS_SCHEMA_VERSION, AppSettings, DictationProvider, GlobalShortcut,
        OpenAiTranscriptionModel, RecordingMode, RecordingProjectSelection,
    };

    #[test]
    fn migrates_the_legacy_local_live_recording_mode_name() {
        let mode: RecordingMode =
            serde_json::from_str(r#""local_live""#).expect("legacy mode should deserialize");

        assert_eq!(mode, RecordingMode::LocalAfterRecording);
        assert_eq!(
            serde_json::to_string(&mode).expect("mode should serialize"),
            r#""local_after_recording""#
        );
    }

    #[test]
    fn reads_existing_settings_with_system_audio_defaulted_on() {
        let settings: AppSettings = serde_json::from_str(
            r#"{
                "revision": 1,
                "setup_completed": true,
                "recording_mode": "record_only",
                "microphone_device_id": null,
                "global_shortcut_enabled": false,
                "global_shortcut": "CommandOrControl+Shift+R",
                "appearance": { "theme": "system", "reduced_motion": false }
            }"#,
        )
        .expect("existing settings should deserialize");

        assert_eq!(
            settings.recording_project_selection,
            RecordingProjectSelection::Automatic
        );
        assert_eq!(
            settings.openai_transcription_model,
            OpenAiTranscriptionModel::GptTranscribe
        );
        assert_eq!(settings.local_models_directory, None);
        assert!(settings.capture_system_audio);
        assert!(!settings.microphone_echo_cancellation);
        assert_eq!(settings.global_shortcut.0, "CommandOrControl+Shift+R");
        assert!(!settings.dictation_shortcut_enabled);
        assert_eq!(settings.dictation_shortcut.0, "Alt+Space");
        assert_eq!(settings.dictation_provider, DictationProvider::Local);
        assert_eq!(settings.dictation_local_model_id, None);
        assert_eq!(
            settings.dictation_openai_model,
            OpenAiTranscriptionModel::GptTranscribe
        );
        assert!(settings.dictation_auto_paste);
        assert!(!settings.dictation_cleanup_enabled);
        assert_eq!(settings.schema_version, APP_SETTINGS_SCHEMA_VERSION);
    }

    #[test]
    fn preserves_an_explicit_system_audio_preference() {
        let settings: AppSettings = serde_json::from_str(
            r#"{
                "revision": 1,
                "recording_mode": "record_only",
                "capture_system_audio": false,
                "appearance": { "theme": "system", "reduced_motion": false }
            }"#,
        )
        .expect("settings should deserialize");

        assert!(!settings.capture_system_audio);
        assert!(!settings.microphone_echo_cancellation);
    }

    #[test]
    fn maps_openai_file_models_to_api_ids() {
        assert_eq!(
            OpenAiTranscriptionModel::GptTranscribe.model_id(),
            "gpt-transcribe"
        );
        assert_eq!(
            OpenAiTranscriptionModel::Gpt4oTranscribe.model_id(),
            "gpt-4o-transcribe"
        );
        assert_eq!(
            OpenAiTranscriptionModel::Gpt4oTranscribeDiarize.model_id(),
            "gpt-4o-transcribe-diarize"
        );
        assert_eq!(
            OpenAiTranscriptionModel::Gpt4oMiniTranscribe.model_id(),
            "gpt-4o-mini-transcribe"
        );
    }

    #[test]
    fn preserves_a_custom_dictation_shortcut_through_settings_storage() {
        let settings = AppSettings {
            dictation_shortcut: GlobalShortcut("Alt+Shift+D".to_owned()),
            ..AppSettings::default()
        };
        let stored_settings = serde_json::to_vec(&settings).expect("settings should serialize");
        let loaded_settings: AppSettings =
            serde_json::from_slice(&stored_settings).expect("settings should deserialize");

        assert_eq!(loaded_settings.dictation_shortcut.0, "Alt+Shift+D");
    }
}
