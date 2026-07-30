---
id: project-guidance
title: Project Guidance
description: Repository-specific guidance, tooling, and architecture decisions for Project.
kind: guidance
scope: project
name: project
tags:
  - project
applies_to:
  - project
status: active
order: 0
---

# Project Guidance

## Purpose

- Capture repository-specific conventions, local tooling, and product architecture decisions.
- Keep general development rules in `agents/guidance/guidance.md`.
- Use this file for decisions tied to this repo's structure, migration state, or product direction.

## Guidance Authoring And Migration

### Authored Guidance

- Update authored guidance under `agents/guidance/` instead of editing generated `AGENTS.md` output directly.

### Guidance Map

| File | Scope |
|---|---|
| `agents/guidance/guidance.md` | Shared development guidance |
| `agents/guidance/languages/python/guidance.md` | Python guidance |
| `agents/guidance/frameworks/django/guidance.md` | Django, DRF, and Celery guidance |
| `agents/guidance/frameworks/vue/guidance.md` | Vue, TypeScript, Pinia, and frontend API guidance |
| `agents/guidance/project/guidance.md` | Project-specific guidance |
| `agents/reference/antipatterns/overview.md` | Cross-stack anti-pattern catalog |
| `agents/reference/review/` | Review rubric and reporting templates |

### Skills And Commands

- OpenCode command content currently spans `agents/content/commands/` in the new tree and `agents/opencode/commands/` in the legacy tree.

## Repository Layout

- **Backend**: Django REST Framework app in `backend/` when present.
  - Apps usually follow Django conventions with `models.py`, `views.py`, `serializers.py`, `urls.py`, `admin.py`, and `tests/`.
  - Base classes live in `core/base_models.py` (`ProjectBaseModel`) and `core/base_views.py` (`ProjectBaseAPIView`, `ProjectBaseAuthenticatedAPIView`).
- **Frontend**: Vue 3 and TypeScript in `frontend/` when present.
  - Common directories include `src/components/`, `src/views/`, `src/composables/`, `src/types/`, `src/core/`, `src/styles/`, and `src/utils/`.
  - Route folders under `src/views/` can own their route component, subcomponents, local store modules, and route-specific helpers.
  - Shared shell-level frontend state should live under `src/views/application/`, and the canonical API client should live under `src/utils/api.ts`.

## Local Verification And Tooling

### Build, Lint, And Test Commands

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

**Backend (direct Django and Python commands):**

```bash
pytest
pytest path/to/test_file.py
pytest path/to/test_file.py::TestClass
pytest path/to/test_file.py::TestClass::test_method

ruff check
ruff check --fix
ruff format

python manage.py migrate
python manage.py makemigrations
python manage.py makemigrations app_name

python manage.py runserver
python manage.py shell
```

**Frontend (Vue and TypeScript, if present):**

```bash
cd frontend

npm run dev
npm run type-check
npm run lint
npm run format
npm run build
npm run preview
```

**Docker (if present):**

```bash
task docker:start
task docker:start:detached
task docker:stop
task docker:bootstrap
task docker:logs SERVICE=web
```

### Tooling Notes

- If the repo exposes a context7 MCP server or web search, use them only when needed and within repo rules.
- If the repo has custom CLI tooling, prefer it over ad hoc scripts.
- Python tooling is centralized in the repo-root `pyproject.toml` even when hooks run from `backend/`; keep formatter and import-sorter indentation aligned there to avoid Ruff and isort reformat loops.
- For repo-local machine automation, keep Task entrypoints in `tasks/*.yml`, put scripts in `scripts/<feature>/`, and write runtime PID, log, and state files to a gitignored repo-root dot directory.

## Current Architecture Decisions

### Django Repository Conventions

