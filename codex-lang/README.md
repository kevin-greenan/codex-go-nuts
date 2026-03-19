# codex-lang

`codex-lang` is the current Noema toolchain.

The working model is:

- a Rust stage-1 compiler in Docker
- a canonical Noema compiler source at [compiler.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/selfhost/compiler.noe)
- a direct-native `native-arm64` backend for this Mac target
- a bootstrap path where Noema compiles Noema

Noema is machine-first. The syntax is compact on purpose so it is cheap for me to generate, transform, and diff.

```text
@fib(n: i64) -> i64 {
    ? (n <= 1) {
        ^ n;
    }
    | {
        ^ fib(n - 1) + fib(n - 2);
    }
}
```

## Current State

- The canonical language/compiler pair is `selfhost/compiler.noe` plus the Rust stage-1 bootstrap compiler.
- The direct backend target is `arm64-apple-darwin`.
- The direct compiler can build every retained example under [examples](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples).
- The direct-built compiler can rebuild `compiler.noe`; the generated stage-1 and stage-2 compiler assembly matches exactly.

## Layout

- `compiler/`: Rust stage-1 compiler
- `container/`: Docker image for the Rust toolchain
- `bin/codexc`: wrapper for running the stage-1 compiler without a local Rust install
- `selfhost/`: canonical Noema compiler source
- `lib/`: reusable Noema libraries
- `examples/`: feature-focused Noema programs
- `tests/`: direct-compiler test harnesses
- `docs/noema.md`: current language reference

## Bootstrap Flow

1. Build `compiler` with the Rust stage-1 compiler.
2. Use `compiler` to build `noema_compiler.direct`.
3. Use `noema_compiler.direct` to rebuild `compiler.noe` again and check for a fixed point.

```sh
cd codex-lang
make bootstrap-direct
make direct-bootstrap-check
```

## Example Set

- [hello.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/hello.noe): minimal program
- [exit_status.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/exit_status.noe): arithmetic and exit codes
- [control_flow.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/control_flow.noe): functions, branches, loops, recursion
- [records_and_lists.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/records_and_lists.noe): `%` records and `list<T>`
- [text_pipeline.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/text_pipeline.noe): text, file I/O, slicing
- [compiler_frontend.noe](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/examples/compiler_frontend.noe): compiler-shaped records and text rendering
## Checks

Build the bootstrap compiler:

```sh
make -C codex-lang bootstrap-compiler
```

Build the direct compiler:

```sh
make -C codex-lang bootstrap-direct
```

Compile every example with the direct compiler and smoke-run the local ones:

```sh
make -C codex-lang test-direct-examples
```

Rebuild the compiler with the direct compiler and verify the bootstrap fixed point:

```sh
make -C codex-lang direct-bootstrap-check
```

## Language Surface

Current core surface:

- top-level `@` functions and `%` record declarations
- `i64`, `bool`, `text`, `socket`, `list<T>`, and `void`
- `:=`, `::`, `=`, `!`, `^`, `?`, `|`, `~`
- record literals, field access, list literals, indexing
- builtins: `arg`, `arg_count`, `read_text`, `write_text`, `count`, `append`, `find`, `slice`, `text_of`, `i64_of`, `socket_open`, `socket_send`, `socket_recv`, `socket_recv_all`, `socket_close`
- include expansion via leading `&`

The reference grammar and examples live in [noema.md](/Users/kevin/Documents/Projects/AI/codex-go-nuts/codex-lang/docs/noema.md).

The current bootstrap milestone matters because Noema now has a canonical self-hosted compiler artifact under the final name, and the direct compiler can rebuild it consistently.
