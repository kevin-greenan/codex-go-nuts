# Noema

`Noema` is the language implemented by `codex-lang`.

Current compiler shape:

- Rust stage-1 bootstrap compiler
- canonical self-hosted source at [compiler_1.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler_1.noe)
- direct-native backend target: `native-arm64` on `arm64-apple-darwin`
- direct bootstrap check with a stage-2 rebuild of `compiler_1.noe`

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

The current production path is the direct-native backend:

- backend id: `native-arm64`
- target: `arm64-apple-darwin`
- output: arm64 assembly that is linked into a native executable

The bootstrap flow is:

1. use the Rust stage-1 compiler to build `compiler_1`
2. use `compiler_1` to build `noema_compiler.direct`
3. use `noema_compiler.direct` to rebuild `compiler_1.noe`

`make -C codex-lang test-direct-examples` compiles every retained example with the direct compiler, and `make -C codex-lang direct-bootstrap-check` verifies the stage-2 rebuild path.

## Self-Hosting Track

`Noema` now has a canonical bootstrap compiler written in `Noema` at [compiler_1.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler_1.noe).

The working path is:

1. `./bin/codexc selfhost/compiler_1.noe build/compiler_1`
2. `./build/compiler_1 selfhost/compiler_1.noe build/noema_compiler.direct native-arm64`
3. `./build/noema_compiler.direct selfhost/compiler_1.noe build/noema_compiler.stage2 native-arm64`

The direct-built compiler can compile every retained example, and the generated stage-1 and stage-2 compiler assembly matches exactly in the bootstrap check.

## Networking

Noema has a first-pass low-level TCP client layer.

- The language exposes an opaque `socket` type.
- Programs can open a TCP connection, send raw bytes, receive raw bytes, and close the connection.
- This stays below HTTP on purpose so protocol layers can be written in Noema later.

The runtime/library pieces exist, but they are not part of the retained direct-compiler example suite yet.

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
