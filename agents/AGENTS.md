# Development Guidelines

Always read this file in full before starting work so updates stay consistent with the latest guidance.

## Living Document Philosophy

This file **evolves with the codebase**. Update it when you discover:
- Repeated patterns that improve clarity or maintainability
- Architectural decisions future work should respect
- Common mistakes worth preventing
- Better ways to explain existing guidelines
- UI/UX component patterns (for example, dropdown/search inputs) that should be reused instead of reinvented

This guidance lives in `agents/AGENTS.md`. Update this file and do not edit any AGENTS.md copy in the repo root.

When you encounter patterns, architectural decisions, or learnings that would help future work, **update this file immediately**. Don't wait to be asked -- if it's worth knowing, document it.

Be proactive about extracting preferences and micro-decisions (like "query templates live in code, not settings") into this doc. If a preference changes how similar work should be done later, capture it here right away.

During a session, if you discover durable context or decisions that future similar tasks should know, record them in this file.

Before finishing a task, re-check this file and decide whether:
- A new lesson, rule refinement, or exception should be recorded
- An existing rule needs clarification or update
- A pattern emerged that should be documented

When reflecting at the end of a task, explicitly check for:
- Verification coverage (tests/lint/typecheck) and any failures/timeouts
- UI/UX consistency with existing patterns (loading/empty/error states, dialogs, controls)
- Reuse of existing components/utilities before creating new ones
- Data handling consistency (query params, formatting, error handling)
- Migration/housekeeping needs (regen migrations, update settings/docs)

If no update is needed, state that you reviewed and decided no changes were warranted.

### How to Update

- Prefer small, incremental edits over large rewrites.
- Add context explaining why a rule exists, not just what it is.
- If a rule becomes obsolete, remove or revise it instead of layering exceptions.
- If two rules conflict, resolve the conflict explicitly.
- Avoid speculative rules. Document decisions made, not hypotheticals.

---

## Quick Reference

### Build, Lint, and Test Commands

**Backend (task runner, if available):**
```bash
task backend:tests:unit
task backend:tests:coverage
task backend:lint
task backend:test
task backend:db:migrate
task backend:db:makemigrations
task backend:db:makemigrations-app APP=app_name
task backend:db:bootstrap
task backend:server:run
task backend:shell:django
```

**Backend (direct Django/Python commands):**
```bash
# Run all tests
pytest
pytest path/to/test_file.py
pytest path/to/test_file.py::TestClass
pytest path/to/test_file.py::TestClass::test_method

# Linting and formatting
ruff check
ruff check --fix
ruff format

# Database
python manage.py migrate
python manage.py makemigrations
python manage.py makemigrations app_name

# Development server
python manage.py runserver
python manage.py shell
```

**Frontend (Vue/TypeScript, if present):**
```bash
cd frontend

# Development
npm run dev

# Type checking and linting
npm run type-check
npm run lint
npm run format

# Build
npm run build
npm run preview
```

**.NET (if present):**
```bash
dotnet build
dotnet test
dotnet format
```

**Docker (if present):**
```bash
task docker:start
task docker:start:detached
task docker:stop
task docker:bootstrap
task docker:logs SERVICE=web
```

### Tools

- If the repo exposes a context7 MCP server or web search, use them only when needed and within repo rules.
- If a repo has custom CLI tooling, prefer it over ad-hoc scripts.

## Guide Index

This file contains cross-stack principles that apply to **every** project. Stack-specific guidance lives in dedicated files:

| File | Scope |
|---|---|
| `agents/django.md` | Django, DRF, Python backend patterns, Celery, testing |
| `agents/frontend.md` | Vue 3, TypeScript, Pinia, frontend API clients, UI/UX |
| `agents/dotnet.md` | C#, .NET, Godot, ECS, game client/server, code generation |
| `agents/architecture-review.md` | Reusable audit rubric (50 quality principles) |
| `agents/preserved-guidance.md` | Archived project-specific content displaced from core guides |

Read the relevant stack-specific guide(s) in addition to this file before starting work.

---

## Project Structure (Generic)

- **Backend**: Django REST Framework app in `backend/` (if present)
  - Apps follow Django conventions with `models.py`, `views.py`, `serializers.py`, `urls.py`, `admin.py`, `tests/`
  - Base classes typically live in `core/` (example: `ProjectBaseModel`, `ProjectBaseAPIView`)
- **Frontend**: Vue 3 + TypeScript in `frontend/` (if present)
  - `src/components/`, `src/features/`, `src/stores/`, `src/composables/`, `src/types/`
  - `src/core/` for models/utilities, `src/styles/` for global styling
- **Shared libraries**: `shared/` or `shared_lib/` (if present) for reusable adapters/clients
- **Game stack**: Separate shared library, server, client, and codegen tooling (if present)

