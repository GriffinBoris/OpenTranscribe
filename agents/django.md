## Backend Guidelines (Django / DRF / Python)

This file covers Django, Django REST Framework, Celery, and Python backend patterns. Read `agents/AGENTS.md` first for cross-stack principles.

---

### Data Ownership & Tenant Scoping

Every authenticated endpoint must scope data to the current user or tenant. This is a security requirement, not a convenience.

- **List endpoints** must filter by ownership: `request.user`, `request.employee.company`, or the equivalent boundary. Never use `Model.objects.all()` in an authenticated list view.
- **Detail/update/delete endpoints** must verify the requested object belongs to the current user/tenant before operating on it.
- **Tests must verify ownership boundaries.** If endpoint is scoped to user A, a test must confirm user B cannot see user A's data.
- **Multi-tenant middleware** (for example, middleware scoping by `request.employee.company`) is a good pattern. When it exists, all queries should flow through it.

```python
# Correct -- scoped to current user
def get(self, request):
	alerts = Alert.objects.filter(user=request.user)
	return Response(AlertOutputSerializer(alerts, many=True).data)

# WRONG -- exposes all users' data
def get(self, request):
	alerts = Alert.objects.all()
	return Response(AlertOutputSerializer(alerts, many=True).data)
```

---

### Backend View Basics

- All views must inherit from `ProjectBaseAPIView` in `core/base_views.py` (or the project-specific base API view).
- Use fully qualified imports from the real module that defines a symbol. Do not add compatibility re-export shims like `core/common.py` just to preserve shorter imports.
- For object lookups, reuse existing patterns with `get_object_or_404`.
- Call `check_has_permission` for permission checks and `require_fields` for validating request payloads.
- For custom permissions (non-CRUD), build the string with `ProjectBaseModel.get_custom_permission('codename', app_label='core')` instead of indexing `_meta.permissions` or hand-building permission strings.
- **Query Parameter Handling**: trust internal API data and avoid excessive error handling.
  - Use simple conversion: `int(request.query_params.get('param_name'))` or `request.query_params.get('param_name')`.
  - Skip try-except blocks for parameter validation -- let exceptions bubble up naturally.
  - When you need a standardized 400 for invalid query params, use a DRF field converter (for example `BlankableIntegerField().to_internal_value(value)`) and let `ValidationError` bubble up.
  - For comma-separated lists: `[int(x.strip()) for x in request.query_params['param'].split(',') if x.strip()]`.
  - For defaults, prefer passing the default into `get` directly (for example `int(request.query_params.get('days', 7))`) rather than manual `None`/empty checks.
  - Backend must accept snake_case query params; do not add new camelCase params.
  - When parsing common query params (date ranges, CSV lists), reuse shared helpers instead of re-implementing.
  - Example pattern from existing codebase:
    ```python
    # Simple conversion without try-except
    page = int(request.query_params.get('page', 1))
    page_size = int(request.query_params.get('page_size', 10))

    # Optional parameter
    account_id = request.query_params.get('account_id')
    if account_id:
		queryset = queryset.filter(account_id=account_id)

    # Comma-separated list
    schedule_ids = None
    if 'schedule_ids' in request.query_params:
		schedule_ids = [int(x.strip()) for x in request.query_params['schedule_ids'].split(',') if x.strip()]
    ```
- Keep metadata builders straightforward: favor early returns, single-purpose helpers, and shallow loops so pagination/deduplication logic stays readable.

---

### API

- Only add views, serializers, and URLs inside the app you are updating.
- Mirror existing `urls.py` and `views.py` layouts for routing and logic decisions.
- Create tests for views and serializers in the app's `tests` package when behavior changes.
- Design API responses so the frontend can match rows deterministically.
  - Prefer composite identifiers when a "name" is not globally unique (example: `production_database + schema_name`).
  - Avoid backend logic that relies on matching tasks/rows by a non-unique attribute.
