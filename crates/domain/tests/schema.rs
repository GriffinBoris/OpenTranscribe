use opentranscribe_domain::{
    APP_SETTINGS_SCHEMA_VERSION, AppSettings, Project, SCHEMA_VERSION, Session, SessionSource,
};

#[test]
fn project_round_trips_as_readable_json() {
    let project = Project::new("Product".to_owned());
    let json = serde_json::to_string_pretty(&project).expect("project should serialize");
    let decoded: Project = serde_json::from_str(&json).expect("project should deserialize");

    assert_eq!(decoded, project);
    assert_eq!(decoded.schema_version, SCHEMA_VERSION);
    assert!(json.contains("\"glossary\": []"));
}

#[test]
fn session_starts_as_a_draft() {
    let session = Session::new(
        "Untitled recording".to_owned(),
        None,
        SessionSource::Recording,
    );

    assert_eq!(session.schema_version, SCHEMA_VERSION);
    assert_eq!(session.revision, 1);
    assert_eq!(session.duration_ms, 0);
}

#[test]
fn settings_from_before_device_selection_use_safe_capture_defaults() {
    let json = r#"{
        "revision": 1,
        "recording_mode": "record_only",
        "appearance": {"theme": "system", "reduced_motion": false},
        "cloud_quality": "quality",
        "cloud_final_pass": true,
        "default_openai_profile_id": null,
        "launch_at_login": false,
        "notifications_enabled": true
    }"#;
    let settings: AppSettings =
        serde_json::from_str(json).expect("legacy settings should deserialize");

    assert_eq!(settings.microphone_device_id, None);
    assert!(!settings.capture_system_audio);
    assert!(!settings.setup_completed);
    assert!(!settings.global_shortcut_enabled);
    assert_eq!(settings.global_shortcut.0, "CommandOrControl+Shift+R");
    assert_eq!(settings.schema_version, APP_SETTINGS_SCHEMA_VERSION);
}
