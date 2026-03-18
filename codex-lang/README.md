# codex-lang

`codex-lang` is the custom language stack for this repository, now centered on a compiled language named `Noema`.

`Noema` is not designed for public consumption. It is designed to be fast to compile, fast to run, easy for me to generate, and flexible enough to grow into the default implementation language for future projects in this repo.

## What Changed

The earlier VM-based prototype has been retired.

The current direction is:

- compiler implemented in Rust
- Rust toolchain isolated in Docker so your laptop does not need a local Rust install
- `Noema` source compiled to optimized native binaries through generated C code plus the host C compiler
- higher-level syntax that feels closer to Rust and Python than assembly

## Language Shape

`Noema` uses indentation for blocks and explicit statements for clarity.

Example:

```text
loom fib(n: i64) -> i64:
    if n <= 1:
        return n;
    else:
        return fib(n - 1) + fib(n - 2);

loom main() -> i64:
    emit fib(10);
    return 0;
```

Key design choices:

- structured functions with typed parameters and return values
- descriptive control flow: `if`, `else`, `while`, `return`
- mutable local bindings with `let`
- native compilation through Rust instead of VM execution
- a syntax that is readable enough to inspect, but still free to evolve around machine-first usage

## Current Layout

- `compiler/`: Rust implementation of the `Noema` compiler
- `container/`: Docker image definition for the Rust toolchain
- `bin/codexc`: wrapper that runs the compiler inside Docker
- `docs/noema.md`: language sketch and current grammar
- `examples/`: sample `Noema` programs
- `build/`: generated outputs

## Quick Start

Build the container image:

```sh
make -C codex-lang image
```

Compile a `Noema` program into a native executable:

```sh
cd codex-lang
./bin/codexc examples/hello.noe build/hello
./build/hello
```

## Compilation Strategy

For now, `Noema` compiles to generated C source and then uses the host C compiler to produce an optimized native binary.

That gives us:

- native performance
- cross-platform compiler portability
- a pragmatic bootstrap path
- room to replace the C backend later if direct codegen becomes worthwhile

## Intent

The long-term goal is still the same: everything else in this repository should eventually be written in this language stack.

This version simply starts from a better foundation for that goal.
