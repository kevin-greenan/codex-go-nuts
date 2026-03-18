# codex-lang

`codex-lang` is the custom language stack for this repository, now centered on a compiled language named `Noema`.

`Noema` is not designed for public consumption. It is designed to be fast to compile, fast to run, easy for me to generate, and flexible enough to grow into the default implementation language for future projects in this repo, including the eventual self-hosted compiler path.

## What Changed

The earlier VM-based prototype has been retired.

The current direction is:

- compiler implemented in Rust
- Rust toolchain isolated in Docker so your laptop does not need a local Rust install
- `Noema` source compiled to optimized native binaries through generated C code plus the host C compiler
- a dense symbolic syntax optimized for generation and transformation
- richer language features aimed at compiler construction rather than just arithmetic demos
- raw TCP socket primitives so higher-level protocol stacks can be written in Noema

## Language Shape

`Noema` now uses a compact symbolic syntax. That is a better fit for how I generate, rewrite, and diff code mechanically.

Example:

```text
@fib(n: i64) -> i64 {
    ? (n <= 1) {
        ^ n;
    }
    | {
        ^ fib(n - 1) + fib(n - 2);
    }
}

@main() -> i64 {
    ! fib(10);
    ^ 0;
}
```

The current language now also supports:

- `%` declarations for AST and IR-like data
- `bool`, `text`, and `list<T>` types
- an opaque `socket` type for low-level networking
- string literals
- field access and list indexing
- struct literals and list literals
- compiler-oriented builtins such as `arg`, `arg_count`, `read_text`, `write_text`, `count`, `append`, `text_of`, and `i64_of`
- socket builtins such as `socket_open`, `socket_send`, `socket_recv`, and `socket_close`

Key design choices:

- short, unambiguous leading tokens for top-level forms and control flow
- cheap-to-generate bindings via `:=` and typed bindings via `::`
- explicit blocks and delimiters so whitespace is semantically irrelevant
- native compilation through Rust instead of VM execution
- syntax optimized for machine authorship over human ergonomics

## Current Layout

- `compiler/`: Rust implementation of the `Noema` compiler
- `container/`: Docker image definition for the Rust toolchain
- `bin/codexc`: wrapper that runs the compiler inside Docker
- `docs/noema.md`: language sketch and current grammar
- `examples/`: sample `Noema` programs
- `lib/`: reusable Noema source libraries
- `selfhost/`: Noema-written compiler experiments and bootstrap path
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

Compile through the experimental direct native backend on this Mac:

```sh
cd codex-lang
NOEMA_BACKEND=native-arm64 ./bin/codexc examples/hello.noe build/hello.native
./build/hello.native
```

A more compiler-shaped example:

```sh
cd codex-lang
./bin/codexc examples/frontend_demo.noe build/frontend_demo
./build/frontend_demo examples/hello.noe
```

A raw socket example:

```sh
cd codex-lang
./bin/codexc examples/socket_probe.noe build/socket_probe
./build/socket_probe 127.0.0.1 9001 ping
```

A reusable HTTP layer example:

```sh
cd codex-lang
./bin/codexc examples/http_get.noe build/http_get
./build/http_get http://127.0.0.1:9010/examples/hello.noe
```

A self-hosting bootstrap example:

```sh
cd codex-lang
./bin/codexc selfhost/mini_compiler.noe build/mini_compiler
./build/mini_compiler examples/mini_source.noe build/mini_source.generated.c
cc -O3 build/mini_source.generated.c -o build/mini_source.generated
./build/mini_source.generated
```

## Compilation Strategy

For now, `Noema` compiles to generated C source and then uses the host C compiler to produce an optimized native binary.

That gives us:

- native performance
- cross-platform compiler portability
- a pragmatic bootstrap path
- room to replace the C backend later if direct codegen becomes worthwhile

There is now also an experimental `native-arm64` backend for `arm64-apple-darwin`.

- It emits assembly directly instead of C.
- It currently supports a narrow scalar subset: `i64` functions, locals, arithmetic, comparisons, `?`, `~`, calls, and `!` on `i64`.
- The C backend remains the primary backend for the full language surface.

There is also now a first self-hosting bridge:

- `selfhost/mini_compiler.noe` is a compiler written in Noema
- it tokenizes, builds a small AST, parses a strict `i64` subset, and emits C
- that means Noema is now compiling Noema, even though Rust still provides the outer bootstrap compiler
- the next job is widening that Noema-written compiler until the Rust compiler becomes optional

## Intent

The long-term goal is still the same: everything else in this repository should eventually be written in this language stack.

This version now includes the first set of features needed to write parsers, AST builders, front-end utilities, and code generators directly in Noema, using a syntax that is intentionally denser and more symbolic than a conventional human-facing language.

The new self-hosting milestone matters because it shifts Noema from "compiler-friendly" to "already capable of implementing compiler phases itself." The language still needs a larger supported subset and eventually a native backend that can stand on its own, but the compiler logic is no longer confined to Rust.
