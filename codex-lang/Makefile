CC ?= cc
CFLAGS ?= -O3 -std=c11 -Wall -Wextra -Wpedantic

BUILD_DIR := build
SRC_DIR := src
TOOLS_DIR := tools
EXAMPLES_DIR := examples

VM := $(BUILD_DIR)/cdxvm

.PHONY: all clean examples

all: $(VM)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(VM): $(SRC_DIR)/cdxvm.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -o $(VM) $(SRC_DIR)/cdxvm.c

examples: $(VM) $(BUILD_DIR)/hello.cdx $(BUILD_DIR)/sum_to_ten.cdx

$(BUILD_DIR)/hello.cdx: $(EXAMPLES_DIR)/hello.cdxasm | $(BUILD_DIR)
	python3 $(TOOLS_DIR)/cdxasm.py $(EXAMPLES_DIR)/hello.cdxasm $(BUILD_DIR)/hello.cdx

$(BUILD_DIR)/sum_to_ten.cdx: $(EXAMPLES_DIR)/sum_to_ten.cdxasm | $(BUILD_DIR)
	python3 $(TOOLS_DIR)/cdxasm.py $(EXAMPLES_DIR)/sum_to_ten.cdxasm $(BUILD_DIR)/sum_to_ten.cdx

clean:
	rm -rf $(BUILD_DIR)