- **Backend API responses must use snake_case** so the frontend ApiClient can consistently convert them to camelCase.
- **Return created/updated resources from mutating endpoints.** POST should return the created object with HTTP 201. PUT should return the updated object. Do not return empty `{}` on success -- the frontend needs server-assigned values (IDs, timestamps, computed fields).
- **Enumerated options**: when responding with selectable options from Django `choices`, use a shared helper like `create_options_from_choices` if it exists in the base API view.

---

### Views

#### Permission Patterns

- Always verify context first (for example community membership) at the top of every action.
- Membership checks commonly pair with staff checks for staff-only actions.
- When a view should behave differently for owners vs staff, copy existing patterns:
  - Use property-scoped checks for property-owned resources.
  - Use explicit permission checks for cross-entity reads.
  - For owned resources, raise `PermissionDenied` when business rules fail.

#### Query Handling

- Favor queryset scoping over branching logic. Build a base queryset filtered by context and then apply optional filters using `request.query_params.get(...)`.
- Finish with `.order_by('id')` (or another deterministic field) and `.distinct()` when joins are involved.
- For serializers that need enforced context, wrap request data in an `edited_data` dict so callers cannot spoof protected fields.
- Validate query parameters for special actions and return `400 BAD REQUEST` if they are missing.

#### Example Filter + Permission Handling

```python
class ExampleListView(ProjectMemberAuthenticatedAPIView):
	def get(self, request, community_id: int):
		self.check_has_permission(ExampleModel.get_view_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)

		queryset = ExampleModel.objects.filter(community_id=community_id)

		search = request.query_params.get('search')
		if search:
			queryset = queryset.filter(name__icontains=search)

		if not request.member.check_is_community_staff():
			queryset = queryset.filter(owner=request.member)

		queryset = queryset.order_by('id').distinct()
		serializer = ExampleOutputSerializer(queryset, many=True)
		return Response(serializer.data, status=status.HTTP_200_OK)
```

#### Example Create/List/Detail Trio

```python
from django.shortcuts import get_object_or_404
from rest_framework import status
from rest_framework.response import Response

from core.base_views import ProjectMemberAuthenticatedAPIView
from example.models import ExampleModel
from example.serializers import (
	ExampleInputSerializer,
	ExampleOutputSerializer,
)


class ExampleListView(ProjectMemberAuthenticatedAPIView):
	def get(self, request, community_id: int):
		self.check_has_permission(ExampleModel.get_view_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)

		queryset = ExampleModel.objects.filter(community_id=community_id).order_by('id')
		serializer = ExampleOutputSerializer(queryset, many=True)
		return Response(serializer.data, status=status.HTTP_200_OK)


class ExampleCreateView(ProjectMemberAuthenticatedAPIView):
	def post(self, request, community_id: int):
		self.check_has_permission(ExampleModel.get_add_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)
		request.member.check_is_community_staff(raise_permission_error=True)

		edited_data = {'community': community_id}
		edited_data.update(request.data)

		serializer = ExampleInputSerializer(data=edited_data)
		serializer.is_valid(raise_exception=True)
		instance = serializer.save(community_id=community_id)
		return Response(ExampleOutputSerializer(instance).data, status=status.HTTP_201_CREATED)


class ExampleDetailView(ProjectMemberAuthenticatedAPIView):
	def get(self, request, community_id: int, pk: int):
		self.check_has_permission(ExampleModel.get_view_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)

		instance = get_object_or_404(ExampleModel, pk=pk, community_id=community_id)
		return Response(ExampleOutputSerializer(instance).data, status=status.HTTP_200_OK)

	def put(self, request, community_id: int, pk: int):
		self.check_has_permission(ExampleModel.get_change_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)
		request.member.check_is_community_staff(raise_permission_error=True)

		instance = get_object_or_404(ExampleModel, pk=pk, community_id=community_id)
		serializer = ExampleInputSerializer(instance, data=request.data, partial=True)
		serializer.is_valid(raise_exception=True)
		instance = serializer.save()
		return Response(ExampleOutputSerializer(instance).data, status=status.HTTP_200_OK)

	def delete(self, request, community_id: int, pk: int):
		self.check_has_permission(ExampleModel.get_delete_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)
		request.member.check_is_community_staff(raise_permission_error=True)

		instance = get_object_or_404(ExampleModel, pk=pk, community_id=community_id)
		instance.delete()
		return Response(status=status.HTTP_204_NO_CONTENT)
```

