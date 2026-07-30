---
name: context-gatherer
description: >
  Systematically gather and document context about a specific topic, feature, component, or area of the codebase.
  Use when exploring unfamiliar code, onboarding to a feature, or creating documentation about how things work.
---

# Context Gatherer - Load IQ

## When to use
- Learning about a new feature or component
- Understanding data flow through the system
- Documenting how pieces fit together
- Onboarding to unfamiliar parts of the codebase
- Investigating dependencies and relationships
- Creating knowledge base entries
- Before making significant changes to understand impact
- When asked "how does X work?" or "what does Y do?"

---

## What this skill produces

A structured report containing:
- **File/directory inventory** - What exists and where
- **Key components** - Main files, classes, functions
- **Data flow** - How information moves through the system
- **Dependencies** - What imports what, what calls what
- **Usage patterns** - Where and how components are used
- **Business logic** - What the code actually does
- **Integration points** - APIs, stores, events, external systems

---

## Investigation workflow

### 1. Define the scope (2 minutes)

**What are you investigating?**
- Specific feature? (e.g., "load builder wizard")
- Component type? (e.g., "all operator input forms")
- Data domain? (e.g., "Snowflake connection management")
- Technical pattern? (e.g., "how authentication works")
- System integration? (e.g., "Airflow DAG generation")

**Set boundaries:**
- Backend only, frontend only, or both?
- Specific subdirectory or entire codebase?
- Just structure, or include implementation details?
- Surface level (files/folders) or deep dive (logic/algorithms)?

**Expected output:**
- One-sentence summary of what you're exploring
- Clear boundary statement (e.g., "frontend components only, not backend")

---

### 2. Discover structure (5-10 minutes)

**Find relevant files:**
```bash
# By file pattern
find <path> -name "*<keyword>*"
glob pattern: "**/*<keyword>*"

# By directory
ls -la <directory>
view <directory>  # For tree view

# By content
grep pattern: "<keyword>" path: <directory>
grep -r "<keyword>" <directory> --include="*.py"
```

**Map the landscape:**
- [ ] List all relevant directories
- [ ] Count files by type (`.py`, `.vue`, `.ts`)
- [ ] Identify largest files (potential complexity hotspots)
- [ ] Note file naming conventions
- [ ] Spot organizational patterns (features/, components/, models/)

**Document:**
- Directory tree with purpose annotations
- File count statistics
- Top 5-10 largest/most important files

---

### 3. Identify key components (10-15 minutes)

**Backend components:**
- [ ] Models - Data structures and database tables
- [ ] Views/ViewSets - API endpoints and request handlers
- [ ] Serializers - Input/output data validation
- [ ] Services - Business logic layer
- [ ] Tasks - Celery/background jobs
- [ ] Utils/Helpers - Shared functionality

**Frontend components:**
- [ ] Views - Top-level page components
- [ ] Components - Reusable UI pieces
- [ ] Stores - Pinia state management
- [ ] Composables - Shared reactive logic
- [ ] Services - API clients and utilities
- [ ] Types - TypeScript interfaces/types

**For each key component, extract:**
- Purpose (what does it do?)
- Location (file path)
- Key methods/functions (top 3-5)
- Dependencies (what does it import?)
- Size (line count)

---

### 4. Trace data flow (10-15 minutes)

**Follow the data:**

**Backend flow:**
1. **Entry point** - Which URL/endpoint receives requests?
2. **Validation** - What serializers validate input?
3. **Processing** - What services/models handle business logic?
4. **Storage** - What database tables are affected?
5. **Response** - What data is returned?

**Frontend flow:**
1. **User action** - What triggers the flow? (button, form, route)
2. **Component** - Which Vue component handles it?
3. **Store** - Does it update Pinia state?
4. **API call** - Which backend endpoint is called?
5. **Response handling** - How is data processed and displayed?

**Integration flow:**
1. **Frontend → Backend** - API calls, request format
2. **Backend → Database** - Queries, models, migrations
3. **Backend → External** - Third-party APIs, services
4. **Background jobs** - Async processing, task queues

**Document:**
- Request/response examples
- Data transformation points
- State management patterns
- Error handling approach

---

