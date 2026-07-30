## Preserved Guidance (Project-Specific Content)

This file archives project-specific content that was displaced from the core guidance files (`AGENTS.md`, `django.md`, `frontend.md`) when they were made generic. This content is still valid for its respective projects but does not belong in cross-project guidance.

---

### Project-A Specific Patterns (from former AGENTS.md)

#### Proof Requirement

When completing any request in Project-A, output the agreed proof phrase exactly as specified in that project.

#### Verification Commands

```bash
task backend:lint        # ruff check for Python
task backend:test        # pytest for backend
task frontend:lint       # TypeScript/Vue linting
task frontend:typecheck  # TypeScript validation
```

#### Docker Images

- When a service imports the shared library, copy `shared_lib/` into the image and install it (mirror the MCP Dockerfile pattern used in that project).

#### Skills Available

Project-A exposes specialized agent skills. See `.github/skills/` for the canonical list and follow their `SKILL.md` instructions.

#### PydanticAI Agent Patterns

These patterns are specific to the AI agent feature:

- Prefer `message_history=` over embedding prior messages into a giant prompt string.
- If `message_history` is provided and non-empty, PydanticAI will **not** generate a new system prompt. Ensure your history includes a `SystemPromptPart`.
- Keep stored history as a simple list of `{role, content}` (Slack-style) if needed, and convert to `ModelRequest`/`ModelResponse` at runtime.
- Inject summaries of active skills and user memories into the system prompt.
- Provide read-only tools to fetch skill/memory details when needed.
- Use `UsageLimits` (at least `tool_calls_limit`) to prevent runaway tool loops.
- If an agent calls another agent inside a tool (delegation), pass `ctx.usage` into the delegate `agent.run(..., usage=ctx.usage)` so totals aggregate.

#### Testing Note

Tests should not be created for the legacy external integration app, as it connects to a third-party database that is not available in the test environment.

#### Test Organization

- Add reusable object builders to `core/test_fixtures.py`.
- Place metadata serializer and view tests alongside their feature modules under `metadata/views/<feature>/tests/`, and keep metadata model tests in `backend/metadata/tests`.

---

### Project-A Specific Patterns (from former django.md)

#### ProjectBaseAPIView Helpers

- All views must inherit from `ProjectBaseAPIView` in `core/common.py`.
- Call `check_has_permission` for permission checks and `require_fields` for validating request payloads.
- For custom permissions (non-CRUD), build the string with `ProjectBaseModel.get_custom_permission('codename', app_label='core')`.

#### Background Tasks (Project-A Specific)

Task model pattern specific to the task app structure:

1. Add method to `backend/task/models.py` in the `Task` class.
2. Register in `task_map` dictionary.
3. Create Celery task in `backend/task/celery.py`.
4. Schedule in `setup_periodic_tasks()`.
5. Test in `backend/task/tests/`.

#### Celery App Name

The Celery app name in `task/celery.py` should match the project slug, not a hardcoded name from another project (for example, the cookiecutter template hardcodes a different slug).

#### Management Commands

When adding Django utilities or one-off data tasks, implement them as management commands under the relevant app instead of placing scripts in the repo root.

---

### Project-A Specific Patterns (from former frontend.md)

#### PrimeVue Component Conventions

- Prefer shared PrimeVue inputs (`Select`, `IconField` + `InputText`, `Chip`) and keep spacing consistent with Tailwind utility classes.
- For PrimeVue Dialog components, set `:draggable="false"` to prevent positioning issues and use `pt-4` padding on content wrapper to prevent header overlap.
- For searchable dropdowns, default to `Select` with `:filter="true"`, `optionValue`, `optionLabel`, `showClear`, and `size="small"` to match existing form styling.

#### Route Naming

- For pages adjacent to the main feature that are not the main list or wizard, prefer the shared management route prefix to avoid sidebar collisions.

#### Date Preset Controls