#### Example Action View

```python
from rest_framework import status
from rest_framework.response import Response
from rest_framework.exceptions import PermissionDenied


class ExampleActionView(ProjectMemberAuthenticatedAPIView):
	def get(self, request, community_id: int, pk: int):
		self.check_has_permission(ExampleModel.get_view_permission(include_app_name=True))
		request.member.check_has_community(community_id, raise_permission_error=True)
		instance = get_object_or_404(ExampleModel, pk=pk, community_id=community_id)

		property_id = request.query_params.get('property_id')
		member_id = request.query_params.get('member_id')
		if not property_id and not member_id:
			return Response({'error': 'Either property_id or member_id is required'}, status=status.HTTP_400_BAD_REQUEST)

		if property_id:
			request.member.check_has_property(property_id, raise_permission_error=True)
		elif member_id != request.member.id and not request.member.is_community_staff:
			raise PermissionDenied

		return Response({'result': instance.perform_action()}, status=status.HTTP_200_OK)
```

---

### Serializers

#### Input/Output Pattern

- Create two serializers for each model: one for inputs (`ModelInputSerializer`) and one for outputs (`ModelOutputSerializer`).
- Input serializers validate incoming data (POST, PUT).
- Output serializers format data returned to the client (GET).
- For simpler models or when input/output requirements are identical, a single serializer may be used.

#### Naming Conventions

- Input serializers: `ModelInputSerializer`.
- Output serializers: `ModelOutputSerializer`.
- Combined serializers: `ModelSerializer`.

#### Meta Class Structure

Every serializer must have a Meta class that defines:
1. The model being serialized: `model = Model`.
2. The fields to include: `fields = ('id', 'field1', 'field2', ...)`.
3. For output serializers: `read_only_fields = fields`.

#### Field Definitions

- Always include `id` as the first field in the fields tuple.
- **Verify field tuples for completeness.** A duplicate field (for example `address_one` listed twice) silently drops other fields. Review field tuples against the model to ensure every field appears exactly once.
- For longer field lists, use multi-line formatting with consistent indentation:
  ```python
  fields = (
	  'id',
	  'field1',
	  'field2',
	  'field3',
  )
  ```
- By default, all fields should be read-only unless they need to be writable.
- Use `read_only_fields = fields` in output serializers to make all fields read-only.

#### Related Objects

- When accessing related objects, use `source` to specify the attribute path.
- Example: `owner_name = serializers.CharField(source='owner.name', read_only=True)`.
- For nested serializers:
  ```python
  related_items = RelatedItemOutputSerializer(many=True, read_only=True)
  ```

#### Computed Fields

- For computed fields, use `SerializerMethodField` and implement the corresponding `get_field_name` method.
- Example:
  ```python
  full_name = serializers.SerializerMethodField()

  def get_full_name(self, obj):
	  return f"{obj.first_name} {obj.last_name}"
  ```

#### Validation

- For custom validation, implement `validate_field_name` methods or a `validate` method for cross-field validation.
- If an action/task depends on identifiers to correctly associate records, validate them as required fields.
- Example:
  ```python
  def validate_start_date(self, value):
	  if value < date.today():
		  raise serializers.ValidationError('Start date cannot be in the past.')
	  return value

  def validate(self, data):
	  if data['end_date'] < data['start_date']:
		  raise serializers.ValidationError('End date must be after start date.')
	  return data
  ```

#### Custom Create and Update Methods

- For input serializers, implement custom `create()` and `update()` methods when you need explicit control over instance creation or modification.
- Always return the instance at the end of these methods.
- Use `.get()` with defaults for optional fields.

#### Example Input/Output Serializer Pair

