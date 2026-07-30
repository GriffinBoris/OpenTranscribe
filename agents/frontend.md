## Frontend Guidelines (Vue 3 / TypeScript)

This file covers Vue 3, TypeScript, Pinia, frontend API patterns, and UI/UX conventions. Read `agents/AGENTS.md` first for cross-stack principles.

---

### Frontend Project Structure

- `src/assets`: Static assets such as images and fonts.
- `src/components`: Reusable UI components.
- `src/composables`: Shared Vue composables.
- `src/features`: Feature-specific views, logic, and child components.
- `src/router`: Vue Router configuration.
- `src/stores`: Pinia stores.
- `src/types`: Global shared types.
- `src/core`: Core functionality including models and utilities.
- `src/styles`: Global styles and CSS utilities.
- Keep shared composables under `src/composables`, transport/domain interfaces under `src/types`, and page-level components under `src/components/page`. Route views should use dynamic imports so production builds remain split by feature.

---

### Frontend API

#### Single API Client Rule

Every project must have **exactly one** canonical API client. Do not maintain parallel implementations (e.g., both `apiService.ts` and `useApiClient.ts` doing the same thing). If you find duplicates, consolidate to the one that best matches the project's patterns.

The canonical client should:
- Centralize Axios configuration (base URL, interceptors, CSRF handling).
- Handle automatic camelCase <-> snake_case conversion.
- Provide typed response wrappers.
- Be the **only** place where `axios` is imported outside of tests.

#### API Client Patterns

- Every request flows through the `ApiClient` composable/service for Axios access, CSRF handling, camelCase <-> snake_case conversion, and typed responses.
- Backend responses should be **snake_case**; the ApiClient will convert them to camelCase on receipt.
- Group API calls by domain and use consistent method names (`list`, `create`, `detail`, `update`, `delete`, plus specialized verbs like `duplicate`).
- Type responses and payloads with the appropriate interfaces or `Partial<Interface>` for updates.
- Follow RESTful URL structures; include IDs in the path and use nested paths for related resources.
- Use `FormData` for uploads and customize headers only when required.
- Use `buildParamsConfig` with camelCase params; do not pass snake_case keys from components/stores (ApiClient handles conversion).
- Define API domain modules as top-level `const` blocks and export them via a unified `api` object.
- Order method parameters from most important to least important.

---

### Type Discipline

- **No `any` or `unknown` at API transport boundaries.** Every API call must have typed request payloads and typed response interfaces. If the backend shape is uncertain, define the interface and mark optional fields with `?` -- do not use `any`.
- **Fix return type mismatches.** If a method declares `Promise<ModelInterface>` as its return type, it must not actually return `AxiosResponse<ModelInterface>`. Unwrap the response inside the API client so call sites receive the model directly.
- **No mixed `AxiosResponse` and model types.** Pick one convention (preferably unwrapped models) and use it consistently across all API domain modules.
- **Separate form DTOs from persisted entity models.** Do not reuse a persisted entity interface (with server-assigned `id`, `createdTs`, etc.) as the form state for creating new records. Instead, create a dedicated input/form type:

```typescript
// Entity model (from server)
interface Alert {
    id: number;
    name: string;
    threshold: number;
    createdTs: string;
}

// Form DTO (for create/edit)
interface AlertInput {
    name: string;
    threshold: number;
}
```

Do not use placeholder IDs (`0`, `-1`) in form state to satisfy an entity interface.

---

### Frontend Models

- Organize models by domain under `src/core/models` or `src/types` (for example, `load`, `customer`, `alert`).
- Keep related models together inside the same domain folder.
- Store-related interfaces (for example calendar day view models) must also live under `src/core/models/<domain>` and be imported where needed instead of being defined inside stores or views.

#### Naming Conventions (when not deprecated)

- Model interfaces should follow the pattern `[ModelName]Interface.ts`.
- Input models should follow the pattern `[ModelName]InputInterface.ts`.
- Request models should follow the pattern `[Action]RequestInterface.ts`.
- Specialized action models should follow the pattern `[ModelName][Action]InputInterface.ts`.
- Enum files should follow the pattern `[ModelName]Enums.ts` or be included in the main interface file.

#### Interface Structure (when not deprecated)

