use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    RecordOnly,
    LocalLive,
    OpenAiLive,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum GlobalShortcutPreset {
    #[default]
    CommandOrControlShiftR,
    CommandOrControlShiftSpace,
    AltShiftR,
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
    pub revision: u64,
    #[serde(default)]
    pub setup_completed: bool,
    pub recording_mode: RecordingMode,
    #[serde(default)]
    pub microphone_device_id: Option<String>,
    #[serde(default)]
    pub capture_system_audio: bool,
    #[serde(default)]
    pub global_shortcut_enabled: bool,
    #[serde(default)]
    pub global_shortcut: GlobalShortcutPreset,
    pub appearance: Appearance,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            revision: 1,
            setup_completed: false,
            recording_mode: RecordingMode::RecordOnly,
            microphone_device_id: None,
            capture_system_audio: false,
            global_shortcut_enabled: false,
            global_shortcut: GlobalShortcutPreset::default(),
            appearance: Appearance {
                theme: ThemePreference::System,
                reduced_motion: false,
            },
        }
    }
}