```python
from rest_framework import serializers

from example.models import ExampleModel


class ExampleInputSerializer(serializers.ModelSerializer):
	class Meta:
		model = ExampleModel
		fields = (
			'id',
			'community',
			'name',
			'status',
			'assigned_member',
		)
		read_only_fields = ('id',)


class ExampleOutputSerializer(serializers.ModelSerializer):
	status_display = serializers.CharField(source='get_status_display', read_only=True)
	community_name = serializers.CharField(source='community.name', read_only=True)
	assigned_member_name = serializers.SerializerMethodField()

	class Meta:
		model = ExampleModel
		fields = (
			'id',
			'community',
			'community_name',
			'name',
			'status',
			'status_display',
			'assigned_member',
			'assigned_member_name',
			'created_ts',
			'updated_ts',
		)
		read_only_fields = fields

	def get_assigned_member_name(self, obj):
		if obj.assigned_member and obj.assigned_member.user:
			user = obj.assigned_member.user
			return f'{user.first_name} {user.last_name}'.strip() or user.username
		return None
```

---

### Django Models

- Extend `ProjectBaseModel` from `core/base_models.py` for all models.
- Prefer `models.TextField` for new string fields unless a specific length constraint is required.
- Follow the strict ordering:
  1. Class definition (`class SomeModel(ProjectBaseModel):`).
  2. `class Meta`.
  3. Supporting inner classes (for example `TextChoices`).
  4. Field declarations with arguments in this order:
     1. For relations, pass the target model as `'app.Model'`.
     2. Field-specific arguments.
     3. `default` (optional).
     4. `null`.
     5. `blank`.
     6. `verbose_name` wrapped in `gettext`.
     7. For relations, set `on_delete` explicitly.
  5. Optional dunder (`__str__`, `__repr__`).
  6. Optional `save`, `delete`, then any other helpers.
- Keep field declarations single-line.
- Enable history logging when needed by defining `history_log_fields` or `history_log_private_fields` and pass `log_user_id` to `save` when tracking user changes.
- **Prefer `@staticmethod` over `@classmethod`** for model helper methods.

#### `on_delete` Policy

The existing codebase uses `on_delete=models.DO_NOTHING` universally. This is the current convention and should be followed for consistency within existing projects. Be aware of the trade-offs:

- `DO_NOTHING` risks orphaned records and referential integrity issues.
- When adding new models or relationships in a greenfield context, consider `CASCADE`, `SET_NULL`, or `PROTECT` based on the actual domain relationship.
- Any change from `DO_NOTHING` to another strategy in existing models requires careful migration planning.

#### Model Lifecycle Side Effects

Do not hide I/O in model `save()` or `delete()`. Third-party API calls, email sends, webhook notifications, and other network I/O must not live inside model lifecycle methods.

- Keep `save()` and `delete()` limited to database-level concerns (field defaults, validation, audit logging).
- Put third-party I/O in explicit service functions or Celery tasks that the view/command calls directly.
- If the current codebase has I/O in `save()`/`delete()`, do not add more. When modifying those methods, consider extracting the I/O to a service layer.

---

### God Module Prevention (`core/common.py`)

Every Django project in the portfolio has an overstuffed `core/common.py` mixing auth backends, encryption, email utilities, admin widgets, test base classes, and base models. This is the most common structural problem across all repos.

When modifying `core/common.py`:
- Do not add new unrelated concerns.
- If you're touching a specific concern (for example email), extract it to a dedicated module (`core/email.py`, `core/encryption.py`, `core/auth_backends.py`) and update imports to reference that module directly.
- Do not keep or add compatibility re-export shims once direct imports are practical; use the fully qualified module path instead.
- New projects should start with separated modules rather than a single `common.py`.

Target structure for `core/`:
```
core/
	base_models.py       # ProjectBaseModel, audit logging
	base_views.py        # ProjectBaseAPIView, permission helpers
	auth_backends.py     # Authentication backends
	encryption.py        # Fernet encryption utilities
	email.py             # Email sending utilities
	admin_widgets.py     # Custom admin widgets
	test_fixtures.py     # Test fixture helpers
```

---

### URL Patterns

