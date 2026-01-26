.PHONY: build rust-release check-export-presets export-presets godot-export-release package-release

RUST_MANIFEST ?= rust/Cargo.toml
RUST_CRATE ?= bin
GODOT_BIN ?= godot
GODOT_PROJECT_DIR ?= godot
EXPORT_PRESET ?= macOS
EXPORT_NAME ?= godot_rust_demo
EXPORT_DIR ?= dist
PROJECT_ROOT ?= .
EXPORT_DIR_ABS := $(abspath $(PROJECT_ROOT))/$(EXPORT_DIR)
EXPORT_EXT ?=
ifeq ($(EXPORT_PRESET),macOS)
EXPORT_EXT := .app
endif
EXPORT_PATH ?= $(EXPORT_DIR_ABS)/$(EXPORT_NAME)$(EXPORT_EXT)

build:
	cargo build -p $(RUST_CRATE) --manifest-path $(RUST_MANIFEST)

check-export-presets:
	@test -f $(GODOT_PROJECT_DIR)/export_presets.cfg || ( \
		echo "missing $(GODOT_PROJECT_DIR)/export_presets.cfg"; \
		echo "请先在 Godot 编辑器里创建导出预设（Project -> Export）"; \
		echo "或执行: make export-presets"; \
		exit 1; \
	)

export-presets:
	$(GODOT_BIN) --path $(GODOT_PROJECT_DIR)

rust-release:
	cargo build -p $(RUST_CRATE) --release --manifest-path $(RUST_MANIFEST)

# 导出 Godot 工程
godot-export-release: check-export-presets
	mkdir -p $(EXPORT_DIR_ABS)
	$(GODOT_BIN) --headless --path $(GODOT_PROJECT_DIR) --export-release "$(EXPORT_PRESET)" "$(EXPORT_PATH)"

package-release: rust-release godot-export-release

# 后台运行 godot 的 LSP 服务
godot-lsp:
	$(GODOT_BIN) --headless --path $(GODOT_PROJECT_DIR) --lsp-port 6005