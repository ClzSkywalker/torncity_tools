#!/bin/bash
# package-android.sh - 打包 Android 版本

set -e  # 遇到错误立即退出

# 获取脚本所在目录的上级目录作为项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PROJECT_ROOT:-$(dirname "$SCRIPT_DIR")}"

# 配置变量（可通过环境变量覆盖）
RUST_MANIFEST="${RUST_MANIFEST:-rust/Cargo.toml}"
RUST_CRATE="${RUST_CRATE:-bin}"
GODOT_BIN="${GODOT_BIN:-godot}"
GODOT_PROJECT_DIR="${GODOT_PROJECT_DIR:-godot}"
EXPORT_NAME="${EXPORT_NAME:-godot_rust_demo}"
EXPORT_DIR="${EXPORT_DIR:-dist}"

ANDROID_PRESET="${ANDROID_PRESET:-Android}"
ANDROID_EXT="${ANDROID_EXT:-.apk}"
ANDROID_SUBDIR="${ANDROID_SUBDIR:-android}"

EXPORT_DIR_ABS="${PROJECT_ROOT}/${EXPORT_DIR}/${ANDROID_SUBDIR}"
EXPORT_PATH="${EXPORT_PATH:-${EXPORT_DIR_ABS}/${EXPORT_NAME}${ANDROID_EXT}}"

# 切换到项目根目录
cd "$PROJECT_ROOT"

check_export_presets() {
    if [ ! -f "${GODOT_PROJECT_DIR}/export_presets.cfg" ]; then
        echo "missing ${GODOT_PROJECT_DIR}/export_presets.cfg"
        echo "请先在 Godot 编辑器里创建导出预设（Project -> Export）"
        echo "或执行: make export-presets"
        exit 1
    fi
}

rust_release() {
    if [ "${SKIP_RUST_BUILD}" = "1" ]; then
        echo ">>> 跳过 Rust 构建"
        return
    fi
    echo ">>> 构建 Rust release 版本..."
    cargo build -p "$RUST_CRATE" --release --manifest-path "$RUST_MANIFEST"
}

godot_export_release() {
    echo ">>> 检查导出预设..."
    check_export_presets

    echo ">>> 导出 Godot 工程..."
    mkdir -p "$EXPORT_DIR_ABS"
    "$GODOT_BIN" --headless --path "$GODOT_PROJECT_DIR" --export-release "$ANDROID_PRESET" "$EXPORT_PATH"
}

main() {
    echo "=== 开始打包 Android 版本 ==="
    echo "项目根目录: $PROJECT_ROOT"
    echo "导出预设: $ANDROID_PRESET"
    echo "导出路径: $EXPORT_PATH"
    echo ""

    rust_release
    godot_export_release

    echo ""
    echo "=== Android 打包完成 ==="
    echo "输出位置: $EXPORT_PATH"
}

main "$@"