## GriffLab Household Pattern

- Shared data belongs to a `Household` and is scoped through `HouseholdMember`.
- Use `HouseholdMember.objects.get(user=request.user)` to derive the household boundary in views.
- Registration creates a new household and an owner membership for the user.
- Household-scoped third-party connections should be stored once per household on the backend, keep credentials encrypted at rest, and sync vendor data into local household-owned models for dashboard reads. Do not call vendor APIs directly from the SPA.

## GriffLab Auth Pattern

- The app authenticates with Google on the frontend and exchanges the Google credential for a Django session on the backend.
- Backend auth responses expose `google_client_id` alongside `csrf_token` so the SPA can bootstrap Google Sign-In before auth status resolves.
- First-time Google sign-in should reuse `HouseholdMember.get_or_create_for_user(user)` so household bootstrap stays consistent with other entry points.

## Product Direction

- GriffLab is a broad household operations hub, not a single-purpose vehicle or warranty app.
- Optimize shared surfaces like the home dashboard, navigation, and top-level labels for a growing mix of tools, automations, home systems, records, integrations, and experiments.
- Keep specific domains like vehicle coverage as modules within the larger platform rather than framing the product around one feature.
- Cat feeding plans can span multiple food sources. Model each source separately with its own label math and feeder schedule, then aggregate actual delivered daily totals at the cat level. Do not invent a single target grams/cups number across mixed foods unless the source mix itself is explicitly modeled.

## Expectations & Best Practices

### Agent Compliance

- Agents must read this file at the start of every task and treat it as required instructions.
- Agents must read the entire document top-to-bottom and confirm they reviewed all requirements before taking action.
- Before making changes, scan all sections for relevant requirements and follow them explicitly.
- If a request conflicts with this file, call out the conflict and follow the most restrictive rule.
- Agents must explicitly confirm they reviewed and complied with every section before making changes, and include that confirmation in the final response.

### Skills
Skill definitions live in `agents/skills/` or `.github/skills/` when present. If a repo has a skill index, follow it. In OpenCode, use `/agent <skill-name>` or `--agent`.

### Match Existing Patterns
- Follow the project's established coding style, naming conventions, and architectural patterns.
- Reference similar existing code before implementing new features.
- Use `task local:*` as the standard host development workflow. Keep domain-specific commands in the existing `agents:*` and `docker:*` namespaces rather than forcing greenfield template structure onto mature project operations.
- Treat `agents/` as the versioned source of truth. Generate local Codex, Claude, Copilot, Gemini, and OpenCode outputs with `task agents:generate`, and validate authored guidance with `task agents:check` instead of editing generated files.
- Frontend UI should use PrimeVue components and services (e.g., Toast) instead of custom UI components.
- Prefer GriffLab's shared `App*` and form-field wrappers over importing the underlying PrimeVue input directly. Migrate consumers alongside each new wrapper so the repository does not accumulate an unused parallel component library.
- Authenticated routes render inside `views/AppShell/AppShellView.vue`; the Google-only login route declares `meta.guestOnly` and renders inside `GuestPageShell`. Keep navigation and responsive sidebar behavior in the shell components, never inline in `App.vue` or individual views.
- Deployment automation lives under `infrastructure/terraform/`, `deploy/`, and `.github/workflows/`. Terraform and cloud-init provision the Linode host; do not reintroduce Pi, Tailscale, or Ansible infrastructure.
- Resolve deployment secrets from 1Password through `deploy/production-environment.json` and `infrastructure/sync_github_environment.py`; the server never fetches them directly.
- GitHub Actions renders and uploads the Compose release, then deploys it over strict-key-verified OpenSSH. Local Task workflows do not deploy.

### Verification Required
**Verify your changes for each task when there is any reasonable local option.**

Use minimal tools first:
- Python: `ruff check`, `pytest` (or targeted test files)
- TypeScript/Vue: linter, typecheck
- C#/.NET: `dotnet build`, `dotnet test`
- Focused `grep` or `rg` searches to verify usage patterns

**Treat verification as the default, not optional, and report what you ran.**

**Always run the relevant linter on modified files before completing a task.** Pre-commit hooks and CI enforce lint rules and will reject unclean code.

If you cannot run verification, explicitly say why and list the exact commands the user should run.

## General Principles

### Core Philosophy

**Simplicity, readability, and organization above all else.** Every decision should optimize for code that is easy to understand, intelligently organized, loosely coupled, and minimal in scope. We do not sacrifice these ideals for performance, cleverness, or theoretical future-proofing. Specifically:

