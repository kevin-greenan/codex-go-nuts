# codex-lang

`codex-lang` is the custom programming language stack for this repository.

The first implementation is intentionally biased toward execution efficiency and portability instead of human-friendly syntax. The language runtime is a compact bytecode virtual machine written in C, and the bytecode format is designed to be stable, binary, and cheap to load.

## Design Direction

- Fast startup and low runtime overhead
- Portable execution across macOS, Linux, and Windows
- Deterministic binary format with explicit opcodes
- General-purpose control flow and arithmetic as the foundation for future higher-level language layers
- Bootstrap-friendly tooling so the language can gradually replace its own implementation stack over time

## Current Layout

- `docs/spec.md`: bytecode format and instruction set
- `src/cdxvm.c`: the reference virtual machine
- `tools/cdxasm.py`: a bootstrap assembler that emits `.cdx` binaries
- `examples/`: sample programs in the bootstrap assembly syntax
- `build/`: generated binaries and build outputs

## First Runtime Model

The initial language kernel is called `CDX`.

- Execution model: stack machine with indexed local slots
- Value type: signed 64-bit integers
- Program format: binary module with a small header and raw bytecode
- Control flow: conditional and unconditional relative jumps
- Tooling model: textual assembly for bootstrapping, binary modules for execution

This is enough to support loops, branching, arithmetic, counters, accumulators, and other low-level building blocks. Higher-level syntax, richer types, functions, memory management, and a native compiler can be layered on top once the kernel stabilizes.

## Quick Start

Build the VM:

```sh
make -C codex-lang
```

Assemble and run the sample programs:

```sh
python3 codex-lang/tools/cdxasm.py codex-lang/examples/hello.cdxasm codex-lang/build/hello.cdx
./codex-lang/build/cdxvm codex-lang/build/hello.cdx

python3 codex-lang/tools/cdxasm.py codex-lang/examples/sum_to_ten.cdxasm codex-lang/build/sum_to_ten.cdx
./codex-lang/build/cdxvm codex-lang/build/sum_to_ten.cdx
```

## Why This Shape

Because I am the primary consumer of the language, the bytecode does not need to optimize for human readability. The important constraints are:

- small runtime
- predictable execution
- easy cross-platform compilation
- room to grow into a fuller systems and application language

The bootstrap assembler exists only to let us develop the runtime before we have a self-hosted front end.
