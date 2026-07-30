---
name: architecture-audit
description: >
  Audit Load IQ codebase for simplicity, readability, and clean architecture.
  Use when reviewing PRs, refactoring, or checking if code has become unnecessarily complex.
---

# Architecture & Readability Audit - Load IQ

## When to use
- Reviewing PRs that add/change structure or introduce new patterns
- Code feels harder to navigate or understand
- Suspecting mixed responsibilities or tight coupling
- Before shipping: "is this still clean and simple?"
- Refactoring: "did we improve or just move complexity around?"
- Files are growing large (>500 lines) or feeling unfocused
- Unsure whether to split or consolidate files/folders
- Difficulty finding code ("where would this logic live?")
- Adding new Django apps or Vue features
- Error handling inconsistencies across backend/frontend

---

## Core principles

### Follow existing architecture
- **Backend**: Django REST Framework with standardized errors
- **Frontend**: Vue 3 + TypeScript + Pinia + PrimeVue
- Reference other views, models, serializers, components for patterns
- Don't introduce new patterns when existing ones work

### Simplicity over cleverness
- Use standard Django/Vue patterns that any developer would recognize
- Favor straightforward, explicit code even if it means repeating a line or two
- Skip docstrings and comments unless the flow is complex or non-obvious
- Return early to avoid deep nesting
- Simple `if/else` over complex comprehensions when clearer
- Standard library/framework features over custom implementations

### Readability
- Code should read like prose: obvious intent, clear flow
- Names should make comments unnecessary
- Control flow should be easy to follow top-to-bottom
- No surprises or hidden behavior
- Variable names reveal intent

### Modularity
- One responsibility per module/file/function/component
- Clear boundaries with minimal surface area
- Dependencies point in one direction (no cycles)
- Easy to understand a piece in isolation

### Not over-engineered
- No "framework-y" helpers that obscure what's happening
- No configuration for configuration's sake
- No layers added for hypothetical future needs
- No generic solutions to specific problems
- Don't abstract until you have 3+ similar real use cases

---

## Audit workflow

### 0. Pre-Audit Completeness Check (5 minutes)

**Backend scope:**
```bash
# Count Python files and lines
find backend -name "*.py" | wc -l
find backend -name "*.py" -exec wc -l {} + | tail -1

# List largest Python files
find backend -name "*.py" -exec wc -l {} + | sort -rn | head -20

# Count by app
for app in backend/*/; do echo "$app: $(find $app -name '*.py' | wc -l) files"; done
```

**Frontend scope:**
```bash
# Count Vue/TS files and lines
find frontend/src -name "*.vue" -o -name "*.ts" | wc -l
find frontend/src -name "*.vue" -o -name "*.ts" -exec wc -l {} + | tail -1

# List largest Vue/TS files
find frontend/src -name "*.vue" -o -name "*.ts" -exec wc -l {} + | sort -rn | head -20

# Count by feature
for dir in frontend/src/features/*/; do echo "$dir: $(find $dir -name "*.vue" | wc -l) components"; done
```

**Thoroughness requirement:**
- For full codebase audits: Read ALL files >100 lines
- For PR audits: Read all changed files + related modules
- For feature audits: Read the feature + its API/store/types
- Start with entry points, then core types, then implementations

**At the end of audit:**
- Verify you've at least skimmed every file >50 lines
- If you skipped files, explicitly state "Not reviewed: X, Y, Z" in output
- Acknowledge any blind spots in the audit scope

---

### 1. Understand the scope (5 minutes)

**For full codebase audits:**
- Backend: How many Django apps? What are they?
- Frontend: What are the main features? (`features/`, `stores/`, `components/`)
- What are the 5-10 largest files? (Probable complexity hotspots)
- High-level architecture (Django → DRF → Vue → Pinia)?

**For PR/change audits:**
- What problem does this solve?
- What's the core mechanism?
- Backend change, frontend change, or both?
- Can you explain it in one sentence?

**For feature audits:**
- What's the user-facing feature?
- Backend endpoints involved?
- Frontend components/views?
- Data flow: API → Store → Component?

If you can't explain it simply, that's your first finding.

---

### 2. Check structure & responsibilities

