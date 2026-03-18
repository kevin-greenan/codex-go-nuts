# Noema v2

`Noema` is the first native-compiled language for `codex-lang`.

## Objectives

- Use Rust as the implementation language for the compiler
- Produce optimized native binaries instead of bytecode
- Keep the syntax structured, concise, and expressive
- Favor implementation speed and machine efficiency over human-oriented polish
- Add the data-modeling and text-processing features needed for a future self-hosted compiler

## Syntax

The syntax is intentionally closer to a compact compiler IR language than a hand-authored scripting language:

- top-level functions start with `fn`
- top-level data declarations start with `type`
- blocks are delimited by `{` and `}`
- statements end with `;`
- control-flow conditions may be wrapped in `(...)`

### Function

```text
fn main() -> i64 {
    emit 42;
    return 0;
}
```

### Type

```text
type Token {
    kind: text;
    lexeme: text;
    line: i64;
}
```

### Variables

```text
let total = 0;
total = total + 1;
let tokens: list<Token> = [Token { kind: "word", lexeme: "emit", line: 1 }];
```

### Control Flow

```text
if (value > 10) {
    emit value;
}
else {
    emit 10;
}

while (index < 5) {
    index = index + 1;
}
```

## Supported Types

- `i64`
- `bool`
- `text`
- `socket`
- `list<T>`
- named `type` declarations
- `void`

## Supported Statements

- `let name = expr;`
- `let name: type = expr;`
- `name = expr;`
- `record.field = expr;`
- `emit expr;`
- `return expr;`
- `return;`
- `if (condition) { ... }`
- `else { ... }`
- `while (condition) { ... }`
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
- `socket_close(socket)` -> `bool`

## Backend

The compiler parses `Noema`, builds an AST, lowers it to C source, and the wrapper script invokes the host C compiler with optimization enabled.

This backend is deliberately practical:

- the compiler can stay small while the language evolves
- generated binaries are efficient and portable
- the Rust implementation stays containerized while final binaries remain host-native

## Why This Matters

With `shape`, `text`, and `list<T>`, Noema can now represent the core ingredients of a compiler:

- source text and token buffers
- AST nodes and typed intermediate data
- generated output buffers
- command-line and file-driven compilation workflows

It is not self-hosting yet, but it now has the structural features needed to start building real front-end and codegen components in Noema itself.

## Networking

Noema now has a first-pass low-level TCP client layer.

- The language exposes an opaque `socket` type.
- Programs can open a TCP connection, send raw bytes, receive raw bytes, and close the connection.
- This is intentionally lower-level than HTTP so future protocol implementations can be written in Noema on top of sockets instead of being baked into the runtime.

Example:

```text
fn main() -> i64 {
    let sock = socket_open("127.0.0.1", 9001);
    let sent = socket_send(sock, "ping");
    let reply = socket_recv(sock, 1024);
    emit text_of(sent);
    emit reply;
    let closed = socket_close(sock);
    if (not closed) {
        return 1;
    }
    return 0;
}
```