- Each app needs its own `urls.py` with `app_name` defined.
- Import `path` from `django.urls` and the app's views.
- Follow project REST conventions:
  - List: `path('list/', views.ModelListView.as_view(), name='model-list')`
  - Create: `path('create/', views.ModelCreateView.as_view(), name='model-create')`
  - Detail: `path('<int:pk>/', views.ModelDetailView.as_view(), name='model-detail')`
  - Nested resources include parent identifiers (for example, `path('<int:parent_id>/related/', ...)`).
- Name routes with kebab-case `{model-name}-{action}`.

---

### Admin Configuration

- Register models with `@admin.register(Model)` and subclass `admin.ModelAdmin`.
- Always include `id`, `created_ts`, and `updated_ts` in `list_display` and `readonly_fields`.
- Use `search_fields`, `list_filter`, and `raw_id_fields` where relevant to mirror existing admin behavior.
- Custom actions belong in the `actions` tuple and should provide meaningful `short_description` text.
- Use multi-line tuples when listing many fields for clarity.

---

### Background Tasks (Celery)

- Use Celery for background operations that do not need to complete synchronously.
- Never run cleanup, purge, or third-party I/O operations in `save()`, `delete()`, or view methods.

Pattern for creating background tasks:
1. Add a method to the `Task` class in the task app:
   ```python
   def purge_expired_records(self):
	   count = Model.objects.filter(timestamp__lt=cutoff).delete()[0]
	   self.set_message_and_percent(f'Purged {count} records', 100)
   ```
2. Register in `task_map` dictionary.
3. Create a Celery task that calls `Task.create_task('task_name')`.
4. Schedule in `setup_periodic_tasks()`.
5. Add tests in the task app's test directory.

- Use `self.set_message_and_percent(message, percent)` in task methods to report progress and status.
- Periodic tasks are configured using cron syntax (for example `crontab(minute='*/30')`).

---

### Settings Hierarchy

Follow the established settings inheritance chain:

```
base.py -> dev_local.py -> dev_pytest.py -> dev_docker.py -> dev_github.py -> production.py
```

- `base.py`: Shared configuration, reads from environment variables for secrets.
- `dev_local.py`: Local development overrides.
- `dev_pytest.py`: Test-specific settings (fast password hasher, in-memory cache, etc.).
- `production.py`: Production settings, must not hardcode secrets. Use `os.environ` or a secrets manager.
- No wildcard `ALLOWED_HOSTS` in production settings.
- No hardcoded `SECRET_KEY` in any committed settings file.
- Wildcard imports (`from core.settings.settings_base import *`) are acceptable in settings files only.

---

### Security Checklist (Django-Specific)

- [ ] Every view has explicit authentication and permission classes (no empty lists)
- [ ] List/detail endpoints scope data to current user/tenant
- [ ] No secrets hardcoded in settings files
- [ ] `ALLOWED_HOSTS` is not `('*',)` in production
- [ ] CSRF protection is not disabled without explicit justification
- [ ] Admin tools are gated behind `DEBUG=True` or admin-only access
- [ ] `.env.secret` files are in `.gitignore`
- [ ] Encrypted fields used for sensitive data (payment info, API tokens, credentials)

---

### Backend Coding Style & Consistency

- Review existing apps before adding new patterns; prefer imitation over invention.
- **Import organization**:
  - Order imports: standard library, third-party, Django, project-local.
  - Place all imports at the top of the file.
  - Avoid inline imports inside functions (prevents circular imports and improves readability).
  - Exception: only use inline imports when absolutely necessary to break circular dependencies.
  - Avoid wildcard imports (`from module import *`).
- **Logical spacing**:
  - Add blank lines between major code sections (imports, constants, classes, function groups).
  - Use one blank line between methods, two blank lines between classes.