**Backend (Django) module organization:**
- [ ] Each app has one clear responsibility (not "utils", "helpers", "common")
- [ ] App names describe what they do: `metadata`, `mitty`, `load_builder`, `snowflake_management`
- [ ] Related views/models/serializers grouped within app
- [ ] No "junk drawer" apps accumulating random functionality
- [ ] `core` app is for shared infrastructure only, not business logic

**Backend file structure:**
- [ ] Views in `views/` subdirectory with sensible grouping
- [ ] Serializers in `serializers/` or alongside views
- [ ] Models in `models/` or `models.py`
- [ ] Each view file is 100-400 lines (sweet spot)
- [ ] No view files >800 lines (god objects)

**Backend patterns:**
- [ ] All views inherit from `ProjectBaseAPIView`
- [ ] Input/output serializers split when needed
- [ ] URL patterns follow `{model}-{action}` naming
- [ ] Permission checks use `check_has_permission()`
- [ ] Field validation uses `require_fields()` or serializer validation

**Frontend (Vue) module organization:**
- [ ] Features organized by domain in `features/`: `loadbuilder`, `loadApprovals`, `dailyMonitor`
- [ ] Shared UI in `components/ui/`
- [ ] Stores in `stores/` with clear naming
- [ ] Types in `types/` with domain grouping
- [ ] Services in `services/` (API client, auth)

**Frontend file structure:**
- [ ] Feature folders contain: `components/`, `views/`, sometimes local composables
- [ ] Vue components are 100-400 lines (ideal)
- [ ] Components >500 lines reviewed for extraction opportunities
- [ ] Store files are focused (one domain per store)

**Frontend patterns:**
- [ ] Components use PrimeVue consistently
- [ ] API calls go through `api` client with camelCase/snake_case conversion
- [ ] Stores follow Pinia composition API pattern
- [ ] Reactive state accessed directly (no destructuring)
- [ ] Forms use composables for validation/errors

**File size and splitting:**
- [ ] Backend: Views 100-400 lines, models 100-500 lines
- [ ] Frontend: Components 100-400 lines, stores 100-300 lines
- [ ] Files >500 lines reviewed for split opportunities
- [ ] Files >800 lines are red flags (likely mixed concerns)
- [ ] No premature splitting (avoid 10-50 line files unless distinct concepts)

**When to split a file:**
- ✅ Multiple unrelated types/models in one file
- ✅ File mixes layers (e.g., views + business logic + data access)
- ✅ File has multiple "sections" with blank line separators
- ✅ You naturally say "this handles X *and* Y" (not "X with Y")
- ✅ Function/method names need prefixes to disambiguate

**When NOT to split a file:**
- ❌ File implements one cohesive concept with helpers
- ❌ Splitting would expose internal helpers via `pub(crate)` equivalents
- ❌ Code is tightly coupled and would need circular imports if split
- ❌ File is already well-organized with clear sections
- ❌ Splitting just to hit arbitrary line limits

**Folder organization:**
- [ ] Backend apps represent domains, not code types
  - ✅ Good: `metadata/`, `load_builder/`, `snowflake_management/`
  - ❌ Bad: `models/`, `views/`, `serializers/` at project root
- [ ] Frontend features represent user features
  - ✅ Good: `loadbuilder/`, `dailyMonitor/`, `loadApprovals/`
  - ❌ Bad: `forms/`, `tables/`, `buttons/`
- [ ] Folder depth is 2-4 levels (avoid deep nesting)
- [ ] Each folder has a clear purpose you can state in one sentence
- [ ] Sibling folders are at similar abstraction levels

**Signs of poor organization:**
- 🚩 Backend: Apps named `utils`, `helpers`, `common` with actual business logic
- 🚩 Frontend: Folders like `components/misc/`, `views/old/`
- 🚩 Files named `types.ts`, `utils.py`, `helpers.ts` with unrelated code
- 🚩 Files >1000 lines mixing responsibilities
- 🚩 Folders with 20+ files at the same level
- 🚩 Folders with only 1-2 files (over-organization)
- 🚩 Need to modify 5+ files for a simple change
- 🚩 Can't predict where to find functionality

---

### 2.5. Hunt for duplication (10 minutes)