### 5. Map dependencies and relationships (10 minutes)

**Imports and usage:**
```bash
# What does X import?
grep -n "^import\|^from" <file>

# Who imports X?
grep -r "from.*<module> import\|import.*<module>" <directory>

# What calls function/class X?
grep -r "<function_name>\(" <directory>
```

**Track relationships:**
- [ ] What does this component depend on?
- [ ] What components depend on this?
- [ ] Are there circular dependencies?
- [ ] What external libraries are used?
- [ ] What internal utilities are used?

**Identify coupling:**
- Tight coupling (direct imports, shared state)
- Loose coupling (events, dependency injection)
- Integration points (APIs, databases, external services)

---

### 6. Understand business logic (15-20 minutes)

**Read key implementations:**

For each major component:
- [ ] Read the main entry point (view, component setup)
- [ ] Identify core algorithm or workflow
- [ ] Note special cases and edge handling
- [ ] Understand validation rules
- [ ] Document business rules embedded in code

**Look for:**
- Conditional logic (if/else, switch)
- Loops and iterations
- Data transformations
- Validation rules
- Permission checks
- Error handling

**Extract:**
- What problem does this solve?
- What are the happy path and edge cases?
- What business rules are enforced?
- What assumptions does the code make?

---

### 7. Find usage patterns (10 minutes)

**How is it used in practice?**

```bash
# Find all usages
grep -r "<ComponentName>\|<function_name>" <directory>

# In templates
grep -r "<ComponentName" --include="*.vue"

# In imports
grep -r "import.*<ComponentName>" --include="*.ts" --include="*.vue"
```

**Document:**
- Common usage patterns (with code examples)
- Typical prop/parameter values
- Standard configurations
- Anti-patterns (what NOT to do)
- Edge cases handled in practice

---

### 8. Note integration points (5-10 minutes)

**External connections:**
- [ ] REST APIs consumed or exposed
- [ ] Database tables read/written
- [ ] External services (Snowflake, Airflow, etc.)
- [ ] Background jobs triggered
- [ ] Events emitted or listened to
- [ ] WebSocket connections
- [ ] File system operations

**Internal connections:**
- [ ] Shared stores accessed
- [ ] Global state dependencies
- [ ] Router navigation
- [ ] Event bus usage
- [ ] Composable dependencies

---

## Output format

### Summary (5 sentences max)
- What was investigated
- Overall purpose/function
- Key technologies/patterns used
- Maturity/completeness assessment
- Main complexity drivers

### Structure
```
<topic>/
├── directory1/          # Purpose
│   ├── file1.py        # What it does
│   └── file2.py        # What it does
└── directory2/          # Purpose
    └── file3.vue       # What it does
```

File counts, size stats, organizational notes

### Key Components

**Backend:**
- **ModelName** (`path/to/model.py`, 250 lines)
  - Purpose: What it represents
  - Key fields: Important attributes
  - Methods: Top 3-5 methods
  - Dependencies: What it uses

**Frontend:**
- **ComponentName** (`path/to/Component.vue`, 180 lines)
  - Purpose: What it displays/does
  - Props: Key inputs
  - Emits: Events fired
  - Dependencies: Stores, composables used

### Data Flow

**Request flow:**
```
User Action → Component → Store → API → Backend View → Service → Model → Database
                                                                          ↓
Response ← Component ← Store ← API ← Backend View ← Service ← Model ← Query Result
```

With specific examples for this feature/component

### Dependencies

**Imports:**
- Lists what this component/module imports
- External libraries used
- Internal modules depended on

**Used by:**
- Components/modules that depend on this
- Frequency of usage (high/medium/low)

### Business Logic

- Core algorithms or workflows
- Validation rules
- Permission/authorization logic
- Edge cases handled
- Known limitations

### Usage Patterns

Code examples showing:
- Typical initialization
- Common operations
- Standard configurations
- Error handling approach

### Integration Points

**APIs:**
- Endpoints exposed or consumed
- Request/response formats
- Authentication/authorization

**External Systems:**
- Snowflake, Airflow, etc.
- What data is exchanged
- How connection is managed

**Internal Systems:**
- Stores accessed
- Events used
- Shared utilities

### Open Questions