- Interfaces should use the `interface` keyword.
- Properties should use camelCase naming.
- Required properties should be declared without the optional operator.
- Optional properties should be marked with the `?` operator.
- Include appropriate TypeScript types for all properties.
- For consistency with backend, include standard fields like `id`, `createdTs`, `updatedTs` when applicable.

#### Enums (when not deprecated)

- Enum names should be PascalCase.
- Enum values should be UPPERCASE.
- String enum values should match backend values.

#### Factory Functions (when not deprecated)

- Factory functions should follow the pattern `createDefault[ModelName](): [ModelName]Interface`.
- Factory functions should provide sensible defaults for required fields.

#### Deprecated Conventions

- Some repos mark the above naming and factory conventions as deprecated. If a repository marks these as deprecated, do not reintroduce them until guidance changes.

---

### Stores (Frontend State Management)

- Keep Pinia state minimal; derive view data with computed properties.
- On data changes, trigger reloads (`loadX`) instead of manually clearing state.
- Use stable, deterministic keys for UI rows and lookup maps.
    - If a domain object is only unique by multiple fields, key by a composite (example: `${productionDatabase}::${schemaName}`), not by a display name.
    - Do not implement fallback matching based on non-unique attributes.
- Access Pinia store state, getters, and actions directly via the store object (e.g., `customStore.field`, `customStore.loadAllData()`) instead of destructuring, so reactivity and intent stay explicit.

---

### Frontend Coding Style & UI/UX Layout Consistency

- Place generic controls in `src/components/ui/`, page-level states and tables in `src/components/page/`, and authenticated/guest framing in `src/components/layout/`. Avoid recreating those responsibilities inline in views.
- Reuse existing components, tokens, and layout patterns before creating new ones.
- Keep component filenames PascalCase and composables prefixed with `use`.
- Favor shared SCSS/utility classes instead of inline styles unless values are dynamic.
- Prefer Tailwind flex utilities for layout before reaching for grid.
- Follow established form layout, validation messaging, and table column ordering conventions.
- Dense comparison tables and matrices must provide a mobile card/list presentation on small screens; horizontal scrolling alone is only an acceptable desktop overflow fallback.
- Keep modal structure consistent (header, body, footer) with accessibility attributes (`role="dialog"`, `aria-label`).
- Manage state through computed properties rather than duplicating derived data.
- Avoid introducing new global styles; place shared styles with existing peers.
- Reuse confirm dialogs for destructive actions, mirror accessibility attributes, and respect existing responsive breakpoints.
- Add tests for complex components using the same patterns (unit, shallow mount, snapshot) already in use.
- Use shared loading UI before introducing new spinners or progress banners.
- When a copy interaction is introduced, add or reuse a shared clipboard composable instead of calling `navigator.clipboard` from a view.
- Favor extracting sizable UI blocks into dedicated subcomponents to keep pages lean, improve readability, and maximize reuse across views.
- For large dossier/report-style pages, extract each major section into feature-local subcomponents before adding more content so responsive behavior and visual tweaks stay localized.
- Align error/retry UI with shared components: show the shared error message component, pair it with a warn-toned retry button, and keep spinner blocks minimal and centered.
- Prefer shared UI inputs and keep spacing consistent with existing utility classes.
- Theme controls should live in the app shell, and theme classes should be applied before the Vue app mounts so dark mode does not flash the wrong palette on first paint.
- Only expose theme options that have real CSS token support; do not keep enum or storage values for unfinished themes.

#### camelCase / snake_case Discipline

- **Frontend code uses camelCase everywhere**: variable names, function names, interface properties, component props.
- **Backend API responses are snake_case**; the API client converts automatically.
- **Never pass snake_case keys** from components or stores to the API client.
- **Never manually convert casing** in components -- trust the API client interceptors.

---

### No Mixed Component Paradigms

Do not mix Options API and Composition API (`script setup`) in the same file. Pick one paradigm per component:

- **New code**: Always use `<script setup lang="ts">` (Composition API).
- **Legacy code** (legacy JS app): If an existing component uses Options API, keep it consistent within that file. Do not add `script setup` blocks alongside `export default`.
- **Migration**: When significantly refactoring a legacy component, convert it fully to Composition API.

---

### Frontend Form Validation & Error Handling