- Stick to single quotes unless triple quotes are required.
- Keep functions small; extract helpers only when reused or improving clarity.
- Use explicit names (`instance`, `queryset`) already established in the codebase.
- Avoid placing logic in `__init__.py` files. Keep them minimal -- only use them for explicit exports (`__all__`) when absolutely necessary for clean imports.
- Standardize error responses:
  - This project uses drf-standardized-errors.
  - Prefer raising DRF exceptions (for example `rest_framework.exceptions.ValidationError`) and assert responses using the standardized shape:
    `{ "type": "validation_error"|..., "errors": [{"code": str, "detail": str, "attr": str|null}] }`.
  - In views, use `rest_framework.exceptions.ValidationError` (not `serializers.ValidationError`) so standardized errors are produced consistently.
  - In serializers, continue using `serializers.ValidationError` for field-level validation.
  - Only return `{'detail': ...}` payloads if you have a strong reason and it matches surrounding code.
- Reuse permission checks and attribute ordering (`constants`, `queryset`, `serializer_class`, `permission_classes`, methods).
- Catch specific exceptions; let unexpected errors surface.
- Avoid dynamic `getattr` and `setattr` unless absolutely necessary.
- Compare new serializers/views against similar existing ones to ensure structural parity.
- Do not use `from __future__ import annotations` for type hints. Use `typing.Optional` and `typing.Union` instead of the `|` union syntax.
- Use builtin literal types (for example `list`, `dict`, `tuple`) instead of `typing.List`, `typing.Dict`.
- Avoid regex when an exact match works.
- Avoid type-only casts. Prefer focused `# type: ignore[...]` when clarity is more important.
- Prefer using `settings.<VAR>` directly in views/helpers for configuration values instead of reassigning to local module constants.

#### Imports

- Prefer explicit module paths for project types.
- Avoid re-exported paths when a direct module path is available.
- Use fully qualified imports for project modules.
- Keep `__init__.py` files minimal: only declare modules or explicit exports, no code.

---

### Ruff Configuration

The shared ruff config lives in the Common repo (`ruff-pyproject.toml`). Key settings:

- Line length is currently set too high in some repos; prefer 120 for new projects.
- Always run `ruff check` on modified Python files before completing a task.

---

### AI Agent Patterns (PydanticAI)

- Prefer `message_history=` over embedding prior messages into a giant prompt string.
  - If `message_history` is provided and non-empty, PydanticAI will not generate a new system prompt. Ensure your history includes a `SystemPromptPart`.
  - Keep stored history as a simple list of `{role, content}` (Slack-style) if needed, and convert to `ModelRequest`/`ModelResponse` at runtime.
- Inject summaries of active skills and user memories into the system prompt so the model knows what exists without extra tool calls.
- Provide read-only tools to fetch skill/memory details when needed.
- Use `UsageLimits` (at least `tool_calls_limit`) to prevent runaway tool loops.
- If an agent calls another agent inside a tool (delegation), pass `ctx.usage` into the delegate `agent.run(..., usage=ctx.usage)` so totals aggregate.

---

## Testing Guidelines

- Add reusable object builders to `core/test_fixtures.py` and reference them from tests instead of writing ad-hoc helper functions inside test modules.
- Keep fixture helpers explicit: accept named parameters with sensible defaults rather than generic `**kwargs`.
- Place tests alongside their feature modules (for example `views/<feature>/tests/`), and keep model tests in the app's `tests` directory.
- Tests should explicitly control permission assignments rather than relying on defaults so the expectation is deterministic.

### Serializer Tests

- Create a pytest class per serializer and use `setup_method` to prepare fixtures that will be reused across tests.
- Input serializer tests must cover the happy path, missing required fields (use `pytest.mark.parametrize` so each omission is asserted individually), and any domain-specific validation. When serializers override `create` or `update`, add explicit tests that exercise those code paths and confirm persisted state.
- Output serializer tests should assert the exact field set, verify expected values, and inspect nested objects. Copy `serializer.data`, `pop` each expected key while asserting its value, and assert the remainder is empty so unexpected keys are caught.
- When output serializers depend on lookups or query helpers, monkeypatch those helpers to return deterministic stub objects so tests do not touch external systems.

#### Example Serializer Test Pattern

