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

A stage-1 bootstrap example:

```sh
cd codex-lang
./bin/codexc selfhost/compiler_1.noe build/compiler_1
./build/compiler_1 examples/mini_source.noe build/mini_source.generated
./build/mini_source.generated
```

A wider self-hosted example:

```sh
cd codex-lang
./bin/codexc selfhost/compiler_1.noe build/compiler_1
./build/compiler_1 examples/series.noe build/series.selfhost
./build/series.selfhost
```

## Compilation Strategy

For now, `Noema` compiles to generated C source and then uses the host C compiler to produce an optimized native binary.

That gives us:

- native performance
- cross-platform compiler portability
- a pragmatic bootstrap path
- room to replace the C backend later if direct codegen becomes worthwhile

There is now also an experimental `native-arm64` backend for `arm64-apple-darwin`.

- It emits arm64 assembly directly and links a generated support C file when features are not natively lowered yet.
- Scalar `i64` codepaths run through direct native codegen.
- Text literals, text emission, argument/file/socket builtins, and simple text/socket programs now also lower through the native backend via native-handle runtime calls.
- Higher-level aggregate features such as `%` records, fields, and `list<T>` still fall back to generated C support code inside the same native build.
- That means every program under `examples/` can now be built and run through `NOEMA_BACKEND=native-arm64` on this Mac, even though the fallback surface is still larger than we want.

## Parity Rule

The Rust compiler and the Noema-written compiler are intended to move in parallel.

- New language/compiler features should be reflected in both compiler tracks, not just the Rust implementation.
- `make -C codex-lang selfhost-check` is the baseline proof that Noema still compiles Noema.
- `make -C codex-lang parity-check` is the combined workflow target for self-hosted and native-backend verification.

At the moment the Noema-written compiler is still behind the full Rust compiler surface, so feature parity is not complete yet. But from here forward, parity work is part of the definition of done rather than follow-up work.

There is now a canonical self-hosted bootstrap compiler:

- `selfhost/compiler_1.noe` is the Noema-written compiler artifact
- the Rust stage-1 compiler builds it into `compiler_1`
- `compiler_1` then rebuilds itself into `noema_compiler`
- that means Noema is now compiling Noema through a binary-to-binary bootstrap workflow, even though the internal backend is still C-backed today

Current self-hosted subset:

- multiple `@fn(...) -> i64 { ... }` functions
- `i64`, `bool`, `text`, and `socket` values in the current bootstrap surface
- typed function parameters and returns for the currently supported scalar/runtime-backed types
- `:=` bindings and `=` assignment
- typed `::` bindings
- `!` and `^`
- `?` / `|` and `~`
- function calls
- integer arithmetic and scalar comparisons
- text literals, `+`, `==`, `!=`, and `not`
- runtime-backed builtins such as `arg`, `arg_count`, `read_text`, `write_text`, `count`, `find`, `slice`, `text_of`, `i64_of`, and the low-level socket builtins
- `%` declarations, struct literals, field access, list literals, and indexing

That subset is now large enough for [compiler_1.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler_1.noe) to compile [series.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/series.noe), [selfhost_text.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/selfhost_text.noe), [socket_probe.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/socket_probe.noe), and [frontend_demo.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/frontend_demo.noe) end to end.

## Bootstrap Workflow

There is now an explicit staged bootstrap path:

1. The Rust stage-1 compiler in [main.rs](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/compiler/src/main.rs) builds [compiler_1.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler_1.noe) into the `compiler_1` binary.
2. The resulting `compiler_1` binary compiles [compiler_1.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler_1.noe) again into the final `noema_compiler` binary.
3. The final `noema_compiler` binary can then compile other Noema programs.

Current commands:

```sh
cd codex-lang
./bin/codexc selfhost/compiler_1.noe build/compiler_1
./build/compiler_1 selfhost/compiler_1.noe build/noema_compiler
./build/noema_compiler examples/frontend_demo.noe build/frontend_demo.bootstrap
./build/frontend_demo.bootstrap examples/hello.noe
```

Or via make:

```sh
make -C codex-lang bootstrap-check
```

Today `compiler_1` produces final binaries by generating a `.generated.c` file internally and invoking the host C compiler itself. So the bootstrap deliverable now exists as a binary-to-binary workflow, but the Noema-written compiler backend is still C-backed internally rather than a pure direct native emitter.

There is now also a first direct-native path inside [compiler_1.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler_1.noe):

```sh
cd codex-lang
./bin/codexc selfhost/compiler_1.noe build/compiler_1
./build/compiler_1 examples/hello.noe build/hello.direct native-arm64
./build/hello.direct
./build/compiler_1 examples/mini_source.noe build/mini_source.direct native-arm64
./build/mini_source.direct
```

That direct backend currently targets `arm64-apple-darwin` and only supports a narrow scalar subset:

- a single `@main() -> i64`
- linear `let` / `assign` / `!` / `^`
- local `i64` values
- integer literals and `+` / `-` / `*`

It is a real no-C direct path for those programs, but it is not yet wide enough to compile `compiler_1.noe` itself.

## Intent

The long-term goal is still the same: everything else in this repository should eventually be written in this language stack.

This version now includes the first set of features needed to write parsers, AST builders, front-end utilities, and code generators directly in Noema, using a syntax that is intentionally denser and more symbolic than a conventional human-facing language.

The new self-hosting milestone matters because it shifts Noema from "compiler-friendly" to "already capable of implementing compiler phases itself." The language still needs a larger supported subset and eventually a native backend that can stand on its own, but the compiler logic is no longer confined to Rust.