**Backend duplication:**
```bash
# Find similar view method names
grep -r "def get\|def post\|def put\|def delete" backend/ --include="*.py" | cut -d: -f2 | sort | uniq -c | sort -rn

# Find duplicated serializer patterns
grep -r "class.*InputSerializer\|class.*OutputSerializer" backend/ --include="*.py"

# Look for duplicated validation logic
grep -r "raise ValidationError\|serializer.is_valid" backend/ --include="*.py" | wc -l
```

**Frontend duplication:**
```bash
# Find similar component names
find frontend/src -name "*.vue" | xargs basename -a | sort | uniq -c | sort -rn

# Find duplicated API calls
grep -r "await api\." frontend/src --include="*.vue" --include="*.ts" | cut -d. -f3-4 | sort | uniq -c | sort -rn

# Look for duplicated error handling
grep -r "catch.*error\|try {" frontend/src --include="*.vue" --include="*.ts" | wc -l
```

**Pattern recognition:**
- [ ] Backend: Multiple views with identical structure (CRUD patterns)
- [ ] Backend: Copied permission checks or validation logic
- [ ] Frontend: Components with similar form layouts
- [ ] Frontend: Duplicated error handling in stores
- [ ] Frontend: Repeated data fetching patterns

**Severity guidelines:**
- **Blocker**: Duplicated security/permission/validation logic (must fix)
- **Important**: >20 lines duplicated in 3+ places (maintenance hazard)
- **Polish**: Minor boilerplate duplication (annoying but not critical)

---

### 3. Review API & interfaces

**Backend API design:**
- [ ] All views inherit from `ProjectBaseAPIView`
- [ ] Permission checks at start of methods
- [ ] Query params handled consistently (simple conversion, trust internal API)
- [ ] Serializer validation before processing
- [ ] Response format consistent (DRF standardized errors)
- [ ] URL patterns follow conventions

**Backend response patterns:**
- [ ] GET (list): `Response(serializer.data, status=200)`
- [ ] GET (detail): `Response(serializer.data, status=200)`
- [ ] POST (create): `Response(serializer.data, status=201)`
- [ ] PUT/PATCH: `Response(serializer.data, status=200)`
- [ ] DELETE: `Response(status=204)`
- [ ] Async tasks: `Response({'task_id': task.id}, status=202)`
- [ ] Errors: Let DRF standardized errors handle

**Frontend API integration:**
- [ ] All API calls through `apiClient` or `api` composable
- [ ] Automatic camelCase/snake_case conversion
- [ ] CSRF token handling in interceptor
- [ ] Error responses parsed correctly
- [ ] Auth errors trigger custom event

**Function design (both backend/frontend):**
- [ ] Functions do one thing
- [ ] Function names are verbs describing the action
- [ ] Parameters are all used (no unused params)
- [ ] Return types are clear
- [ ] Side effects are obvious (or absent)

**Size & complexity:**
- [ ] Functions are < 50 lines (ideally < 20)
- [ ] Nesting is < 3 levels deep
- [ ] No functions that are hard to name
- [ ] Helper functions extracted when logic genuinely repeats

---

### 4. Assess readability

**Clarity:**
- [ ] Variable names reveal intent
- [ ] Control flow is linear and obvious
- [ ] No "clever" code that requires careful reading
- [ ] State changes are explicit
- [ ] Error paths are clear

**Simplicity:**
- [ ] Backend: Simple loops over complex comprehensions when clearer
- [ ] Frontend: Simple `if/else` over complex ternaries when clearer
- [ ] Avoid deep nesting (early returns, guard clauses)
- [ ] No overly long chains of method calls/property access
- [ ] Temporary variables have meaningful names
- [ ] Named intermediate variables instead of nested expressions

**Backend-specific readability:**
- [ ] Django queries are readable (prefer multiple simple queries over complex joins)
- [ ] Serializer fields have sensible names matching API contract
- [ ] Model methods are self-explanatory
- [ ] Admin configuration is straightforward

**Frontend-specific readability:**
- [ ] Template code is minimal (logic in script, not template)
- [ ] Computed properties for derived state
- [ ] Methods for complex operations
- [ ] Props and emits are well-typed
- [ ] Component composition is clear

**Comments:**
- [ ] Code is self-documenting (names explain intent)
- [ ] Comments explain *why*, not *what*
- [ ] No commented-out code
- [ ] No comments that restate the code

---

### 5. Check for over-engineering

