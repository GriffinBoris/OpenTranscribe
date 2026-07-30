## .NET / C# / Godot Guidelines

This file covers C#, .NET, Godot 4, ECS (Arch), game client/server architecture, and code generation tooling. Read `agents/AGENTS.md` first for cross-stack principles.

---

### Project Structure

Follow clear project separation within .NET solutions:

| Project | Responsibility |
|---|---|
| **Shared** (class library) | Game logic, ECS components/systems, network protocol contracts, code generation |
| **Server** (console app) | Connection handling, session management, game loop orchestration |
| **Client** (Godot project) | Presentation, scene management, input handling, ECS-to-Godot bridge |
| **CodeGen** (Python tool) | Schema-driven code generation for network protocol |

Do not leak presentation concerns into shared libraries or game logic into network transport code.

#### Architecture Decisions (from existing game stack)

- Layering is intentional: protocol -> transport -> sync/async bridge. Do not flatten layers to “simplify”.
- Transport queue vs endpoint: keep the queue public if higher-level wrappers are convenience utilities.
- Custom channel variants are scaffolding for future extensibility; do not remove because they are unused today.
- Three-tier config separation (game -> network -> transport) is deliberate.
- Small files are acceptable when they enforce explicit imports and keep modules focused.

---

### ECS Architecture (Arch)

- **Components are structs.** Keep them as plain data containers with no behavior. Use extension methods for serialization/deserialization.
- **Systems contain logic.** Each system should have a single responsibility (rendering, movement, growth, network sync).
- **Keep the ECS boundary clean.** Godot scene nodes should not directly manipulate ECS components. Use a bridge layer (e.g., `GameInstanceService`) that translates between Godot events and ECS operations.
- **Interface segregation for entities.** Define small, focused interfaces (`IGodotArchEntity`, `IGodotArchHoverComponent`, `IGodotArchGrowthComponent`) rather than one large entity interface.
- **Avoid large switch statements for entity type dispatch.** When mapping entity types to factory calls or scene instantiation, prefer a dictionary or registry pattern that can be extended without modifying existing code.

---

### Network Protocol & Packets

- **Schema-driven code generation** is the correct pattern for multi-platform networking. Generate packet serialization code from YAML schemas so client and server stay in sync.
- **Mark generated files clearly** with "do not edit" headers and keep them in a separate directory from hand-written code.
- **Packet serialization must be correct.** Audit all `Add*` methods to ensure they write the correct parameter, not a stale field. This is a real bug category (e.g., `AddLong` using `SentTimestamp` instead of the `l` parameter).
- **Avoid shared static buffers for packet construction.** A static `byte[1024]` buffer is not thread-safe and silently truncates large packets. Use per-call buffers or a buffer pool.
- **Use `ConcurrentQueue<T>`** instead of `Queue<T>` when queues are accessed from multiple threads.
- **Consistent key naming in packet data.** Pick one convention (camelCase or PascalCase) for component data keys and enforce it across all components. Do not mix conventions.

---

### Thread Safety

Thread safety issues are the most common critical bug category in the .NET projects. Follow these rules:

- **Use `ConcurrentDictionary`, `ConcurrentQueue`, `ConcurrentBag`** for collections accessed from multiple threads. Plain `List<T>`, `Dictionary<K,V>`, and `Queue<T>` are not thread-safe.
- **Use `Interlocked.Increment`** for counters shared across threads (session IDs, entity IDs, packet IDs). `_counter++` is a data race.
- **Do not share mutable static state** across threads. Static fields like `PacketFactory._buffer` or `GameState._entityIdCounter` are unsafe without synchronization.
- **`CancellationTokenSource` cannot be reused after cancellation.** Create a new one for each cancellation cycle. Reusing a cancelled token means the next operation is immediately cancelled.
- **Use `lock` or `SemaphoreSlim`** when atomic read-modify-write operations span multiple statements.

```csharp
// WRONG -- data race
private static int _sessionId = 0;
public int GetNextSessionId() => _sessionId++;

// CORRECT -- atomic increment
private static int _sessionId = 0;
public int GetNextSessionId() => Interlocked.Increment(ref _sessionId);
```

---

### Server Lifecycle

- **Explicit startup/shutdown ownership.** Server processes must have clear initialization, run, and shutdown phases. Do not use `while (true)` as the process lifetime -- it spins CPU at 100%. Use `ManualResetEventSlim`, `TaskCompletionSource`, or a host abstraction.
- **Graceful shutdown.** Listen for cancellation signals and clean up connections, sessions, and background tasks before exiting.
- **No nested fire-and-forget tasks.** Every `Task.Run` should be tracked and awaited during shutdown. Unobserved task exceptions are silent bugs.
- **Bind to configurable addresses.** Do not hardcode `IPAddress.Loopback` or fixed port numbers. Use configuration or command-line arguments.
- **Tests should use ephemeral ports** (`port 0`) rather than fixed ports to avoid conflicts in CI.

---

### Game Client (Godot)

#### Scene Management

- **Keep scene classes thin.** Scene nodes handle input, presentation, and Godot lifecycle (`_Ready`, `_Process`, `_ExitTree`). Push game logic, networking, and state management to services.
- **Always clean up in `_ExitTree()`.** Unsubscribe from events, dispose materials/resources, and disconnect signals. This prevents memory leaks and dangling callbacks.
- **Use typed scene path management** (e.g., enum-based scene paths) rather than hardcoded string paths scattered through the codebase.
- **Centralize scene transitions** in a scene manager with stack-based navigation, animation support, and pause behavior.

#### Global State

