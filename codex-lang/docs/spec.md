# CDX Specification v0

`CDX` is the first executable core of `codex-lang`.

## Goals

- Portable binary execution format
- Low-complexity interpreter core
- Efficient integer operations
- A stable enough base to support later compilers and higher-level syntaxes

## Module Format

All integers are little-endian.

### Header

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 4 | Magic bytes: `CDX0` |
| 4 | 2 | Version, currently `1` |
| 6 | 2 | Reserved, currently `0` |
| 8 | 4 | Declared local slot count |
| 12 | 4 | Entry offset in code bytes |
| 16 | 4 | Code size in bytes |

The header is followed immediately by the code section.

## Machine Model

- Value stack of signed 64-bit integers
- Local slot array addressed by unsigned byte index
- Program counter into the code section
- Relative jumps encoded as signed 32-bit offsets from the next instruction

## Instruction Encoding

Each instruction begins with a 1-byte opcode.

Immediate forms:

- `CONST_I64`: opcode + signed 64-bit immediate
- `LOAD`: opcode + 1-byte slot index
- `STORE`: opcode + 1-byte slot index
- `JMP`, `JZ`, `JNZ`: opcode + signed 32-bit relative offset

## Opcode Set

| Opcode | Mnemonic | Effect |
| --- | --- | --- |
| `0x00` | `HALT` | Stop execution successfully |
| `0x01` | `CONST_I64 n` | Push immediate `n` |
| `0x02` | `LOAD i` | Push local slot `i` |
| `0x03` | `STORE i` | Pop into local slot `i` |
| `0x04` | `ADD_I64` | Pop `b`, pop `a`, push `a + b` |
| `0x05` | `SUB_I64` | Pop `b`, pop `a`, push `a - b` |
| `0x06` | `MUL_I64` | Pop `b`, pop `a`, push `a * b` |
| `0x07` | `DIV_I64` | Pop `b`, pop `a`, push `a / b` |
| `0x08` | `MOD_I64` | Pop `b`, pop `a`, push `a % b` |
| `0x09` | `EQ_I64` | Push `1` if equal, else `0` |
| `0x0A` | `LT_I64` | Push `1` if `a < b`, else `0` |
| `0x0B` | `GT_I64` | Push `1` if `a > b`, else `0` |
| `0x0C` | `JMP rel32` | Jump unconditionally |
| `0x0D` | `JZ rel32` | Pop condition, jump if zero |
| `0x0E` | `JNZ rel32` | Pop condition, jump if non-zero |
| `0x0F` | `DUP` | Duplicate top stack value |
| `0x10` | `DROP` | Discard top stack value |
| `0x11` | `PRINT_I64` | Pop and print value as decimal |

## Bootstrap Assembly

The bootstrap assembler uses a tiny text form:

- One instruction per line
- Labels end with `:`
- Comments start with `;`
- `.locals N` declares local slot count

Example:

```text
.locals 2

CONST_I64 0
STORE 0

CONST_I64 1
STORE 1

loop:
LOAD 1
CONST_I64 10
GT_I64
JNZ done

LOAD 0
LOAD 1
ADD_I64
STORE 0

LOAD 1
CONST_I64 1
ADD_I64
STORE 1

JMP loop

done:
LOAD 0
PRINT_I64
HALT
```

## Roadmap

Planned layers after v0:

1. Functions and call frames
2. Static data section and strings
3. Typed aggregates and heap management
4. Register-based or JIT-capable execution path
5. Higher-level surface syntax that compiles to CDX bytecode
