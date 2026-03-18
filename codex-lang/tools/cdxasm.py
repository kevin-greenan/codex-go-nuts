#!/usr/bin/env python3

import struct
import sys
from dataclasses import dataclass
from typing import Dict, List, Optional

MAGIC = b"CDX0"
VERSION = 1

OPCODES = {
    "HALT": 0x00,
    "CONST_I64": 0x01,
    "LOAD": 0x02,
    "STORE": 0x03,
    "ADD_I64": 0x04,
    "SUB_I64": 0x05,
    "MUL_I64": 0x06,
    "DIV_I64": 0x07,
    "MOD_I64": 0x08,
    "EQ_I64": 0x09,
    "LT_I64": 0x0A,
    "GT_I64": 0x0B,
    "JMP": 0x0C,
    "JZ": 0x0D,
    "JNZ": 0x0E,
    "DUP": 0x0F,
    "DROP": 0x10,
    "PRINT_I64": 0x11,
}

INSTRUCTION_SIZES = {
    "HALT": 1,
    "CONST_I64": 9,
    "LOAD": 2,
    "STORE": 2,
    "ADD_I64": 1,
    "SUB_I64": 1,
    "MUL_I64": 1,
    "DIV_I64": 1,
    "MOD_I64": 1,
    "EQ_I64": 1,
    "LT_I64": 1,
    "GT_I64": 1,
    "JMP": 5,
    "JZ": 5,
    "JNZ": 5,
    "DUP": 1,
    "DROP": 1,
    "PRINT_I64": 1,
}

NO_ARG = {
    "HALT",
    "ADD_I64",
    "SUB_I64",
    "MUL_I64",
    "DIV_I64",
    "MOD_I64",
    "EQ_I64",
    "LT_I64",
    "GT_I64",
    "DUP",
    "DROP",
    "PRINT_I64",
}

SLOT_ARG = {"LOAD", "STORE"}
INT_ARG = {"CONST_I64"}
JUMP_ARG = {"JMP", "JZ", "JNZ"}


@dataclass
class ParsedInstruction:
    name: str
    arg: Optional[str]
    offset: int
    line_no: int


def strip_comment(line: str) -> str:
    return line.split(";", 1)[0].strip()


def parse_source(path: str):
    locals_count = 0
    labels = {}
    instructions = []
    offset = 0

    with open(path, "r", encoding="utf-8") as handle:
        for line_no, raw_line in enumerate(handle, start=1):
            line = strip_comment(raw_line)
            if not line:
                continue

            if line.startswith(".locals"):
                parts = line.split()
                if len(parts) != 2:
                    raise ValueError(f"{path}:{line_no}: invalid .locals directive")
                locals_count = int(parts[1], 10)
                if locals_count < 0 or locals_count > 255:
                    raise ValueError(f"{path}:{line_no}: .locals must be between 0 and 255")
                continue

            if line.endswith(":"):
                label = line[:-1].strip()
                if not label:
                    raise ValueError(f"{path}:{line_no}: empty label")
                if label in labels:
                    raise ValueError(f"{path}:{line_no}: duplicate label {label}")
                labels[label] = offset
                continue

            parts = line.split(None, 1)
            name = parts[0].upper()
            arg = parts[1].strip() if len(parts) == 2 else None

            if name not in OPCODES:
                raise ValueError(f"{path}:{line_no}: unknown instruction {name}")

            instructions.append(ParsedInstruction(name=name, arg=arg, offset=offset, line_no=line_no))
            offset += INSTRUCTION_SIZES[name]

    return locals_count, labels, instructions


def assemble_instruction(path: str, parsed: ParsedInstruction, labels: Dict[str, int]) -> bytes:
    opcode = OPCODES[parsed.name]

    if parsed.name in NO_ARG:
        if parsed.arg is not None:
            raise ValueError(f"{path}:{parsed.line_no}: {parsed.name} does not take an argument")
        return bytes([opcode])

    if parsed.arg is None:
        raise ValueError(f"{path}:{parsed.line_no}: {parsed.name} requires an argument")

    if parsed.name in SLOT_ARG:
        slot = int(parsed.arg, 10)
        if slot < 0 or slot > 255:
            raise ValueError(f"{path}:{parsed.line_no}: slot index must be between 0 and 255")
        return bytes([opcode, slot])

    if parsed.name in INT_ARG:
        value = int(parsed.arg, 10)
        return bytes([opcode]) + struct.pack("<q", value)

    if parsed.name in JUMP_ARG:
        if parsed.arg not in labels:
            raise ValueError(f"{path}:{parsed.line_no}: unknown label {parsed.arg}")
        target = labels[parsed.arg]
        next_offset = parsed.offset + INSTRUCTION_SIZES[parsed.name]
        relative = target - next_offset
        return bytes([opcode]) + struct.pack("<i", relative)

    raise ValueError(f"{path}:{parsed.line_no}: unsupported instruction {parsed.name}")


def assemble(input_path: str, output_path: str) -> None:
    locals_count, labels, instructions = parse_source(input_path)
    code = bytearray()

    for parsed in instructions:
        code.extend(assemble_instruction(input_path, parsed, labels))

    header = bytearray()
    header.extend(MAGIC)
    header.extend(struct.pack("<H", VERSION))
    header.extend(struct.pack("<H", 0))
    header.extend(struct.pack("<I", locals_count))
    header.extend(struct.pack("<I", 0))
    header.extend(struct.pack("<I", len(code)))

    with open(output_path, "wb") as handle:
        handle.write(header)
        handle.write(code)


def main(argv: List[str]) -> int:
    if len(argv) != 3:
        print(f"usage: {argv[0]} <input.cdxasm> <output.cdx>", file=sys.stderr)
        return 1

    try:
        assemble(argv[1], argv[2])
    except Exception as exc:
        print(f"cdxasm: {exc}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
