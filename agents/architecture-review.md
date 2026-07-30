## Architecture Review Rubric

This file defines a reusable audit rubric for evaluating code quality across any repository. Use it for periodic reviews, onboarding assessments, or pre-merge audits.

The rubric is organized into 11 categories with 50 evaluation principles. Each principle has a question to ask, signals to look for, and severity guidance.

---

### How to Use This Rubric

1. **Select scope**: Full repo, single app/module, or a PR.
2. **Walk the categories**: For each category, answer the questions using actual source code, not assumptions.
3. **Record findings**: Note specific file paths, line numbers, and code examples. Avoid vague observations.
4. **Classify severity**: Critical (security/data loss risk), High (correctness bug), Medium (maintainability), Low (style/convention).
5. **Document in FINDINGS.md**: The canonical audit ledger lives in `FINDINGS.md` at the repo root.

---

### Category 1: Simplicity, Readability, and Intent Clarity

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 1 | Single responsibility per function | Does each function do one thing? | Functions under 30 lines, clear names | Functions over 100 lines, vague names like `process()` |
| 2 | Shallow control flow | Are conditionals shallow with early returns? | Max 2 levels of nesting | 4+ levels, deeply nested if/else chains |
| 3 | Self-documenting names | Can you understand the code without comments? | `get_active_subscriptions()` | `get_data()`, `process()`, `handle()` |
| 4 | No unnecessary comments | Are comments explaining "why" not "what"? | Comments on business rules only | `# increment counter` above `counter += 1` |
| 5 | Logical spacing | Is code grouped by intent with blank lines? | Related steps grouped together | Wall of code with no visual separation |

### Category 2: Boundaries, Responsibility, and Separation of Concerns

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 6 | Layer separation | Are UI, transport, domain, and persistence separated? | Views call services, services call models | Views contain SQL queries and HTML rendering |
| 7 | No god modules | Is any file handling 3+ unrelated concerns? | Files under 300 lines, focused names | `common.py` with auth + email + crypto + models |
| 8 | Service boundaries | Are third-party integrations behind service layers? | `stripe_service.py` wraps all Stripe calls | Stripe API calls scattered across views |
| 9 | No I/O in model lifecycle | Are `save()`/`delete()` free of network calls? | Lifecycle methods only do DB work | Stripe/email/webhook calls in `save()` |
| 10 | Clear module boundaries | Can you change one module without affecting unrelated ones? | Explicit interfaces between modules | Circular imports, shared mutable state |

### Category 3: Abstraction Quality and Indirection

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 11 | Justified abstractions | Does every abstraction serve 2+ consumers? | Shared base classes used by 5+ models | `AbstractFactory` with one implementation |
| 12 | No premature abstraction | Are abstractions born from real duplication? | Helper extracted after 3rd copy | Abstract class created for the first implementation |
| 13 | Transparent indirection | Can you trace a call from entry to effect in 3 hops? | View -> service -> model | View -> adapter -> factory -> strategy -> handler -> model |
| 14 | YAGNI compliance | Is there code for features that don't exist yet? | No unused interfaces or stub methods | Empty methods "for future use" |
| 15 | Appropriate generality | Are abstractions at the right level? | `BaseExtractor` for multiple data sources | `BaseAnything` for one thing |

### Category 4: Coupling, Cohesion, and Modularity

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 16 | Loose coupling | Do modules depend on interfaces, not internals? | Explicit arguments, clear boundaries | `from other_app.models import *` |
| 17 | High cohesion | Do files/classes contain related functionality? | `AlertService` handles alert CRUD only | `Utility` class with email + date + string helpers |
| 18 | No circular dependencies | Can you draw the dependency graph as a DAG? | Clean import hierarchy | `A imports B imports C imports A` |
| 19 | Explicit dependencies | Are dependencies passed, not reached for? | Constructor/function parameters | Global state access, `import settings` deep in logic |
| 20 | Package isolation | Can you remove an app without cascading failures? | Feature apps are self-contained | Removing one app breaks 5 others |

### Category 5: Data Flow and Side Effects

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 21 | Predictable data flow | Can you trace data from input to output? | Request -> validate -> process -> respond | Data modified through 3 signal handlers before response |
| 22 | No hidden side effects | Do functions do only what their name says? | `calculate_total()` returns a number | `calculate_total()` also sends email |
| 23 | Immutable where practical | Is shared state minimized? | Pure functions, new objects returned | Global mutable dictionaries modified everywhere |
| 24 | Deterministic behavior | Same input, same output? | No time-dependent, random, or order-dependent logic without explicit seeding | Results change based on execution order |
| 25 | Clean error propagation | Do errors flow up, not sideways? | Exceptions bubble to the appropriate handler | Errors stored in global state, checked later |

### Category 6: Error Handling and Robustness

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 26 | Fail fast | Do errors surface immediately? | `ValidationError` raised at boundary | Silent `None` return, error logged but swallowed |
| 27 | Specific exceptions | Are caught exceptions specific? | `except KeyError:` | `except Exception: pass` |
| 28 | No silent swallowing | Are errors always reported? | Exceptions logged or re-raised | `try/except: pass` |
| 29 | Appropriate defaults | Are defaults justified, not defensive? | Default page size = 10 | Default user_id = 0 |
| 30 | Graceful degradation | Does partial failure leave the system consistent? | Transaction rollback on error | Half-written records on exception |

