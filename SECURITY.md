# Security Policy

OpenTranscribe is pre-release software. Please do not use it for sensitive
recordings until the release-hardening work and platform capture audits are
complete.

Report suspected vulnerabilities privately to the repository owner. Do not
include real API keys, recordings, transcripts, or other personal information
in a public issue.

## Credential handling

- Never commit API keys or signing credentials.
- Provider secrets belong in the operating-system credential vault.
- Pull-request workflows must not receive provider or signing secrets.
- Release signing credentials belong only in a protected GitHub environment.

If an OpenAI API key may have been exposed, remove it in the OpenAI dashboard,
create a replacement, and review account usage.