- **Use global singletons sparingly.** Static state (`GlobalState`, `NavController`) should be limited to stable, cross-cutting concerns (main-thread dispatch queue, navigation controller). Do not use globals for game state that could be scoped to a session or scene.
- **Thread-safe main-thread dispatch** via `ConcurrentQueue<Action>` is the correct pattern for dispatching network responses to Godot's main thread. Keep using this.

#### Presentation Layer

- **Do not instantiate network/service classes directly in view-layer entities.** Scene nodes should receive services via dependency injection, autoload singletons, or a service locator -- not by constructing `new NetworkApi()` inline.
- **Extract duplicated behavior** (e.g., hover logic shared between Tile and Ploppable) into a base class or composition helper.
- **Delete commented-out code.** Version control has history. Do not leave 100+ lines of commented-out code as "reference."
- **Remove empty scaffold methods.** If `_Ready()` or `_Process()` have no implementation, remove them entirely.
- **Avoid hardcoded magic numbers.** Window sizes, camera constants, animation speeds, and growth parameters should be in constants, project settings, or resource files.

#### CI / Build

- **Keep CI Godot version in sync with the project.** If the project uses Godot 4.3, CI export workflows must also use 4.3, not 4.2.2.
- **Custom project-specific linters** (e.g., PascalCase enforcement for scene nodes) are a good pattern. Keep using them.

---

### Code Generation Tooling

- **YAML schema -> Pydantic models -> Jinja2 templates -> C# output** is a sound architecture. Keep it.
- **Use `yaml.safe_load()`** always. Never use `yaml.unsafe_load()` or `yaml.load()` without a safe loader.
- **Jinja2 custom filters** for casing conversion (`camel_case`, `pascal_case`) are clean and should be reused across templates.
- **Always use `with open()`** for file I/O. Unclosed file handles are resource leaks.
- **Honor all CLI arguments.** If `argparse` declares `--user` and `--replace`, the code must use them.
- **No hardcoded user paths.** Replace `/home/griffin/...` with environment variables or runtime path discovery.
- **Version-pin all dependencies.** `requirements.txt` must have specific version constraints.
- **Declare all dependencies.** If `pyyaml` is imported, it must be in `requirements.txt`.
- **Remove dead code generation targets.** If Kotlin templates exist but are never rendered, remove them.

---

### C# Coding Style

- **File-scoped namespaces** (`namespace Foo;`) are preferred over block-scoped.
- **Nullable reference types** should be enabled (`<Nullable>enable</Nullable>`).
- **Prefer `var` for locals** when the right-hand side makes the type obvious.
- **Use target-typed `new()`** where it improves readability (especially for DI and simple constructors).
- **Use collection expressions** (`[]`) for empty list initialization when supported.
- **Use proper exception types.** `throw new Exception("message")` is too generic. Use `InvalidOperationException`, `ArgumentException`, `ArgumentNullException`, or custom exception types as appropriate. Never use informal messages like `"Ahhh shit, can't move there!"`.
- **Properties over public fields.** Use `{ get; set; }` or `{ get; private set; }` instead of `public` fields on data objects (except for ECS component structs, which should be plain fields for performance).
- **Null-forgiving operator (`!`)** should be used sparingly. If a reference might actually be null, check it. Do not use `!` to suppress warnings on legitimately nullable values.
- **Use `ILogger` or Serilog** for logging. Replace `Console.WriteLine` with structured logging in any code that will run in production.
- **Keep consistent version references.** If a shared NuGet package is referenced by multiple projects, all projects should reference the same version.

#### Example Patterns from the Codebase

**Primary constructor with DI**:
```csharp
public class GameService(IGameServiceConsumer gameServiceConsumer, Settings settings) : IGameService
{
    private readonly List<ArchSystem> _systems = [];
    private readonly List<ArchSystem> _physicsSystems = [];
}
```

**Switch expression for OpCode dispatch**:
```csharp
IPacket? responsePacket = gameRequest.RequestPacket.OpCode switch
{
    (int)OpCode.JoinMatchRequest => HandleJoinMatchRequest((JoinMatchRequestPacket)gameRequest.RequestPacket),
    (int)OpCode.SendMessageRequest => HandleSendChatMessage((SendMessageRequestPacket)gameRequest.RequestPacket),
    _ => null
};
```

**Null-forgiving on Godot nodes set in _Ready()**:
```csharp
private Node _entityContainer = null!;

public override void _Ready()
{
    _entityContainer = GetNode<Node>("World/EntityContainer");
}
```

**NUnit constraint-style assertions**:
```csharp
Assert.That(_socket.Connected, Is.True);
Assert.That(_sessionService.ClientSessions.Count, Is.EqualTo(1));
```

---

### Signal / Event Patterns

- **`SignalBridge` (static event system)** is a convenient pattern for cross-layer communication but increases hidden coupling. Use it for:
  - Network response -> game logic notifications
  - Game state changes -> UI updates
  - Scene lifecycle events
- **Always unsubscribe in cleanup methods** (`_ExitTree`, `Dispose`, destructors). Leaked subscriptions cause memory leaks and ghost callbacks.
- **Prefer explicit event parameters** over broad `EventArgs`. The subscriber should know exactly what data it's receiving.

---

## Consistency Checklist (.NET / Godot)

- Thread-safe collections used for shared state
- Atomic operations for shared counters (`Interlocked`)
- `CancellationTokenSource` not reused after cancellation
- Events unsubscribed in `_ExitTree()` / `Dispose()`
- No commented-out dead code
- No hardcoded magic numbers, ports, or user paths
- Generated code in separate directory with "do not edit" headers
- CI Godot version matches project version
- Proper exception types (not bare `Exception`)
- `ILogger` / Serilog instead of `Console.WriteLine`
- All NuGet package versions consistent across projects