### Category 7: Testability and Change Safety

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 31 | Tests exist | Is there meaningful test coverage? | Tests for happy path, edge cases, permissions | Zero tests or only smoke tests |
| 32 | Tests mirror real usage | Do tests exercise the code like a real user would? | API tests via HTTP client, correct auth | Tests calling internal methods directly |
| 33 | Tests verify boundaries | Do tests confirm ownership/permission isolation? | "User B cannot see User A's data" test | Tests that codify insecure behavior as expected |
| 34 | Parameterized coverage | Are edge cases systematically tested? | `@pytest.mark.parametrize` for missing fields | One happy-path test per endpoint |
| 35 | Deterministic tests | Do tests pass consistently? | No time-dependent flakes, no shared state | Tests fail on certain days or in certain order |

### Category 8: Performance and Scalability

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 36 | No premature optimization | Is performance work driven by measurement? | Profiling before optimizing | Complex caching for endpoints called once/minute |
| 37 | Appropriate data structures | Are collections chosen for their access patterns? | `dict` for lookups, `list` for iteration | Linear search through a list for every lookup |
| 38 | No busy loops | Are waits event-driven, not poll-based? | `await`, `ManualResetEventSlim`, signals | `while (true) { Thread.Sleep(10); }` |
| 39 | Resource cleanup | Are resources released promptly? | `with open()`, `using`, `_ExitTree()` cleanup | Unclosed file handles, leaked connections |
| 40 | Bounded operations | Are batch operations bounded? | Pagination, chunked processing | `Model.objects.all()` on a million-row table |

### Category 9: Observability, Traceability, and Debuggability

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 41 | Structured logging | Are logs useful for debugging? | `logger.info("order_created", order_id=123)` | `print("here")`, `logger.info("Spinach")` |
| 42 | Appropriate log levels | Are levels used correctly? | ERROR for failures, INFO for state changes, DEBUG for details | Everything at INFO or DEBUG |
| 43 | No debug artifacts | Is development logging cleaned up? | Clean log output | `print()` statements, memory dumps, banner separators |
| 44 | Traceable requests | Can you follow a request through the system? | Request IDs, correlation headers | No way to connect a log line to a specific request |
| 45 | Meaningful error messages | Do errors explain what went wrong? | `"User 123 not found in company 456"` | `"Error"`, `"Something went wrong"`, `"Ahhh shit"` |

### Category 10: Domain Alignment and Conceptual Integrity

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 46 | Domain-driven naming | Do names match the business domain? | `WorkOrder`, `Customer`, `Subscription` | `DataObject`, `Item`, `Thing` |
| 47 | Consistent naming | Is the same concept called the same thing everywhere? | `customer` in models, views, serializers, templates | `customer` in models, `client` in views, `user` in templates |
| 48 | Consistent casing | Does casing follow stack conventions? | PascalCase for C# types, snake_case for Python, camelCase for JS | Mixed conventions in the same file |

### Category 11: Security and Dependency Hygiene

| # | Principle | Question | Good Signal | Bad Signal |
|---|---|---|---|---|
| 49 | No committed secrets | Are credentials external to source code? | `.env` files in `.gitignore`, env vars at runtime | `.env.secret` with live Stripe keys in repo |
| 50 | Authenticated endpoints | Do all API endpoints require auth? | Explicit `authentication_classes` and `permission_classes` | Empty `authentication_classes = []` |

---

### Severity Classification

| Level | Criteria | Examples |
|---|---|---|
| **Critical** | Security vulnerability, data loss/corruption risk, or system crash | Cross-user data leakage, committed production secrets, use-after-cancel bugs |
| **High** | Correctness bug that produces wrong results | Duplicate serializer fields, wrong variable in serialization, broken pathfinding formula |
| **Medium** | Maintainability issue that increases defect risk | God modules, missing tests, code duplication, thread safety in non-production paths |
| **Low** | Style/convention violation with no immediate risk | Naming typos, inconsistent casing, line length violations |

---

### Audit Report Template

When documenting findings, use this structure per repository:

```markdown
## [Repository Name]

### Stack Summary
[One paragraph: language, framework, purpose, deployment model]

### Critical Bugs
| Bug | Location | Severity |
|---|---|---|

### What Is Good
- [Specific patterns, architecture decisions, or code quality strengths]

### What Is Bad
- [Specific problems with file paths and line numbers]

### Candidate Rules to Promote
- [Patterns discovered that should become guidance for all projects]
```

---

### Delta Matrix Template

Track which rules from the rubric are covered by existing guidance and which are missing:

```markdown
| Rule / Pattern | Evidence Repo(s) | Current Location | Status | Proposed Destination | Action |
|---|---|---|---|---|---|
| [Rule name] | [Where discovered] | [File if exists] | match/partial/missing | [Target file] | keep/add/clarify |
```
