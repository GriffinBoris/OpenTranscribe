---
name: backend-homogeneity-audit
description: >
  Find and compare backend patterns (views, serializers, services, URLs) to ensure new work matches
  established conventions in agents/AGENTS.md and avoids duplicating existing solutions.
---

# Backend Homogeneity Audit - Load IQ

## When to use
- Adding new Django views/serializers/URLs
- Introducing a service class or external integration
- Reviewing PRs for backend consistency
- Touching query param handling or error behavior

---

## Goals
- Identify common backend patterns used across apps
- Ensure new work matches `ProjectBaseAPIView`/serializer/URL conventions
- Compare new behavior against rules in `agents/AGENTS.md`

---

## Common places to search

### Views and serializers
- `backend/*/views/**/views.py`
- `backend/*/views/**/serializers.py`

### Shared query param helpers
- `backend/core/common.py`

### URL configs
- `backend/*/views/**/urls.py`
- `backend/*/urls.py`

### Services and helpers
- `backend/*/services/`
- `backend/*/utils/`

### Tests (patterns)
- `backend/*/views/**/tests/`

---

## What to sniff for (examples)

### View conventions
- Inherits `ProjectBaseAPIView`
- Uses `check_has_permission` and `require_fields`
- Query param handling follows simple conversions
- Reuse shared parsing helpers (dates, CSV lists) instead of re-implementing
- Uses standardized DRF errors (`ValidationError`/`APIException`)

### Serializer conventions
- Input/output serializers separated when needed
- `id` first in `fields`
- Output serializers set `read_only_fields = fields`

### URL conventions
- RESTful path shapes (`list/`, `create/`, `<int:pk>/`)
- Kebab-case route names

### Services
- External integrations wrapped in services
- Settings-driven configuration (`settings.<VAR>`)
- No duplicated auth/token logic in views

### Query param naming
- Backend only uses snake_case query params; do not add new camelCase params

---

## Suggested workflow

1. Identify similar endpoints or app modules.
2. Inventory existing views/serializers/URLs for that domain.
3. Compare new work against those patterns.
4. Replace duplicative logic with shared patterns.
5. Verify alignment with `agents/AGENTS.md` backend rules.

---

## Output expectations

Provide:
- A list of reused patterns/components.
- Deviations found and suggested fixes.
- File paths that establish the preferred pattern.

---

## Living document note

This skill is a living document. If you discover new backend patterns or shared helpers while auditing,
update `agents/AGENTS.md` immediately so future work stays consistent.