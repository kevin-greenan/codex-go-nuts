# Kiln Web

Kiln Web is the planned Noema-native web application stack for Hearthlight and future services.

The target is a small embedded web framework closer in spirit to Flask than to a large application server: route declarations, request and response objects, HTML rendering, middleware-style helpers, and a development loop simple enough to use from a single Noema application.

## Mission

Kiln Web should let a Noema program:

1. listen for HTTP requests,
2. route them to handler functions,
3. render HTML responses,
4. serve forms and basic assets,
5. coordinate cleanly with AshDB-backed application logic.

The first version should optimize for server-rendered applications rather than SPAs.

## Product Shape

Kiln Web should be delivered as:

1. A Noema library for HTTP server and routing support.
2. A Noema HTML rendering layer.
3. Example applications under this folder.
4. Development helpers for local testing.

## Repository Placement

Kiln Web should also stay as a root-level project for now.

That gives us a cleaner boundary:

1. `kiln-web/` owns the framework, renderer, examples, and app-facing APIs.
2. `codex-lang/lib/` can absorb generic helpers like shared text, bytes, or HTTP utility code only when they prove broadly reusable.
3. `codex-lang/` owns the compiler and runtime features needed to make the framework possible.

This keeps `codex-lang` focused on the language and runtime while still allowing shared primitives to migrate into `codex-lang/lib/` over time.

## Non-Goals For Early Versions

1. ASGI-style concurrency.
2. WebSockets.
3. Full template inheritance and macro systems.
4. ORM integration.
5. Production-grade reverse proxy features.

## Proposed Repository Layout

This folder should grow into:

1. `kiln-web/lib/` for HTTP, routing, and rendering code.
2. `kiln-web/examples/` for route and form examples.
3. `kiln-web/docs/` for API and template docs.
4. `kiln-web/tests/` for request and response cases.
5. `kiln-web/dev/` for local harnesses.

## Architecture Overview

Kiln Web should be built in layers:

### 1. TCP Server Runtime

Accept TCP connections, read request bytes, write response bytes, and close connections safely.

### 2. HTTP Parsing and Serialization

Parse request lines, headers, query strings, and bodies. Serialize responses with status lines, headers, cookies, and bodies.

### 3. Router

Map method and path patterns to handler functions and route params.

### 4. Request and Response Model

Represent:

1. method
2. path
3. query params
4. headers
5. form body
6. route params
7. response status
8. response headers
9. response body

### 5. HTML Rendering Layer

Render HTML from structured Noema helpers rather than relying only on string concatenation.

## Rendering Direction

The first renderer should be deliberately simple:

1. functions for safe HTML escaping
2. helpers for tags and attributes
3. plain text templates assembled from Noema functions
4. layout helpers for common page structure

We should not begin with a complex template language. A function-based rendering layer is more achievable and easier to debug while the language is still evolving.

## Required Noema Language and Runtime Support

Kiln Web also needs features that the current runtime does not yet expose.

### Server-Side Networking

Current Noema has outbound TCP clients only. Kiln Web requires inbound server primitives:

1. `listener` or `server_socket` type.
2. `socket_listen(host, port)` or `listener_open(port)`.
3. `socket_accept(listener)` returning a connected socket.
4. optional `socket_set_reuseaddr(...)`.

### Better Byte Handling

HTTP parsing should work on raw bytes or byte-safe text handling, especially for headers and request bodies.

Kiln Web should reuse the same `bytes` runtime work planned for AshDB.

### Time and Date Helpers

Even minimal HTTP responses benefit from timestamps and cookie expiry support. We should plan for:

1. current unix timestamp
2. basic date formatting helpers, either in runtime or library space

This is not strictly phase-one critical, but it will matter quickly.

### Optional Process Helpers For Development

Auto-reload is nice to have, not required. The first version can skip it.

## Compiler Work Required

Kiln Web depends on the same compiler discipline as AshDB:

1. Rust stage-1 compiler updates.
2. Self-hosted compiler updates.
3. C backend lowering.
4. Native backend lowering.
5. emitted runtime support in both compiler paths.

The likely builtin additions are:

1. server listener type support
2. listen and accept builtins
3. bytes support shared with AshDB
4. optional time builtins later

## Shared Foundation With AshDB

These two library efforts should not drift apart. They share a common platform roadmap:

1. `bytes` type
2. stronger runtime result conventions
3. backend parity testing
4. bootstrap-safe compiler changes

That means the shared compiler/runtime work should be delivered once and consumed by both projects.

## Delivery Phases

### Phase 0: Shared Runtime Foundations

Before the framework itself, land the common language/runtime work:

1. `bytes`
2. clearer fallible-operation conventions
3. any shared text and encoding helpers

Exit criteria:

1. HTTP parser smoke tests can consume raw request bytes without lossy hacks.

### Phase 1: TCP Server Primitives

Extend Noema for inbound network service:

1. listener type
2. bind/listen builtin
3. accept builtin
4. request read loop using existing or extended socket reads

Exit criteria:

1. A tiny Noema echo or hello server can accept multiple sequential connections.

### Phase 2: HTTP Core

Build the protocol layer:

1. request line parser
2. header parser
3. response builder
4. status helpers
5. body handling

Exit criteria:

1. A browser or `curl` can request a page from a Noema server and receive valid HTTP responses.

### Phase 3: Routing and Handlers

Build a Flask-like programming model:

1. route registration
2. method matching
3. path params
4. handler dispatch
5. basic middleware hooks

Exit criteria:

1. A Noema app can define several routes and return distinct responses based on path and method.

### Phase 4: HTML Rendering

Build the template and rendering layer:

1. HTML escaping
2. tag builders
3. form helpers
4. layout helpers
5. reusable components via plain Noema functions

Exit criteria:

1. A multi-page server-rendered app can render forms, lists, and detail pages without ad hoc string assembly everywhere.

### Phase 5: Application Integration

Integrate Kiln Web with AshDB and Hearthlight use cases:

1. request-scoped database access
2. form post handling
3. validation helpers
4. redirect and flash-style message conventions
5. static asset serving for local development

Exit criteria:

1. Hearthlight can be bootstrapped as a real Noema web application.

## Testing Strategy

Kiln Web needs layered testing:

1. socket/listener smoke tests
2. raw HTTP parser fixtures
3. response serialization tests
4. route dispatch tests
5. browser or `curl` integration checks
6. compiler regression coverage for new server builtins

## First Implementation Targets

The first concrete deliverables after this plan are:

1. listener runtime smoke test
2. minimal HTTP request parser
3. `hello world` route example
4. tiny HTML helper library with escaping

These will prove that Noema can host a real application server instead of only acting as a client or batch tool.

## Definition of Success

Kiln Web succeeds when a Noema application can define routes, render HTML, accept browser requests locally, and grow into a complete server-rendered app without depending on another web framework.