**Backend over-engineering red flags:**
- 🚩 Custom base classes beyond `ProjectBaseAPIView`
- 🚩 Complex mixins that obscure behavior
- 🚩 Generic solutions for specific problems
- 🚩 Layers of abstraction without clear benefit
- 🚩 Custom serializer fields when DRF built-ins work
- 🚩 Meta-programming or dynamic model generation

**Frontend over-engineering red flags:**
- 🚩 Custom composables that just wrap one library call
- 🚩 Complex state machines when simple booleans work
- 🚩 Generic components with 10+ props/slots
- 🚩 Abstraction layers over PrimeVue components
- 🚩 Custom build configs beyond Vite defaults

**Abstraction checklist:**
- [ ] No abstraction without 3+ real use cases
- [ ] No "framework" code that hides simple operations
- [ ] No generic solutions to specific problems
- [ ] No indirection without clear benefit

**Configuration:**
- [ ] Configuration exists only for real variation
- [ ] No flags that are always one value
- [ ] No "just in case" extensibility hooks

---

### 6. Load IQ-specific checks

**Backend: DRF Standardized Errors:**
- [ ] `drf-standardized-errors>=0.15.0` in dependencies
- [ ] Exception handler configured in `settings/base.py`
- [ ] Views don't manually construct error responses
- [ ] All errors return `{type, errors[]}` structure
- [ ] Tests verify error response format

**Backend: Permission patterns:**
- [ ] All protected views use `check_has_permission()`
- [ ] Permissions checked before processing
- [ ] Permission names follow Django conventions
- [ ] No hard-coded permission checks

**Backend: Query patterns:**
- [ ] Simple conversions: `int(request.query_params.get('param'))`
- [ ] No excessive try-except for internal APIs
- [ ] Optional params checked with `if 'param' in request.query_params:`
- [ ] Comma-separated lists: `[int(x.strip()) for x in param.split(',') if x.strip()]`

**Frontend: Error handling:**
- [ ] TypeScript types for standardized errors exist
- [ ] Error utilities handle `{type, errors[]}` format
- [ ] Forms use `useSchemaValidation` + `useFormErrors` composables
- [ ] Client-side validation (Zod) AND server-side errors both displayed
- [ ] Field errors shown inline on form fields (both client and API errors)
- [ ] Form-level errors shown in Message banner
- [ ] Errors clear appropriately (validation on input, API errors on resubmit)

**Frontend: Form validation patterns:**
- [ ] Forms use `useSchemaValidation<T>(schema, initialValues)` for client validation
- [ ] Required `initialValues` parameter provided (not optional)
- [ ] Forms use `useFormErrors(setErrors)` for API error integration
- [ ] Template shows both error types: `validationErrors.field || apiErrors.getFieldError('field')`
- [ ] Cascading selects use `resetField()` not direct assignment
- [ ] Self-contained dialogs handle own API calls and error display
- [ ] Dialogs emit `@success` only, not complex callback props

**Frontend: Store patterns:**
- [ ] Pinia composition API style
- [ ] Loading/error state management
- [ ] API errors extracted and set (not hard-coded)
- [ ] Direct store access (no destructuring)

**Frontend: Component patterns:**
- [ ] PrimeVue components used consistently
- [ ] Tailwind for styling
- [ ] Props typed with TypeScript
- [ ] Emits declared explicitly
- [ ] Large UI blocks extracted to subcomponents
- [ ] PrimeVue Dialogs set `:draggable="false"` to prevent positioning issues
- [ ] Dialog content wrapped with `pt-4` padding to prevent header overlap

**Frontend: Composable patterns:**
- [ ] Form validation uses `useSchemaValidation` + `useFormErrors`
- [ ] Composables are focused (single responsibility)
- [ ] Composables return focused object (not exposing internal details)
- [ ] No composables that just wrap a single library call without added value

**Data flow:**
- [ ] Backend: Request → Permission → Validation → Processing → Response
- [ ] Frontend: User Action → API Call → Store Update → Component Reactivity
- [ ] Errors: Backend Standardized → Frontend Parse → Display on Field/Toast

---

### 7. Look for common problems

**Coupling:**
- [ ] Backend: Apps don't import from each other's views/serializers
- [ ] Frontend: Features don't directly import from other features
- [ ] Shared code lives in proper shared locations (`core`, `components/ui`)
- [ ] No circular dependencies
- [ ] Easy to test pieces in isolation

