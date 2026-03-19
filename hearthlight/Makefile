TMP_ROOT ?= $(TMPDIR)
ifeq ($(TMP_ROOT),)
TMP_ROOT := /tmp
endif

SMOKE_ROOT := $(TMP_ROOT)/hearthlight-smoke
COMPILER_STAGE1 := $(SMOKE_ROOT)/noema_compiler.suite1
COMPILER_STAGE2 := $(SMOKE_ROOT)/noema_compiler.suite2
BUILD_DIR := build
DAEMON_BIN := $(BUILD_DIR)/dashboard_daemon.native
DB ?= /tmp/hearthlight.db
PORT ?= 9124

.PHONY: help compiler smoke daemon run clean

help:
	@echo "Hearthlight local commands"
	@echo "  make -C hearthlight smoke"
	@echo "  make -C hearthlight daemon"
	@echo "  make -C hearthlight run PORT=9124 DB=/tmp/hearthlight.db"

$(COMPILER_STAGE1):
	mkdir -p "$(SMOKE_ROOT)"
	cd .. && ./codex-lang/build/noema_compiler.pure4 codex-lang/selfhost/compiler.noe "$(COMPILER_STAGE1)" native-arm64

$(COMPILER_STAGE2): $(COMPILER_STAGE1)
	cd .. && "$(COMPILER_STAGE1)" codex-lang/selfhost/compiler.noe "$(COMPILER_STAGE2)" native-arm64

compiler: $(COMPILER_STAGE2)

smoke:
	zsh tools/run_direct_smokes.sh core

$(DAEMON_BIN): $(COMPILER_STAGE2)
	mkdir -p "$(BUILD_DIR)"
	cd .. && "$(COMPILER_STAGE2)" hearthlight/examples/dashboard_daemon.noe hearthlight/$(DAEMON_BIN) native-arm64

daemon: $(DAEMON_BIN)

run: $(DAEMON_BIN)
	@echo "Hearthlight on http://127.0.0.1:$(PORT)/"
	./$(DAEMON_BIN) "$(DB)" "$(PORT)"

clean:
	rm -rf "$(BUILD_DIR)"
