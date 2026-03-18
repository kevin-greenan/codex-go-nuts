#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define CDX_MAGIC "CDX0"
#define CDX_VERSION 1
#define CDX_STACK_CAPACITY 65536

enum {
    OP_HALT = 0x00,
    OP_CONST_I64 = 0x01,
    OP_LOAD = 0x02,
    OP_STORE = 0x03,
    OP_ADD_I64 = 0x04,
    OP_SUB_I64 = 0x05,
    OP_MUL_I64 = 0x06,
    OP_DIV_I64 = 0x07,
    OP_MOD_I64 = 0x08,
    OP_EQ_I64 = 0x09,
    OP_LT_I64 = 0x0A,
    OP_GT_I64 = 0x0B,
    OP_JMP = 0x0C,
    OP_JZ = 0x0D,
    OP_JNZ = 0x0E,
    OP_DUP = 0x0F,
    OP_DROP = 0x10,
    OP_PRINT_I64 = 0x11,
};

typedef struct {
    uint32_t locals_count;
    uint32_t entry_offset;
    uint32_t code_size;
    uint8_t *code;
} Program;

typedef struct {
    int64_t data[CDX_STACK_CAPACITY];
    size_t size;
} Stack;

static uint16_t read_u16_le(const uint8_t *src) {
    return (uint16_t)src[0] | ((uint16_t)src[1] << 8);
}

static uint32_t read_u32_le(const uint8_t *src) {
    return (uint32_t)src[0] |
           ((uint32_t)src[1] << 8) |
           ((uint32_t)src[2] << 16) |
           ((uint32_t)src[3] << 24);
}

static int32_t read_i32_le(const uint8_t *src) {
    return (int32_t)read_u32_le(src);
}

static int64_t read_i64_le(const uint8_t *src) {
    uint64_t value = (uint64_t)src[0] |
                     ((uint64_t)src[1] << 8) |
                     ((uint64_t)src[2] << 16) |
                     ((uint64_t)src[3] << 24) |
                     ((uint64_t)src[4] << 32) |
                     ((uint64_t)src[5] << 40) |
                     ((uint64_t)src[6] << 48) |
                     ((uint64_t)src[7] << 56);
    return (int64_t)value;
}

static void fatal(const char *message) {
    fprintf(stderr, "cdxvm: %s\n", message);
    exit(1);
}

static void push(Stack *stack, int64_t value) {
    if (stack->size >= CDX_STACK_CAPACITY) {
        fatal("stack overflow");
    }
    stack->data[stack->size++] = value;
}

static int64_t pop(Stack *stack) {
    if (stack->size == 0) {
        fatal("stack underflow");
    }
    return stack->data[--stack->size];
}

static int64_t peek(const Stack *stack) {
    if (stack->size == 0) {
        fatal("stack underflow");
    }
    return stack->data[stack->size - 1];
}

static Program load_program(const char *path) {
    Program program = {0};
    FILE *file = fopen(path, "rb");
    uint8_t header[20];
    size_t read_size;

    if (file == NULL) {
        fprintf(stderr, "cdxvm: failed to open %s: %s\n", path, strerror(errno));
        exit(1);
    }

    read_size = fread(header, 1, sizeof(header), file);
    if (read_size != sizeof(header)) {
        fatal("file is too small to be a CDX module");
    }

    if (memcmp(header, CDX_MAGIC, 4) != 0) {
        fatal("invalid module magic");
    }

    if (read_u16_le(header + 4) != CDX_VERSION) {
        fatal("unsupported module version");
    }

    program.locals_count = read_u32_le(header + 8);
    program.entry_offset = read_u32_le(header + 12);
    program.code_size = read_u32_le(header + 16);

    if (program.entry_offset > program.code_size) {
        fatal("entry offset is outside the code section");
    }

    program.code = (uint8_t *)malloc(program.code_size);
    if (program.code == NULL) {
        fatal("failed to allocate code buffer");
    }

    read_size = fread(program.code, 1, program.code_size, file);
    if (read_size != program.code_size) {
        fatal("failed to read complete code section");
    }

    fclose(file);
    return program;
}

static void free_program(Program *program) {
    free(program->code);
    program->code = NULL;
}