- **Readability over performance.** If a simpler approach is slightly slower but far easier to understand, choose simplicity. Optimize only when there is a measured, real problem.
- **No defensive programming.** Trust your data and your contracts. Do not wrap things in try/except or try/catch "just in case," do not add fallback values for data that should always exist, and do not silently swallow errors. If something is wrong, fail hard and immediately so the problem is visible.
- **No bloat.** Every line of code, every abstraction, every helper function must earn its place. If something can be removed without losing clarity or correctness, remove it. Do not add code speculatively.
- **Avoid redundancy.** Remove unnecessary normalization/casting or defensive logic once you verify the real behavior. Before changing code, trace how it is used and what depends on it; do not assume behavior without confirming its source.
- **Prefer direct usage over one-off helpers.** If a helper is only used once and adds no clarity, inline the logic (YAGNI). Create helpers only when they meaningfully improve readability or reuse.
- **Loose coupling.** Components, modules, and services should have clear boundaries and minimal dependencies on each other's internals. Prefer passing explicit arguments over reaching into shared state.
- **Intelligent organization.** Group related things together, separate unrelated things, and make the structure of the codebase reflect the structure of the domain. A new developer should be able to find things by intuition.

### Follow Existing Architecture

Reference other views, models, serializers, admin files, apps, folders, and so on, to mirror their architecture and design patterns. When adding utilities or one-off data tasks, implement them as management commands (Django) or dedicated CLI tools (.NET) under the relevant module instead of placing scripts in the repo root.

### Reuse Existing Components

Reuse any already created classes, methods, and structures to ensure consistency and avoid duplication.

### Control Flow

Keep conditional logic shallow. Return early when possible to avoid deep nesting and make the intent clear.

### Keep Logic Simple

- Favor straightforward, explicit code even if it means repeating a line or two.
- Group related steps together to help future readers follow the intent quickly.
- **Do not over-engineer solutions**. Keep code simple, readable, and maintainable. Avoid unnecessary abstractions, helper functions, or complex patterns when a simple approach works.
- **Be deterministic**. Code should have predictable, consistent behavior.
    - If something must be uniquely identified (schema rows, tasks, entities), require the full identity at the API boundary.
    - Do not "guess" by matching on non-unique fields.
    - Avoid "best effort" fallback logic that masks underlying issues.
    - When required data is expected, access it directly and fail fast. Avoid defensive `.get` chains, repetitive type checks, or silent fallbacks that mask issues.
- **Fix root problems, not symptoms**. When encountering bugs or issues:
    - Identify and fix the underlying cause rather than adding bandaid fixes or workarounds.
    - Do not add defensive code to handle edge cases that shouldn't exist in the first place.
    - If data is malformed, fix the source of the malformation rather than adding cleanup code everywhere.
    - Fallback logic should only exist for legitimate alternative paths, not to paper over bugs.
- **Code Readability**:
    - Use logical spacing to separate chunks of code.
    - **Avoid comments or docstrings unless absolutely necessary**. Code should be self-documenting through clear naming and structure. Only add them when:
        - Explaining a complex algorithm or non-obvious logic.
        - Documenting why something is done a certain way (not what is being done).
        - Clarifying business rules or requirements that aren't self-evident.
    - Prefer full descriptive variable names and avoid abbreviations unless universally clear.
    - Eliminate redundant null checks and unnecessary intermediate variables.
- Combine conditions when they lead to the same outcome.
- **Encapsulate fragile third-party integrations**. Wrap external/fragile APIs (auth flows, vendor SDKs, network clients) behind a small service layer so views/commands/tasks only use the service interface. Keep retries, caching, token handling, and environment quirks inside the service; keep call sites thin.

### Parameters and Variables

- **Do not add unused parameters to function signatures.** Remove unused parameters instead of suppressing warnings. If a variable might be needed later, that is speculative -- keep the signature minimal and change it when requirements actually change.

### ID Generation (If Applicable)

- If you use incremental counters for IDs, make the semantics explicit (for example, whether `next()` returns the current value before incrementing) and use them consistently across the codebase.

### God Module Prevention

Large, multi-concern files are a recurring problem across all projects. Prevent god modules by:

- **One responsibility per file.** If a file handles auth, encryption, email, admin widgets, test base classes, and base models, split it. Each concern gets its own module.
- **Watch for growth signals.** When a file exceeds ~300 lines or handles 3+ unrelated concerns, it's time to split.
- **Name files by their responsibility,** not by their location. `common.py`, `utils.py`, `helpers.py` are anti-pattern names that attract unrelated code. Prefer `encryption.py`, `email_service.py`, `auth_backend.py`, `base_models.py`.
- **Apply across all stacks.** This rule applies equally to Python modules, C# classes, TypeScript files, and Vue components.

### Dead Code Discipline

