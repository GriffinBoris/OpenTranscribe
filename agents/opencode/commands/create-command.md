---
description: Create a new OpenCode command file
agent: build
---

Reference https://opencode.ai/docs/commands/ and the existing commands in @agents/opencode/commands/full-review.md and @agents/opencode/commands/review-git-diff.md to match formatting and conventions.
Create a new command file in `agents/opencode/commands/` with the name `$1`. If `$1` is missing, ask for the command name.
Use `$2` as the description (or ask if missing). If `$3` is provided, use it as the agent name. If it is missing, omit the agent field.
Use the remaining arguments ($ARGUMENTS beyond $3) as the command template body. If the body is missing, ask for the exact prompt text to include.
If you learn new patterns or conventions while creating commands, update `agents/opencode/commands/create-command.md` with those learnings (living document ideology).
Return the path of the new command file and a brief usage example (e.g., `/command-name ...`).