List anything unclear or requiring further investigation:
- Ambiguous logic
- Missing documentation
- Inconsistencies found
- Potential issues spotted

---

## Tips for effective context gathering

### ✅ DO:
- Start broad (structure) then go deep (logic)
- Use parallel tool calls (read multiple files at once)
- Document as you go (don't rely on memory)
- Note file locations with every finding
- Include code snippets for clarity
- Verify assumptions by reading actual code
- Track dependencies both ways (imports and imported by)
- Look for patterns across similar components

### ❌ DON'T:
- Get lost in implementation details too early
- Skip reading key files (always verify)
- Assume based on naming alone
- Ignore test files (they show usage)
- Miss integration points
- Forget to note file sizes and complexity
- Trust comments over code
- Document EVERYTHING (focus on key insights)

---

## Common investigation patterns

### "How does feature X work?"
1. Find feature directory
2. Identify main views/components
3. Trace user flow (UI → store → API)
4. Read backend views/models
5. Document data flow
6. Note integration points

### "What does component Y do?"
1. Read component file
2. Check props, emits, exposed methods
3. Find where it's used
4. Trace data dependencies
5. Understand its role in parent context

### "Where is logic Z implemented?"
1. Search for keywords
2. Find relevant files
3. Read implementations
4. Check for duplication
5. Document all locations

### "How do A and B integrate?"
1. Find both components
2. Search for cross-references
3. Identify communication mechanism
4. Document data exchange format
5. Note error handling

---

## Depth levels

Choose appropriate depth based on investigation goal:

### Surface (files and folders only)
- Fast overview of structure
- File counts and sizes
- Directory organization
- Use when: First exposure to codebase area

### Component-level (interfaces and signatures)
- What modules/classes/functions exist
- Function signatures and types
- Component props and emits
- Public APIs
- Use when: Understanding architecture

### Implementation-level (logic and algorithms)
- How things actually work
- Business logic details
- Algorithms and data processing
- Edge cases and validation
- Use when: Making changes or deep troubleshooting

### Integration-level (connections and flow)
- How components communicate
- Data flow through system
- External dependencies
- State management patterns
- Use when: Understanding system behavior end-to-end

---

## Living document notice

**This skill is designed to evolve with the codebase.**

If during context gathering you discover:
- Better investigation techniques
- Useful search patterns
- Common gotchas or confusion points
- Helpful visualization approaches
- Time-saving shortcuts
- Load IQ-specific patterns worth noting

**Update this skill.**

Add to relevant sections:
- New search patterns that worked well
- Better ways to trace dependencies
- Effective documentation formats
- Domain-specific investigation techniques

The goal is to make future context gathering faster and more thorough by capturing what works.

---

## Quick reference commands

```bash
# Structure
find <path> -type f -name "*.py" | wc -l
find <path> -type f -name "*.vue" | wc -l
view <directory>

# Largest files
find <path> -name "*.py" -exec wc -l {} + | sort -rn | head -20

# Search for keywords
grep pattern: "<keyword>" path: <path>
grep -r "class <Name>" <path> --include="*.py"
grep -r "def <function>" <path> --include="*.py"

# Find imports
grep -n "^import\|^from" <file>
grep -r "from.*<module> import" <path>

# Find usage
grep -r "<ComponentName>" <path> --include="*.vue"
grep -r "<function_name>\(" <path>

# File contents
view <file>
view <file> [start, end]  # Specific line range
```

---

## Example usage
- "Gather context on the load builder wizard"
- "How does Snowflake connection management work?"
- "Explain the operator system in the file cleanse tab"
- "What does the metadata app do?"
- "Document how form validation works in the frontend"
- "Map out the daily monitor feature"
- "Show me how background jobs are structured"

---

## Success criteria

A successful context gathering session produces documentation that:
- ✅ Someone unfamiliar with the code can understand the area
- ✅ Shows concrete file locations and examples
- ✅ Explains both structure AND behavior
- ✅ Identifies key complexity and integration points
- ✅ Answers "what", "where", "how", and "why"
- ✅ Includes enough detail to make informed changes
- ✅ Highlights open questions or uncertainties
- ✅ Can be referenced later without re-reading code