---
description: Review Copilot PR comments
agent: create a command for checking the copilot suggestions
---

Review Copilot PR comments for a provided PR URL.
Use `gh api repos/<org>/<repo>/pulls/<id>/comments` to fetch inline review comments.
For each suggestion, decide if changes are needed and why.
List required changes, optional improvements, and no-action items.
If additional context is needed, ask targeted questions.