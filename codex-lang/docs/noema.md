# Noema v3

`Noema` is the native-compiled language core for `codex-lang`.

## Objectives

- Keep the implementation language in Rust
- Produce optimized native binaries instead of bytecode
- Bias the surface syntax toward machine generation, rewriting, and transformation
- Add the data and runtime features needed for a future self-hosted compiler

## Syntax

The syntax is intentionally dense and symbolic:

- top-level functions start with `@`
- top-level data declarations start with `%`
- blocks are delimited by `{` and `}`
- statements end with `;`
- new bindings use `:=`
- optional binding type annotations use `::`
- `!` emits
- `^` returns
- `?` branches
- `|` is the alternate branch
- `~` loops

### Function

```text
@main() -> i64 {
    ! 42;
    ^ 0;
}
```

### Type

```text
%Token {
    kind: text;
    lexeme: text;
    line: i64;
}
```

### Variables

```text
total := 0;
total = total + 1;
tokens :: list<Token> := [Token { kind: "word", lexeme: "emit", line: 1 }];
```

### Control Flow

```text
? (value > 10) {
    ! value;
}
| {
    ! 10;
}

~ (index < 5) {
    index = index + 1;
}
```

## Supported Types

- `i64`
- `bool`
- `text`
- `socket`
- `list<T>`
- named `%Type` declarations
- `void`

## Supported Statements

- `name := expr;`
- `name :: type := expr;`
- `name = expr;`
- `record.field = expr;`
- `! expr;`
- `^ expr;`
- `^;`
- `? (condition) { ... }`
- `| { ... }`
- `~ (condition) { ... }`
- expression statement `call(...);`

## Supported Expressions

- integer literals
- boolean literals: `true`, `false`
- string literals
- identifiers
- function calls
- unary `-`, `not`
- binary arithmetic: `+`, `-`, `*`, `/`, `%`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
- logical operators: `and`, `or`
- field access: `value.field`
- list indexing: `values[index]`
- record literals: `Node { tag: "root", arity: 1 }`
- list literals: `[1, 2, 3]`

## Builtins

- `arg(index)` -> `text`
- `arg_count()` -> `i64`
- `i64_of(text)` -> `i64`
- `read_text(path)` -> `text`
- `write_text(path, text)` -> `bool`
- `count(text_or_list)` -> `i64`
- `append(list, item)` -> `list<T>`
- `text_of(value)` -> `text`
- `socket_open(host, port)` -> `socket`
- `socket_send(socket, text)` -> `i64`
- `socket_recv(socket, limit)` -> `text`
- `socket_recv_all(socket)` -> `text`
- `socket_close(socket)` -> `bool`

## Includes

Noema source files can include other Noema source files with a leading `&`.

Example:

```text
& "../lib/http.noe";
```

Includes are expanded before parsing, which is the current library mechanism.

## Backend

The compiler parses `Noema`, builds an AST, lowers it to C source, and the wrapper script invokes the host C compiler with optimization enabled.

This backend is deliberately practical:

- the compiler can stay small while the language evolves
- generated binaries are efficient and portable
- the Rust implementation stays containerized while final binaries remain host-native

There is also an experimental direct native backend:

- backend id: `native-arm64`
- target: `arm64-apple-darwin`
- output: arm64 assembly plus a generated support C file when fallback is needed
- direct native subset: scalar `i64` programs with locals, arithmetic, comparisons, branching, loops, calls, and `!`, plus text literals/emits and runtime-backed arg/file/socket builtins
- fallback path: aggregate values such as shapes, field access, and lists currently still link through generated C support code

This is now strong enough to build and run every program in `examples/` through the native backend on `arm64-apple-darwin`, but it is still not a full replacement yet because some features are still lowered through C support code.

## Why This Matters

With `%` types, `text`, and `list<T>`, Noema can now represent the core ingredients of a compiler:

- source text and token buffers
- AST nodes and typed intermediate data
- generated output buffers
- command-line and file-driven compilation workflows

It is not self-hosting yet, but it now has the structural features needed to start building real front-end and codegen components in Noema itself.

## Self-Hosting Track

`Noema` now has a bootstrap compiler written in `Noema` at `selfhost/mini_compiler.noe`.

Parity expectation:

- the Rust compiler is the reference implementation
- the Noema-written compiler should be updated in parallel as features are added
- self-hosting validation is part of the normal workflow, not a later cleanup pass
- `make -C codex-lang selfhost-check` is the current self-hosting gate
- `make -C codex-lang parity-check` is the broader combined verification target

Current scope of that compiler:

- tokenizes source text itself
- builds a small AST for expressions and statements
- parses a scalar `i64`-only subset
- emits portable C

Current accepted subset:

- multiple `@name(...) -> i64 { ... }` functions
- `i64` parameters
- `name := expr;`
- `name = expr;`
- `! expr;`
- `^ expr;`
- `? (condition) { ... } | { ... }`
- `~ (condition) { ... }`
- function-call expressions
- expressions using integer literals, names, `()`, `+`, `-`, `*`, `/`, `<`, `<=`, `>`, `>=`, `==`, `!=`

This is still intentionally narrow relative to full Noema, but it is now large enough to compile `examples/series.noe` through the Noema-written compiler itself. The long-term plan is to keep widening this compiler until the Rust compiler is just a bootstrap artifact.

## Networking

Noema has a first-pass low-level TCP client layer.

- The language exposes an opaque `socket` type.
- Programs can open a TCP connection, send raw bytes, receive raw bytes, and close the connection.
- This stays below HTTP on purpose so protocol layers can be written in Noema later.

Example:

```text
@main() -> i64 {
    sock := socket_open("127.0.0.1", 9001);
    sent := socket_send(sock, "ping");
    reply := socket_recv(sock, 1024);
    ! text_of(sent);
    ! reply;
    closed := socket_close(sock);
    ? (not closed) {
        ^ 1;
    }
    ^ 0;
}
```