When adding date preset controls, reuse existing patterns from:
- The existing feature filter control components in the project (search for `FilterControls.vue` inside feature folders).

#### Model Organization

- Organize models by domain under `src/core/models` (for example, `load`, `loadcolumn`, `operator`).

#### Outdated Frontend Model Conventions

The following conventions are currently deprecated but retained for historical context:
- Naming patterns like `[ModelName]Interface.ts`, `[ModelName]InputInterface.ts`, `[Action]RequestInterface.ts`, `[ModelName][Action]InputInterface.ts`.
- Interface structure and enum guidelines that enforce camelCase properties and PascalCase enum names.
- Factory function conventions such as `createDefault[ModelName]()`. Do not reintroduce these until guidance changes.

---

### Project-B Specific Notes

#### Multi-Tenant Middleware

A multi-tenant middleware scopes all queries to `request.employee.company`. This is a well-designed pattern that other projects should consider adopting.

#### OpenAPI / TypeScript Codegen

This project has a `generate_api_client.py` script that generates TypeScript interfaces from the OpenAPI schema. This is a good concept but needs to become a repeatable pipeline if the team commits to maintaining it.

---

### Project-C Specific Notes

#### ETL Pipeline

This project has a well-structured ETL pipeline with abstract base classes (`BaseExtractor`, `BaseLoader`, `BaseRawProduct`). The blue-green feed deployment pattern (create inactive feed, then swap) is a good pattern for similar data pipeline work.

#### Known Bug: Alert Ownership

Alert list endpoints (`backend/api/alert/views.py`) use `objects.all()` without filtering by `request.user`. This is a critical security bug. Tests codify this insecure behavior as expected. Both the views and tests need to be fixed.

---

### Project-D Specific Notes

#### Legacy Status

This is a legacy project. Accept the following patterns within it, but do not replicate them:
- JavaScript instead of TypeScript.
- Vuex instead of Pinia.
- Raw `axios` without a centralized API client.
- Legacy-prefixed component library (LegacyButton, LegacyComboBox).
- Options API and Composition API mixed in the same component.
- Django-Cron instead of Celery for scheduled tasks.
- Global Vuex store with broad responsibilities.

#### Feature Flags

This project uses `django-constance` for runtime feature flags. This is a good operational pattern.

#### Encrypted Fields

This project uses Fernet encryption for sensitive data (Twilio creds, payment card info). This is a good security practice.

#### Known Bug: Unauthenticated Base View

The legacy base REST view in `api/common.py` has empty `authentication_classes` and `permission_classes`. The base REST view has no authentication. This is a critical security bug.

#### Known Bug: Production Secrets in Repo

`.env.secret` contains production secrets in plaintext (payment processor keys, database credentials, encryption keys, messaging credentials, email/SaaS keys, error tracking DSNs). These need immediate rotation.

---

### Game Project-Specific Notes

#### Known Bug: AddLong Serialization

`GameShared/Network/IPacket.cs:159` -- `AddLong` writes `SentTimestamp` instead of the `l` parameter. Any long value is silently replaced with the timestamp.

#### Known Bug: Pathfinding FCost

`GameShared/Game/Map/PathNode.cs:15` -- `FCost` computes `HCost + HCost` instead of `GCost + HCost`. A* pathfinding is broken.

#### Known Bug: Server CancellationToken Reuse

`GameServer/Services/ConnectionService.cs:28-29` -- `CancellationTokenSource` is reused after cancellation, permanently stopping connection acceptance after 10-second idle.

#### Known Bug: GetOpenTeam Crash

`GameServer/Services/SessionService.cs:113` -- `.First()` on empty collection when no players exist.

#### Code Generation Architecture

The code generation pipeline (YAML -> Pydantic -> Jinja2 -> C#) is split between:
- `GameShared/Game/CodeGen/` (current, integrated into shared project)
- `GameCodegen/` (predecessor, standalone Python tool)

The shared project's codegen is the canonical version. The consolidator is kept for reference but is not actively maintained.
