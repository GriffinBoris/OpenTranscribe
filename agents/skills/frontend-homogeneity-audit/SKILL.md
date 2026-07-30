---
name: frontend-homogeneity-audit
description: >
  Find and compare frontend patterns (components, composables, stores, utilities) to ensure new work matches
  established conventions in agents/AGENTS.md and avoids duplicating existing solutions.
---

# Frontend Homogeneity Audit - Load IQ

## When to use
- Adding a new Vue feature or view
- Introducing a new composable, store, or API call
- Touching shared UI patterns (errors, loading, polling, forms)
- Reviewing PRs for frontend consistency

---

## Goals
- Identify common frontend patterns used across features
- Ensure new work reuses existing components/composables/utilities
- Compare new behavior against rules in `agents/AGENTS.md`

---

## Common places to search

### Shared UI components
- `frontend/src/components/ui/`
  - `ErrorMessage.vue`
  - `LoadingSpinner.vue`
  - `LoadingProgressBanner.vue`

### Composables
- `frontend/src/composables/`
  - `useAPI.ts`
  - `usePolling.ts`
  - `useSchemaValidation.ts`
  - `useFormErrors.ts`
  - `useClipboard.ts`
  - `useCopyFeedback.ts`

### Utilities
- `frontend/src/utils/`
  - `errorHandling.ts`
  - `dateFormatting.ts`
  - `numberFormatting.ts`

### Stores
- `frontend/src/stores/` (Pinia patterns, error/loader conventions)

### Feature views/components
- `frontend/src/features/*/views/`
- `frontend/src/features/*/components/`

---

## What to sniff for (examples)

### API usage
- `api.<domain>` calls (verify domain module exists in `useAPI.ts`)
- Param casing (camelCase → `buildParamsConfig`)
- Use of shared response types (`frontend/src/types/`)

### Error handling
- `ErrorMessage` usage (message + retry actions)
- `useFormErrors` for forms
- `utils/errorHandling.ts` helpers

### Loading states
- Shared `LoadingSpinner` / `LoadingProgressBanner`
- Avoid custom spinners if shared ones exist

### Polling/refresh
- Use `usePolling` instead of manual intervals
- `shouldStop` for error/complete state + UI note when paused

### Form patterns
- `useSchemaValidation` + `useFormErrors`
- Cascading selects use `resetField` instead of assignment

### Clipboard actions
- Prefer `useClipboard` / `useCopyFeedback` over direct `navigator.clipboard`

### Date preset controls
- Reuse existing date preset patterns before creating new UI

---

## Suggested workflow

1. Identify similar feature(s) under `frontend/src/features/`.
2. Inventory existing components/composables used there.
3. Compare new work to these references.
4. Replace duplicative logic with shared patterns.
5. Verify alignment with `agents/AGENTS.md` frontend rules.

---

## Output expectations

Provide:
- A list of reused components/composables/utilities.
- Deviations found and suggested fixes.
- File paths that establish the preferred pattern.

---

## Living document note

This skill is a living document. If you discover new shared components, utilities, or patterns while auditing,
update `agents/AGENTS.md` immediately so future work stays consistent.