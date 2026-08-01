# macOS signing

OpenTranscribe uses the bundle identifier
`com.griffinboris.opentranscribe`. Keep that identifier and the signing
identity stable so macOS can associate microphone and Screen & System Audio
Recording permission with the same application across builds.

macOS is the only supported platform where signing changes runtime behavior.
It binds Screen & System Audio Recording and microphone grants, along with
Keychain item access, to the signing identity. An unsigned or ad-hoc signed
build produces a different identity on every release, so each update silently
drops those permissions and leaves a stale entry the user must remove by hand
before granting again. Windows and Linux have no equivalent coupling; their
installers stay unsigned.

## Entitlements

`apps/desktop/src-tauri/Entitlements.plist` carries
`com.apple.security.device.audio-input` and is referenced from
`bundle.macOS.entitlements` in `apps/desktop/src-tauri/tauri.conf.json`.

Notarization requires the Hardened Runtime, and the Hardened Runtime denies
microphone access without that entitlement. Unsigned development builds
capture audio without it, so a missing entitlement never fails a build. It
surfaces only as a notarized release that records silence. `task
build:signed:macos` and the release workflow both assert the entitlement
survived signing.

Screen & System Audio Recording has no matching entitlement. That permission
is a TCC grant resolved at runtime against the signing identity.

## Local signed build

A distributable build requires an active Apple Developer Program membership and
a `Developer ID Application` certificate with its private key in the login
keychain.

List the identities available to `codesign`:

```bash
security find-identity -v -p codesigning
```

Then build with the exact identity name:

```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
task build:signed:macos
```

The task performs a signing preflight, builds the app and DMG, verifies the
bundle recursively, rejects an ad-hoc signature, and rejects a bundle whose
capture entitlements did not survive signing.

Run and permission-test the packaged application at:

```text
target/release/bundle/macos/OpenTranscribe.app
```

Do not alternate between the development binary and packaged application while
testing permissions. macOS treats identities and application locations as part
of the privacy decision, so use the signed packaged app for repeatable capture
validation.

## Certificate setup

Create the Developer ID Application certificate through the Apple Developer
Certificates, Identifiers & Profiles portal. Install the downloaded certificate
in the login keychain that contains the certificate request's private key.
`security find-identity -v -p codesigning` must list it as valid before running
the signed build.

The certificate and private key are credentials. Never commit an exported
`.p12` file, its password, an App Store Connect API private key, or notarization
passwords.

## Release environment

The protected GitHub `release` environment is the boundary for signing and
notarization credentials. Configure these environment secrets before enabling
signed public releases:

- `APPLE_CERTIFICATE`: base64-encoded exported certificate and private key
- `APPLE_CERTIFICATE_PASSWORD`: password used for the exported certificate
- `APPLE_SIGNING_IDENTITY`: exact Developer ID Application identity
- `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID`, or App Store Connect API
  issuer, key ID, and private key credentials for notarization

`APPLE_PASSWORD` is an app-specific password generated at appleid.apple.com,
not the Apple ID account password.

The release workflow reads `APPLE_SIGNING_IDENTITY` to decide how to build.
While the secret is unset, macOS jobs pass `--no-sign` and the release is
described as an unsigned preview, so tagging keeps working before enrollment
completes. Once the secret is present, macOS jobs sign and notarize, and the
signed bundle is verified before upload. No workflow edit is needed to switch
between the two states.

## Verifying a release build

Confirm the signature, the notarization ticket, and the entitlements on a
downloaded build:

```bash
codesign --verify --deep --strict --verbose=2 /Applications/OpenTranscribe.app
codesign -d --entitlements - --xml /Applications/OpenTranscribe.app
xcrun stapler validate /Applications/OpenTranscribe.app
spctl --assess --type execute --verbose /Applications/OpenTranscribe.app
```

`spctl` reporting `accepted` and `source=Notarized Developer ID` is the state
that gives a first launch without a Gatekeeper warning.

## Troubleshooting

**Local transcription crashes only in a signed build.** The bundled sidecar
links whisper.cpp with the Metal backend. Some ggml builds allocate executable
memory in a way the Hardened Runtime rejects. Confirm the crash is a code
signing failure with `log show --predicate 'sender == "kernel"' --last 5m`, and
if so add `com.apple.security.cs.allow-jit` to the entitlements. Add it only
after confirming the failure; it weakens the Hardened Runtime.

**Microphone capture records silence after notarization.** The entitlement did
not reach the bundle. Verify with `codesign -d --entitlements - --xml` against
the packaged app rather than the development binary.

**Permissions reset after an update.** The signing identity changed between
releases. Confirm both builds report the same `TeamIdentifier` and `Authority`
under `codesign -dvv`.