static void ensure_bytes(const Program *program, uint32_t pc, uint32_t needed) {
    if (pc > program->code_size || needed > program->code_size - pc) {
        fatal("instruction extends past end of code");
    }
}

static uint32_t checked_target(uint32_t base, int32_t relative, uint32_t code_size) {
    int64_t target = (int64_t)base + (int64_t)relative;
    if (target < 0 || target > (int64_t)code_size) {
        fatal("jump target is outside the code section");
    }
    return (uint32_t)target;
}

static int run_program(const Program *program) {
    Stack stack = {0};
    int64_t *locals = NULL;
    uint32_t pc = program->entry_offset;

    if (program->locals_count > 0) {
        locals = (int64_t *)calloc(program->locals_count, sizeof(int64_t));
        if (locals == NULL) {
            fatal("failed to allocate local slots");
        }
    }

    while (pc < program->code_size) {
        uint8_t opcode = program->code[pc++];

        switch (opcode) {
            case OP_HALT:
                free(locals);
                return 0;

            case OP_CONST_I64: {
                int64_t value;
                ensure_bytes(program, pc, 8);
                value = read_i64_le(program->code + pc);
                pc += 8;
                push(&stack, value);
                break;
            }

            case OP_LOAD: {
                uint8_t slot;
                ensure_bytes(program, pc, 1);
                slot = program->code[pc++];
                if (slot >= program->locals_count) {
                    fatal("local slot out of range");
                }
                push(&stack, locals[slot]);
                break;
            }

            case OP_STORE: {
                uint8_t slot;
                ensure_bytes(program, pc, 1);
                slot = program->code[pc++];
                if (slot >= program->locals_count) {
                    fatal("local slot out of range");
                }
                locals[slot] = pop(&stack);
                break;
            }

            case OP_ADD_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                push(&stack, a + b);
                break;
            }

            case OP_SUB_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                push(&stack, a - b);
                break;
            }

            case OP_MUL_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                push(&stack, a * b);
                break;
            }

            case OP_DIV_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                if (b == 0) {
                    fatal("division by zero");
                }
                push(&stack, a / b);
                break;
            }

            case OP_MOD_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                if (b == 0) {
                    fatal("modulo by zero");
                }
                push(&stack, a % b);
                break;
            }

            case OP_EQ_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                push(&stack, a == b ? 1 : 0);
                break;
            }

            case OP_LT_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                push(&stack, a < b ? 1 : 0);
                break;
            }

            case OP_GT_I64: {
                int64_t b = pop(&stack);
                int64_t a = pop(&stack);
                push(&stack, a > b ? 1 : 0);
                break;
            }

            case OP_JMP: {
                int32_t offset;
                ensure_bytes(program, pc, 4);
                offset = read_i32_le(program->code + pc);
                pc += 4;
                pc = checked_target(pc, offset, program->code_size);
                break;
            }

            case OP_JZ: {
                int32_t offset;
                int64_t condition;
                ensure_bytes(program, pc, 4);
                offset = read_i32_le(program->code + pc);
                pc += 4;
                condition = pop(&stack);
                if (condition == 0) {
                    pc = checked_target(pc, offset, program->code_size);
                }
                break;
            }

            case OP_JNZ: {
                int32_t offset;
                int64_t condition;
                ensure_bytes(program, pc, 4);
                offset = read_i32_le(program->code + pc);
                pc += 4;
                condition = pop(&stack);
                if (condition != 0) {
                    pc = checked_target(pc, offset, program->code_size);
                }
                break;
            }

            case OP_DUP:
                push(&stack, peek(&stack));
                break;

            case OP_DROP:
                (void)pop(&stack);
                break;

            case OP_PRINT_I64:
                printf("%" PRId64 "\n", pop(&stack));
                break;

            default:
                fatal("unknown opcode");
        }
    }

    free(locals);
    fatal("program terminated without HALT");
    return 1;
}

int main(int argc, char **argv) {
    Program program;
    int status;

    if (argc != 2) {
        fprintf(stderr, "usage: %s <module.cdx>\n", argv[0]);
        return 1;
    }

    program = load_program(argv[1]);
    status = run_program(&program);
    free_program(&program);
    return status;
}