- **Delete commented-out code.** Version control has history. Do not leave commented-out blocks, empty scaffold methods, or unused imports as "notes for later."
- **Remove dead dependencies.** If a package is in `requirements.txt`, `pyproject.toml`, or `.csproj` but never imported, remove it.
- **Remove dead features.** If templates, views, or entire modules are disabled/unreachable, delete them or document why they exist and when they'll be reactivated.

### Security Rules

These apply across all stacks and all projects:

- **Never commit secrets or credentials.** No `.env.secret`, no hardcoded `SECRET_KEY`, no API tokens, no database passwords in source code. Use environment variables and `.env` files that are `.gitignore`d. If secrets have been committed, rotate them immediately -- even in private repos.
- **Never ship unauthenticated API endpoints.** Every REST/API view must have explicit authentication and permission classes. An empty `authentication_classes = []` or `permission_classes = []` is a critical security bug, not a convenience.
- **Scope data to the current user/tenant.** List and detail endpoints must filter by `request.user`, `request.employee.company`, or the equivalent ownership boundary. Using `objects.all()` in an authenticated endpoint is a cross-user data leakage bug.
- **Tests must verify ownership boundaries.** If an endpoint is scoped to user A, a test must confirm user B cannot see user A's data.
- **Avoid wildcard `ALLOWED_HOSTS`.** `ALLOWED_HOSTS = ('*',)` is acceptable only in local development settings, never in production.

### Logging Discipline

- Log at boundaries (start, end, errors), not every intermediate step.
- One structured line per event -- no banner separators (`'=' * 80`), no per-iteration memory/config dumps.
- Aim for ~5 log lines per method maximum. If a method has 10+ log calls, most are noise.
- Never log data that is only useful during initial development (environment dumps, full config objects, memory snapshots). Remove these before merging.
- Use proper logging frameworks (`logging` in Python, `ILogger`/`Serilog` in .NET), not `print()` or `Console.WriteLine` in production code.
- No joke/placeholder log messages (`logger.info('Spinach')`).

### Dependency Hygiene

- **Declare all dependencies.** Every imported package must appear in the project's dependency file (`requirements.txt`, `pyproject.toml`, `.csproj`). Missing declarations break fresh installs.
- **Pin versions.** Use specific version constraints, not open-ended ranges. Unpinned deps cause non-reproducible builds.
- **Remove unused deps.** If a package is declared but never imported, remove it.
- **Keep dependency versions consistent** across projects that share packages (e.g., shared NuGet package versions between game client and server).
- GriffLab exception: `backend/pyproject.toml` and `frontend/package.json` prefer minimum-version ranges (`>=`) when the ecosystem supports them, with lockfiles providing reproducibility. Do not convert those app manifests to exact pins unless there is a concrete tooling or compatibility reason.

### Tooling & CLI Contracts

- **Honor all declared CLI arguments.** If a CLI tool accepts `--user` and `--replace` flags, it must use them. Parsing arguments and ignoring them is misleading.
- **No hardcoded user paths.** Tools must not contain paths like `/home/griffin/...`. Use environment variables, config files, or runtime discovery.
- **Use context managers for resources.** Always use `with open()` (Python) or `using` (C#) for file handles, database connections, and disposable resources.

---

## Code Review Practices

- **Verify usage before claiming redundancy.** Search for actual usage patterns before suggesting removal.
- **Gather comprehensive context first.** Understand architecture and usage patterns across the codebase before making structural recommendations.
- **Distinguish intentional design from accidental complexity.** Thin wrappers and scaffolding may be deliberate architectural choices -- verify with usage data.
- **Document architectural decisions.** When you discover why something is designed a certain way, record it.
- **Create refactoring plans before implementing.** For non-trivial changes, outline risks and verification steps before coding.

### Centralize Constants

Move feature-level constants and configuration (API IDs, project names, query templates, timeouts, sync windows) into the appropriate settings system (Django settings, `appsettings.json`, Godot project settings), and reference them from there. Avoid module-level config duplication.

**Keep query text in code.** Large SQL/query templates belong in the service/module (as constants or helpers), not in settings. Settings should hold configuration values, not full query bodies.

## Consistency Checklist (Cross-Stack)

Before completing any task, verify:

- [ ] No secrets or credentials in source code
- [ ] No god modules introduced or expanded
- [ ] No commented-out dead code left behind
- [ ] No unauthenticated endpoints added
- [ ] Data scoped to current user/tenant where applicable
- [ ] All dependencies declared and version-pinned
- [ ] Logging uses proper framework, not print/console
- [ ] Verification commands run and results reported
- [ ] This file reviewed for needed updates

---

## Intent

These guidelines exist to:
- Reduce cognitive load
- Preserve architectural integrity
- Keep the codebase approachable for new contributors
- Prevent over-engineering and accidental complexity
- Enable confident, fast iteration

They are constraints in service of clarity, not bureaucracy. If following a rule would make the code worse, pause and update the rule.
