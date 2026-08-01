# Credential storage

OpenTranscribe stores the OpenAI API key in the operating system's credential
vault. The key is never written to application settings, the recording library,
logs, diagnostics, or frontend persistence.

| Platform | Credential backend                         |
| -------- | ------------------------------------------ |
| macOS    | Login Keychain generic password            |
| Windows  | Windows Credential Manager                 |
| Linux    | Secret Service through the desktop keyring |

The native process reads the saved key at most once per launch and reuses the
in-memory value for connection tests, live transcription, and finalized-file
transcription. Saving or removing the key updates that process cache
immediately.

Linux requires an unlocked Secret Service-compatible keyring, such as GNOME
Keyring or KDE Wallet. The app reports an unavailable or locked vault instead
of falling back to a plaintext file.

During macOS development, rebuilding an ad-hoc-signed application can make
Keychain treat the executable as a different requester. A stable Developer ID
signature is the durable fix for repeated access prompts; do not weaken the
Keychain item's access controls or store the key outside Keychain to avoid the
prompt. See [macOS signing](macos-signing.md).

The physical save, relaunch, use, and removal checks for every platform are in
[platform validation](platform-validation.md).