- Base backend classes live in focused `core/` modules: `ProjectBaseModel` in `core/base_models.py`, and `ProjectBaseAPIView` plus `ProjectBaseAuthenticatedAPIView` in `core/base_views.py`.
- Authenticated views inherit from `ProjectBaseAuthenticatedAPIView` (`SessionAuthentication` + `IsAuthenticated`); use `ProjectBaseAPIView` only for anonymous-safe endpoints.
- There is no shared permission/field-helper layer on the base view. Scope every authenticated endpoint to the current household by resolving `HouseholdMember.objects.get(user=request.user).household`, validate payloads with `serializer.is_valid(raise_exception=True)`, and look up owned objects with `get_object_or_404(Model, pk=pk, household=household)`.
- Models should continue extending `ProjectBaseModel`.
- When model audit tracking is needed, use `history_log_fields` or `history_log_private_fields` and pass `log_user_id` into `save()`.
- Existing backend relationships in this repo follow `on_delete=models.DO_NOTHING`; keep that convention unless you are making a deliberate migration with a clear data-integrity plan.
- Keep `core/` concerns in focused modules (`base_models.py`, `base_views.py`, `auth_backends.py`, `encryption.py`, `email.py`, `google_auth.py`, `google_sso.py`, `admin_widgets.py`, `test_fixtures.py`). Do not reintroduce a `core/common.py` catch-all; when touching one concern deeply, keep it in or extract it to its own module.
- This repo uses `drf-standardized-errors`; in views, raise `rest_framework.exceptions.ValidationError`, and in serializers, keep field validation in `serializers.ValidationError` so responses stay in the standardized shape.
- Shared backend fixture builders belong in `core/test_fixtures.py`, and tests should mirror those helpers instead of creating one-off builders in each module.
- When view tests need model permissions, resolve them through the model helper methods the repo already uses and grant them explicitly.
- The Django settings chain in this repo is `base.py -> dev_local.py -> dev_pytest.py -> dev_docker.py -> dev_github.py -> production.py`.
- Keep build-safe defaults in `base.py` so image-time commands such as `collectstatic` do not require production secrets; production settings must still require the real runtime values.
- GriffLab user sessions are Google SSO-only. Do not reintroduce password login, registration, password-change endpoints, frontend token handling, or the Google One Tap SDK. The frontend navigates to Django's Google SSO login endpoint; Django validates state, exchanges the authorization code, verifies the ID token, creates the session, and redirects back to the SPA.
- Apply staff and superuser access only at the verified Google identity boundary. GriffLab's owner account is configured through `GOOGLE_SUPERUSER_EMAIL` rather than from request data.
- Background task wiring follows the task-app pattern already used in the repo: add behavior on the task model or task class, register it in the task map, expose a Celery entrypoint, schedule it when needed, and add tests in the task app.
- When the task framework supports progress reporting, use `self.set_message_and_percent(...)` instead of inventing parallel progress tracking.
- ComputerCraft browser APIs remain household-scoped session endpoints. Turtle device APIs use short-lived, single-use enrollment credentials followed by a unique hashed bearer token per turtle; never expose or persist plaintext device tokens on the browser side.
- The ComputerCraft fleet communicates through outbound HTTPS polling rather than Rednet. Keep GPS optional, require a trusted manually configured pose when GPS is unavailable, and preserve turtle movement checkpoints locally while keeping the active task and latest reported task checkpoint on the server. Startup recovery must re-dispatch running work after a chunk reload without creating a replacement task.
- Quarries are fault-tolerant: a quarry a turtle reports FAILED auto-requeues to the household's pending pool (preserving the same task record, payload, and reported checkpoint) so any eligible turtle finishes it, and a quarry whose turtle has gone silent past `COMPUTER_CRAFT_QUARRY_RECLAIM_MINUTES` (or is disabled/deregistered) is reclaimed the same way by the periodic assignment sweep. Both are bounded by `COMPUTER_CRAFT_QUARRY_MAX_AUTO_RETRIES`, and mining to a deeper level clears that counter, so only a quarry that never makes progress gives up and stays FAILED for manual attention. Manual Retry resumes on the same turtle; manual Reassign returns it to pending and excludes the turtle that failed it for that handoff; both reset the auto-recovery budget. Only quarries auto-cycle this way — other task types record their terminal result as reported.
- An idle turtle check-in must immediately assign eligible pending work for its household and dispatch it in the same response. Celery's periodic assignment task is only a backstop, not the normal pickup path.
- Keep the ComputerCraft installer as a small static recovery bootstrap, but serve the full turtle agent from immutable, database-backed releases. Agent publication and rollback are staff-only; every source keeps the version placeholder, turtles report the running version in telemetry, and self-update retains the previous agent until the new version completes a successful check-in.
- A turtle reinstall must issue a fresh, single-use enrollment code for the existing turtle record. Revoke its old bearer token and cancel its active work, but preserve its server-configured pose and home so a replacement local installation can apply that configuration on first check-in.
- When a newly enrolled turtle presents a computer ID held by a deregistered tombstone, release that tombstone's computer ID and token before assigning the ID to the new record. Continue rejecting collisions with active turtle records.
- CC:Tweaked HTTP calls must retain the third failure-response return value. Non-2xx responses otherwise collapse into generic errors such as `Bad Request`, hiding the server's structured validation body and preventing actionable device diagnostics.
- Every CC:Tweaked enrollment, agent download, update, and check-in request must use the table-form HTTP API with a 30-second connection timeout. The initial check-in still fails visibly so the startup supervisor can retry; active task checkpoints remain persisted while the agent retries later check-ins.
- CC:Tweaked does not guarantee response-header casing. Resolve response headers case-insensitively, and derive an updated agent's version from its rendered `AGENT_VERSION` when an intermediary omits the version header.
- A turtle agent's initial check-in must fail visibly when the server is unreachable or rejects the token. The startup supervisor owns retries; do not let the agent convert an initial transport failure into a silent idle state.
- Use `textutils.empty_json_array` for empty Lua collections sent to API fields that require JSON lists. CC:Tweaked serializes a plain empty table as `{}`, which DRF rejects for list serializers such as turtle inventory.
- Treat server-configured turtle pose and device-reported pose as separate state. Configuration changes remain pending until the turtle reports the applied configuration version, and stale telemetry must never overwrite the server's desired pose or revive a paused or cancelled task.
- When a manual drop-off, refuel, or service task interrupts a quarry, persist the quarry return waypoint before leaving. Resume through the service home’s safe-travel Y, move horizontally over that waypoint, and descend only once the turtle is back at the quarry.
- Deregister a turtle as a tombstone: cancel its active work, hide it from the household fleet, and retain its bearer-token record only long enough for a final check-in to tell the agent to erase its local state. Do not delete the record or token before that acknowledgement path is available.
- Device endpoints that return a non-JSON artifact must declare a renderer for the artifact media type. DRF performs content negotiation before the view runs, so a turtle's explicit `Accept: text/plain` header otherwise receives a 406 even when the view returns an `HttpResponse`.
- Auto-crafting is server-planned and turtle-executed. Recipes are household-scoped `TurtleRecipe` rows (one per output item) storing either a nine-cell crafting grid or a single smelt input. The recursive planner (`services/crafting_planner.py`) expands a target and quantity into an exact base-material list (shared ingredients counted once via a topological pass), an ordered step list, and the recipe subtree; the CRAFT task payload carries the base-material list, the recipe subtree, and the total operation count (the ordered step list is for the plan-preview endpoint and dashboard — the turtle re-derives build order from the recipe subtree). The turtle then fulfils demand-first: it reuses whatever stock is already in the store and only crafts or smelts the shortfall, so a CRAFT task is naturally idempotent and restarts cleanly after a reboot.
- A crafter is a dedicated stationary turtle flagged `is_crafter`; CRAFT tasks are directed at a chosen crafter (never auto-assigned to a roaming miner) and are non-movement, so they dispatch even when the turtle's world position is untrusted. The 3x3 recipe grid maps to turtle slots 1-3 / 5-7 / 9-11 (the inventory is four wide) with every other slot empty during a craft, and slot precision always comes from a wrapped chest via the modem-free `pushItems`-to-slot-1 + `turtle.suck` trick (neither a blind suck nor an RS export can target a slot).
- The agent's CRAFT engine is store-agnostic through a `store` adapter (`count`/`pull`/`stow`/`deliver`/`snapshot`), and `runCraft` auto-selects the adapter: `peripheral.find("rsBridge")` present → Refined Storage mode, else the plain-chest mode. Chest mode uses a LEFT input chest (raw materials + intermediates), a RIGHT output chest, and a FRONT furnace. RS mode (Advanced Peripherals RS Bridge) treats the whole network as the store — "use existing" checks `bridge.getItem(...).amount` across the network and finished items go straight back in — with a RIGHT buffer chest for slot-precise transfers and the RS Bridge on the LEFT (`exportItemToPeripheral` into it, `importItemFromPeripheral` out) and the FRONT furnace still used only when the network can't already supply a smeltable.
- The FRONT furnace auto-ejects its output back into the store: a hopper into the LEFT input chest in chest mode, or an RS importer into the network in RS mode. `smeltRuns` therefore loads the furnace and waits for the smelted item to appear in the store (polling `store.count`, bounded by cook time plus slack) instead of sucking the furnace, which could otherwise grab fuel or uncooked input. This is one mode-agnostic path and is the mitigation for the vanilla side-insertion extraction problem.
- The craft preview's have/need/short comes from a real turtle round-trip, because only the turtle can see its store. A `STOCK_CHECK` task (non-movement, pausable, kept out of the fleet task panels) carries the build's item set; the agent's `runStockCheck` reads the store once via `store.snapshot(items)` and reports counts, then the server's `CraftingPlanner.resolve` runs the same demand-first pass to return per-item `{need, have, short}` and `buildable`. Keep this resolution server-side and unit-tested, mirroring the "server plans, turtle reports" split rather than computing shortfalls on the device or in the browser.
- Stock (and any item-id-keyed data) must cross the browser API as a **list of `{item, count}`**, never a map keyed by item id. The frontend `ApiClient` rewrites object keys between camelCase and snake_case, which silently mangles ids like `minecraft:oak_log` → `minecraft:oakLog`; carrying ids as values sidesteps it. This is the actionable form of the "item-id keys are only safe when never read" note.

