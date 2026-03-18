# Noema v0

`Noema` is the first native-compiled language for `codex-lang`.

## Objectives

- Use Rust as the implementation language for the compiler
- Produce optimized native binaries instead of bytecode
- Keep the syntax structured, concise, and expressive
- Favor implementation speed and machine efficiency over human-oriented polish

## Syntax

The syntax is intentionally close to a Rust and Python hybrid:

- top-level functions start with `loom`
- blocks are introduced with `:`
- blocks are defined by indentation
- statements end with `;` unless they open a block

### Function

```text
loom main() -> i64:
    emit 42;
    return 0;
```

### Variables

```text
let total = 0;
total = total + 1;
```

### Control Flow

```text
if value > 10:
    emit value;
else:
    emit 10;

while index < 5:
    index = index + 1;
```

## Supported Types

- `i64`
- `void`

## Supported Statements

- `let name = expr;`
- `name = expr;`
- `emit expr;`
- `return expr;`
- `return;`
- `if condition:`
- `else:`
- `while condition:`
- expression statement `call(...);`

## Supported Expressions

- integer literals
- identifiers
- function calls
- unary `-`
- binary arithmetic: `+`, `-`, `*`, `/`, `%`
- comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`

## Backend

The compiler parses `Noema`, builds an AST, lowers it to C source, and the wrapper script invokes the host C compiler with optimization enabled.

This backend is deliberately practical:

- the compiler can stay small while the language evolves
- generated binaries are efficient and portable
- the Rust implementation stays containerized while final binaries remain host-native