**Forms:**
- Reuse `FormField` with the relevant `App*` input wrapper and map standardized DRF errors through `utils/errorHandling.ts`.
- Add a client-side schema-validation dependency only when a feature needs validation beyond the backend contract; do not retain VeeValidate/Yup or a form composable without a consumer.

**Dual validation pattern (client + server):**
```typescript
const errors = reactive<Record<string, string>>({});

// In template: show both error types
:has-error="!!errors.field"
<small>{{ errors.field }}</small>
```

**Cascading selects (dependent dropdowns):**
- Clear dependent fields directly from the owning typed form state; do not add a form-state library solely for a simple reset.
```typescript
watch(() => form.retailer, () => {
    form.retailerCountryId = null;
    form.sourceSystemId = null;
});
```

**Self-contained dialog pattern:**
- Dialogs should handle their own API calls, error handling, and success toasts.
- Emit only an `@success` event with minimal data for parent navigation.
- Keep the parent simple: open/close the dialog and handle post-success navigation.

---

### Frontend View Patterns

- Keep derived collections (`filteredRows`, grouped data) inside `computed` getters; never stash them in refs that need manual syncing.
- Use `usePolling` for auto-refresh behavior instead of manual `setInterval`/`setTimeout` handling in views.
- Guard optional identifiers (for example, `calendarId`) before use and return early when dependencies are missing to avoid runtime `undefined` checks in templates.
- Use small helper functions for repeated lookups (such as retailer/country names) rather than inlining ternaries in templates; this keeps markup clean and readable.
- Build view models with explicit loop-based transformers instead of chaining `map`/`filter`/`reduce` calls; it keeps the happy path obvious and simplifies debugging.
- Keep data-fetch helpers focused on a single resource (`fetchRetailers`, `fetchCountries`) and call them sequentially when later requests rely on earlier metadata.
- Use `usePolling` for auto-refresh behavior instead of manual `setInterval`/`setTimeout` handling in views.

---

### Frontend Legacy Anti-Patterns (Do Not Reintroduce)

- Do not perform heavy lookups, ternaries, or formatting inline in the template -- move them into helpers or computed getters so the markup stays readable.
- Skip ad-hoc boolean flags sprinkled across the template; derive `isLoading`, `hasError`, and filtered views from a single source of truth in script setup.
- **Do not use `.value` on nested refs/computeds from composables in templates**. Vue 3 only auto-unwraps top-level refs returned from `setup`. If a composable returns a plain object containing `ComputedRef` or `Ref` properties, accessing `.value` in the template is unreliable for reactivity tracking. Instead, create a top-level `computed` in the component that unwraps it:
    ```typescript
    // Unreliable -- nested ComputedRef, reactivity may not track
    :pinned-customers="favorites.pinnedCustomers.value"

    // Correct -- top-level computed is properly tracked
    const pinnedCustomersList = computed(() => favorites.pinnedCustomers.value);
    // then in template:
    :pinned-customers="pinnedCustomersList"
    ```

### Legacy Tolerance (legacy JS app)

This legacy project is a JavaScript codebase without TypeScript. When working in that codebase:
- Accept JavaScript -- do not add TypeScript to individual files.
- Accept Vuex -- do not partially migrate to Pinia within the legacy app.
- Accept legacy-prefixed component library (LegacyButton, LegacyComboBox, etc.).
- Do not reintroduce these patterns in modern projects.
- If significantly refactoring a legacy component, consider a full migration to modern patterns.

---

### Frontend Code Reference Workflow

1. Identify an existing feature similar to the one you are building.
2. Inspect its components, stores, and API usage.
3. List reusable pieces (components, composables, styles) before coding.
4. Implement the new feature by reusing the gathered pieces first; add new code only when nothing reusable exists.
5. Run the linter/formatter to ensure the new code matches project style.

---

## Consistency Checklist (Frontend)

- Component name and file casing are correct (PascalCase files, camelCase variables).
- Existing UI components (forms, tables, buttons) are reused.
- Spacing and styling align with comparable features.
- API calls go through the single canonical `ApiClient` with camelCase <-> snake_case conversion.
- No `any` or `unknown` at API boundaries.
- No mixed Options API / Composition API in the same file.
- Form state uses dedicated input types, not entity models with placeholder IDs.
- Return types match actual return values (no `AxiosResponse` when model type is declared).