### Deployment Conventions

- Production runs on one Terraform-managed Linode VPS. Terraform and cloud-init own host creation, Docker installation, firewalling, SSH hardening, and the `deploy` account. Do not reintroduce Ansible, Tailscale, Swarm, or Pi-specific provisioning.
- `griffinboris.com` is a shared Linode DNS zone. GriffLab looks it up and owns only `home` records; do not add a second `linode_domain` resource for the shared zone.
- GitHub Actions builds `linux/amd64` GHCR images and deploys them with ordinary OpenSSH. Public HTTP/HTTPS are required for Caddy certificate issuance; SSH remains key-only and uses strict host-key verification.
- Production deployment intentionally stops the Compose stack and runs `docker system prune -af` before pulling and restarting services because host storage is constrained. Do not add volume pruning to this sequence; PostgreSQL data must survive deployment cleanup.
- Build and deploy separate web, Celery, Nginx, and custom PostgreSQL images. The custom PostgreSQL image owns initialization; do not add a parallel server-side initialization script.
- The web Dockerfile must copy built SPA templates and static files into the installed `core` app. Django discovers `api/index.html` through `core/templates/api/`; do not copy those artifacts into the uninstalled `api` package.
- Vite production builds use `/static/` as their asset base and `api` as their asset directory. Do not rewrite only the initial HTML asset URLs: lazy-loaded chunks must also resolve under `/static/api/`.
- Keep Terraform and cloud-init under `infrastructure/terraform/`, production Compose/Caddy/rendering assets under `deploy/`, and operator setup in `infrastructure/setup-runbook.md`.
- Treat `deploy/production-environment.json` as the source of truth for production values, 1Password references, GitHub Environment targets, and image component names. GitHub Actions renders generated deployment files into `.deploy/`; do not commit environment files.
- HCP Terraform owns provider credentials, state, the deploy public key, and the sensitive host-key variable. 1Password owns app secrets, the deploy private key, and the source copy of the pinned host key. GitHub Actions deploys only from workflows; local operators use the 1Password sync script but do not deploy from Task.
- Preserve the raw output when syncing the native 1Password SSH private-key field. Its terminal newline is part of the valid OpenSSH key file; trim ordinary one-line secret fields only.

### Frontend Repository Conventions

- The canonical frontend API client in this repo is `ApiClient`; keep Axios imports there and route domain modules through it.
- In this repository, `ApiClient` lives in `src/utils/api.ts` rather than a top-level `src/services/` folder.
- Use `buildParamsConfig` with camelCase params and let the API client handle casing conversion.
- Reuse the repo's shared components before adding view-local replacements: `PageStatusCard` for loading/error/retry states, `StatCard` for metric tiles, `AppSurface` for card surfaces, `AppSparkline` for line charts, and the shared `Form*`/`App*` inputs (`FormField`, `AppSelect`, `AppInputText`, `AppButton`, `AppIcon`).
- Use `useThemeColor` from `src/composables/useThemeColor.ts` for theme-aware chart colors bound through SVG presentation attributes.
- Parse DRF standardized errors through `src/utils/errorHandling.ts` (`mapApiErrorsToFields` for form field maps, `getFirstApiErrorMessage` for workflow fallback messages) instead of reading `AxiosError` shapes directly in stores or views.

## Consistency Checklist

### Project

- Project-specific rules are captured here instead of being mixed into global guidance.
- Repository layout, local tooling, and migration notes are explicit.
- Cross-cutting product and architecture decisions that are not general development rules live here.
