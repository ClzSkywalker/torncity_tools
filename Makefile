.PHONY: build rust-release godot-lsp

RUST_MANIFEST ?= rust/Cargo.toml
RUST_CRATE ?= bin
GODOT_BIN ?= godot
GODOT_PROJECT_DIR ?= godot

build:
	cargo build -p $(RUST_CRATE) --manifest-path $(RUST_MANIFEST)

clippy:
	cargo clippy -p $(RUST_CRATE) --manifest-path $(RUST_MANIFEST)

# 后台运行 godot 的 LSP 服务
godot-lsp:
	$(GODOT_BIN) --headless --path $(GODOT_PROJECT_DIR) --lsp-port 6005