**Duplication:**
- [ ] No copy-paste code (especially validation/security)
- [ ] Not prematurely abstracted (3+ uses rule)
- [ ] Similar ≠ duplicate (don't force unification)
- [ ] Backend: No identical view structures without base class
- [ ] Frontend: No identical form patterns without composable

**Error handling:**
- [ ] Backend: Errors handled or explicitly raised
- [ ] Frontend: Errors shown to users, not just console
- [ ] No silent failures
- [ ] Error messages are user-friendly
- [ ] Field-level errors show on correct fields

**State management:**
- [ ] Backend: Clear ownership of model instances
- [ ] Frontend: Pinia stores for shared state
- [ ] Local component state for UI-only state
- [ ] No prop drilling >2 levels
- [ ] Mutations are explicit

**Testing gaps:**
- [ ] Backend: Permission tests (positive and negative)
- [ ] Backend: Serializer validation tests
- [ ] Backend: View integration tests
- [ ] Frontend: Would benefit from component tests
- [ ] Frontend: Error handling scenarios tested

---

### 8. Verification
- [ ] Backend: `task backend:lint` passes (or `ruff check`)
- [ ] Backend: `task backend:test` passes (or `pytest`)
- [ ] Frontend: `task frontend:lint` passes
- [ ] Frontend: `task frontend:typecheck` passes
- [ ] Can run or demo the change
- [ ] If verification can't be run: document what should be checked and why

---

## Output format

### Repo map (2 minutes)
**Backend:**
- Top-level apps and their responsibilities
- Key entry points (URLs, main views)
- Data flow patterns

**Frontend:**
- Top-level features and their purpose
- Key routes and views
- State management (stores)
- Component organization

### Summary (3-5 lines)
- What was audited (full codebase / PR / feature)
- Overall assessment: simpler / same / more complex
- Backend and frontend maturity
- Top concern (if any)

### Findings
Group by:
- **Structure & Organization**
- **API Design**
- **Readability**
- **Over-engineering**
- **Duplication**
- **Load IQ Patterns** (DRF errors, permission checks, store patterns, etc.)
- **Correctness**

For each:
- **Severity**: blocker / important / polish
- **Location**: file:line or pattern
- **Issue**: what's wrong and why it matters
- **Fix**: simplest improvement

### Action plan
- 1-3 quick improvements (< 30 min each)
- 1-2 medium refactors (< 2 hours)
- Optional: larger simplification (planned work)

---

## Decision heuristics

### Make changes that:
✅ Reduce cognitive load  
✅ Make intent obvious  
✅ Follow existing Load IQ patterns  
✅ Reduce coupling  
✅ Eliminate duplication  
✅ Simplify control flow  
✅ Use framework features correctly (DRF, Vue, PrimeVue)  
✅ Improve error handling UX

### Avoid changes that:
❌ Add abstraction speculatively  
❌ Introduce indirection without benefit  
❌ Create layers "for organization"  
❌ Deviate from established patterns  
❌ Force consistency over clarity  
❌ Optimize for hypothetical futures  
❌ Break working error flows

---

## Red flags

### 🚩 Backend red flags
- Views not inheriting from `ProjectBaseAPIView`
- Manual error response construction (not using DRF standardized errors)
- No permission checks on protected endpoints
- Complex nested serializers when flat would work
- Business logic in serializers instead of models/services
- Apps with unclear purpose (`utils`, `helpers`, `common`)
- Files >1000 lines mixing views/models/logic
- Copy-pasted permission/validation logic

### 🚩 Frontend red flags
- Direct API calls without going through `apiClient`
- Hard-coded error messages instead of extracting from API
- Forms without client-side AND server-side error handling
- Forms without field-level error display
- Forms using direct assignment for cascading selects (should use `resetField()`)
- Dialogs using callback props instead of self-contained pattern
- Stores without loading/error state
- Components >500 lines without clear sections
- Features importing from other features directly
- Props/emits without TypeScript types
- Business logic in components instead of stores/composables
- Missing `useSchemaValidation` + `useFormErrors` pattern in complex forms

### 🚩 Architecture red flags
- Backend apps importing views/serializers from other apps
- Frontend features tightly coupled to other features
- Circular dependencies anywhere
- Mixed concerns in single files (UI + data + logic)
- No clear data flow (hard to trace request to response)
- Inconsistent error handling across similar features
- Security/permission checks scattered instead of centralized

### 🚩 Organization red flags
- Can't explain structure in 60 seconds
- Can't predict where to find functionality
- Similar features in different locations
- Need to modify 5+ files for simple changes
- Folders/files with unclear purpose
- Deep nesting (5+ levels) making navigation painful

---

## Load IQ Architecture Decisions

**Document known architectural decisions discovered during audits here.**

### Backend
- **DRF standardized errors is configured**: All error responses follow `{type, errors[]}` format
- **ProjectBaseAPIView is the base**: All views inherit from it for permission/field validation
- **Input/output serializers split**: When write and read representations differ
- **Django app organization**: One app per domain (metadata, mitty, load_builder, etc.)

### Frontend
- **Vue 3 Composition API**: All new components use `<script setup lang="ts">`
- **Pinia for state**: Feature stores in `stores/`, no Vuex
- **PrimeVue for UI**: Consistent component library, Tailwind for styling
- **Feature-based organization**: `features/{feature}/components/` and `features/{feature}/views/`
- **Direct store access**: No destructuring for reactivity preservation
- **Form validation pattern**: Use `useSchemaValidation` (client) + `useFormErrors` (server) composables
  - Dual validation: Show both Zod validation errors AND API field errors
  - Required `initialValues` for proper type inference
  - Use `resetField()` for cascading selects, not direct assignment
- **Self-contained dialogs**: Dialogs handle own API calls, errors, toasts; emit only `@success`
- **Dialog styling**: Set `:draggable="false"` and use `pt-4` padding to prevent header overlap

### Integration
- **API client handles conversion**: Automatic camelCase (frontend) ↔ snake_case (backend)
- **CSRF in interceptor**: Automatic CSRF token injection for mutating requests
- **Standardized error parsing**: Frontend utilities parse backend standardized errors
- **Auth errors trigger events**: 401/403 dispatch `auth:required` custom event

---

## Living document

This skill is **intentionally iterative**.

If during a review you discover:
- Important architectural context
- Repeated patterns worth enforcing
- Common failure modes specific to Load IQ
- Django/Vue/PrimeVue best practices
- Better heuristics or checks

**Update this skill.**

Document:
- What the pattern is
- Why it exists (architectural decision)
- When to use/avoid it
- Examples from the codebase

Treat this as a living document that evolves alongside the codebase to reflect what "clean, readable, and simple" actually means *for Load IQ*.

---

## Audit Verification Checklist

Before submitting findings, verify your audit was thorough:

**Completeness:**
- [ ] Did you read or at least skim every file in scope?
- [ ] Did you check the largest files (top 10 by line count)?
- [ ] Did you explicitly search for duplicated code patterns?
- [ ] Did you verify findings with actual line numbers from file reads?
- [ ] Did you check both backend AND frontend for full audits?
- [ ] If you skipped files, did you document what wasn't reviewed and why?

**Quality:**
- [ ] Are severity ratings (blocker/important/polish) justified?
- [ ] Did you provide concrete file:line locations for each finding?
- [ ] Did you suggest specific fixes, not just identify problems?
- [ ] Did you distinguish between "needs fixing" vs "alternative approach"?
- [ ] Did you verify patterns match existing AGENTS.md guidelines?

**Load IQ-specific:**
- [ ] Did you check DRF standardized errors usage?
- [ ] Did you verify permission check patterns?
- [ ] Did you audit frontend error handling?
- [ ] Did you check store patterns follow Pinia composition API?
- [ ] Did you verify TypeScript types exist and are used?

**Honesty:**
- [ ] Did you acknowledge blind spots or incomplete coverage?
- [ ] Did you admit if you did structural audit vs deep line-by-line?
- [ ] Did you tell the user if you missed files on first pass?

**If any checklist item is unchecked**, either complete that step or explicitly state the limitation in your findings.

---

## Example usage
- "Audit the load builder feature for cleanliness"
- "Review this PR for architectural issues"
- "Are we over-engineering the error handling?"
- "Does this refactor improve the codebase?"
- "Is the new feature following Load IQ patterns?"
- "Audit before we ship this feature"
- "Check if backend errors are standardized"