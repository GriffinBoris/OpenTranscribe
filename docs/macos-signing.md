# macOS signing

OpenTranscribe uses the bundle identifier
`com.griffinboris.opentranscribe`. Keep that identifier and the signing
identity stable so macOS can associate microphone and Screen & System Audio
Recording permission with the same application across builds.

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
bundle recursively, and fails if Tauri produced an ad-hoc signature.

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

Unsigned CI previews remain intentionally separate from distributable release
artifacts until all credentials are present and a signed build has passed the
microphone, system-audio, and combined-capture smoke tests.