```python
@pytest.mark.django_db
class TestExampleSerializer:
	def setup_method(self):
		self.community = TestFixtures.create_community()
		self.member = TestFixtures.create_member()
		self.valid_data = {
			'community': self.community.id,
			'name': 'Example',
			'status': ExampleModel.Status.PENDING,
			'assigned_member': self.member.id,
		}
		self.instance = TestFixtures.create_example(community=self.community)

	def test_input_serializer_valid(self):
		serializer = ExampleInputSerializer(data=self.valid_data)
		assert serializer.is_valid(), serializer.errors
		instance = serializer.save()
		assert instance.name == self.valid_data['name']

	@pytest.mark.parametrize('missing_field', ['community', 'name', 'status'])
	def test_input_serializer_missing_required_fields(self, missing_field):
		data = self.valid_data.copy()
		data.pop(missing_field)
		serializer = ExampleInputSerializer(data=data)
		assert not serializer.is_valid()
		assert missing_field in serializer.errors

	def test_input_serializer_update(self):
		update_data = self.valid_data | {'status': ExampleModel.Status.APPROVED}
		serializer = ExampleInputSerializer(self.instance, data=update_data)
		assert serializer.is_valid(), serializer.errors
		instance = serializer.save()
		assert instance.status == ExampleModel.Status.APPROVED

	def test_output_serializer(self):
		serializer = ExampleOutputSerializer(self.instance)
		assert set(serializer.data.keys()) == set(serializer.Meta.fields)
		assert serializer.Meta.read_only_fields == serializer.Meta.fields
```

### View Tests

- Mirror existing fixtures from `core/test_fixtures.py` for setup and authentication.
- Resolve permissions using `Model.get_*_permission(ignore_app_label=True)` in `setup_method` and add them to the authenticated user before making requests.
- Build endpoint URLs with `django.urls.reverse` and the route names defined in `urls.py`; avoid hard-coding path strings.
- When asserting response payloads, instantiate the output serializer with the expected queryset or instance and compare `serializer.data` to `response.json()` so tests stay aligned with the view serialization logic.
- Include positive (with permission) and negative (without permission) paths.
- Include ownership boundary tests: verify that user B cannot access user A's data.
- Assert database state after each action.

#### Example View Test Pattern

```python
@pytest.mark.django_db
class TestExampleListView:
	def setup_method(self):
		self.client = APIClient()
		self.community = TestFixtures.create_community()
		self.member = TestFixtures.create_member()
		TestFixtures.create_membership(community=self.community, member=self.member)
		self.member.user.user_permissions.add(Permission.objects.get(codename='view_examplemodel'))
		self.client.force_authenticate(user=self.member.user)
		self.instance = TestFixtures.create_example(community=self.community)
		self.url = reverse('example:example-list', kwargs={'community_id': self.community.id})

	def test_get_list(self):
		response = self.client.get(self.url)
		assert response.status_code == status.HTTP_200_OK
		assert len(response.data) == 1

	def test_get_list_requires_membership(self):
		other_member = TestFixtures.create_member()
		self.client.force_authenticate(user=other_member.user)
		response = self.client.get(self.url)
		assert response.status_code == status.HTTP_403_FORBIDDEN
```

### Model Tests

- Group related assertions inside a single test class per model; create shared objects in `setup_method`.
- Prefer helper methods for repeated object construction so capacity, timing, or flag changes only live in one place.
- Cover both steady-state behavior and state transitions. Refresh instances before asserting post-conditions.
- Use `timezone.now()` once per helper to avoid drift-related failures.

---

## Consistency Checklist (Django)

- Class inherits from the expected base (`ProjectBaseModel` or `ProjectBaseAPIView`).
- Serializers split into input/output serializers when necessary.
- Serializer field tuples verified for completeness (no duplicates, no omissions).
- URL names follow kebab-case `{model}-{action}`.
- Tests cover permission-positive, permission-negative, and cross-user isolation cases.
- List endpoints scoped to current user/tenant.
- Mutating endpoints return the created/updated resource.
- No I/O hidden in model lifecycle methods.
- `ruff check` passes on all modified files.
