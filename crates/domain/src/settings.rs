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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordingProjectSelection {
    #[default]
    Automatic,
    Inbox,
    Project {
        project_id: String,
    },
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
    pub recording_project_selection: RecordingProjectSelection,
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
            recording_project_selection: RecordingProjectSelection::default(),
            global_shortcut_enabled: false,
            global_shortcut: GlobalShortcutPreset::default(),
            appearance: Appearance {
                theme: ThemePreference::System,
                reduced_motion: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppSettings, RecordingProjectSelection};

    #[test]
    fn reads_existing_settings_without_a_recording_project_selection() {
        let settings: AppSettings = serde_json::from_str(
            r#"{
                "revision": 1,
                "setup_completed": true,
                "recording_mode": "record_only",
                "microphone_device_id": null,
                "capture_system_audio": false,
                "global_shortcut_enabled": false,
                "global_shortcut": "command_or_control_shift_r",
                "appearance": { "theme": "system", "reduced_motion": false }
            }"#,
        )
        .expect("existing settings should deserialize");

        assert_eq!(
            settings.recording_project_selection,
            RecordingProjectSelection::Automatic
        );
    }
}
