#!/bin/bash
# package-desktop.sh - 打包桌面版本

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

# 根据当前操作系统自动检测导出预设和扩展名
# 注意：预设名称需要与 godot/export_presets.cfg 中的 name 字段匹配
detect_platform() {
    case "$(uname -s)" in
        Darwin)
            echo "macOS"
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            echo "Windows Desktop"
            ;;
        Linux)
            echo "Linux"
            ;;
        *)
            echo "Unknown"
            ;;
    esac
}

# 设置导出预设（优先使用环境变量，否则自动检测）
if [ -z "$EXPORT_PRESET" ]; then
    EXPORT_PRESET="$(detect_platform)"
    echo ">>> 检测到操作系统，使用导出预设: $EXPORT_PRESET"
fi

# 根据导出预设设置文件扩展名和平台子目录
EXPORT_EXT=""
PLATFORM_SUBDIR=""
case "$EXPORT_PRESET" in
    macOS)
        EXPORT_EXT=".app"
        PLATFORM_SUBDIR="macos"
        ;;
    "Windows Desktop"|Windows)
        EXPORT_EXT=".exe"
        PLATFORM_SUBDIR="windows"
        ;;
    Linux)
        EXPORT_EXT=""
        PLATFORM_SUBDIR="linux"
        ;;
    *)
        PLATFORM_SUBDIR="other"
        ;;
esac

# 更新导出目录（加入平台子目录）
EXPORT_DIR_ABS="${PROJECT_ROOT}/${EXPORT_DIR}/${PLATFORM_SUBDIR}"
EXPORT_PATH="${EXPORT_PATH:-${EXPORT_DIR_ABS}/${EXPORT_NAME}${EXPORT_EXT}}"

# 切换到项目根目录
cd "$PROJECT_ROOT"

# 检查导出预设文件是否存在
check_export_presets() {
    if [ ! -f "${GODOT_PROJECT_DIR}/export_presets.cfg" ]; then
        echo "missing ${GODOT_PROJECT_DIR}/export_presets.cfg"
        echo "请先在 Godot 编辑器里创建导出预设（Project -> Export）"
        echo "或执行: make export-presets"
        exit 1
    fi
}

# 构建 Rust release 版本
rust_release() {
    echo ">>> 构建 Rust release 版本..."
    cargo build -p "$RUST_CRATE" --release --manifest-path "$RUST_MANIFEST"
}

# 导出 Godot 工程
godot_export_release() {
    echo ">>> 检查导出预设..."
    check_export_presets

    echo ">>> 导出 Godot 工程..."
    mkdir -p "$EXPORT_DIR_ABS"
    "$GODOT_BIN" --headless --path "$GODOT_PROJECT_DIR" --export-release "$EXPORT_PRESET" "$EXPORT_PATH"
}

# 主流程：package-desktop
main() {
    echo "=== 开始打包桌面版本 ==="
    echo "项目根目录: $PROJECT_ROOT"
    echo "导出预设: $EXPORT_PRESET"
    echo "导出路径: $EXPORT_PATH"
    echo ""

    rust_release
    godot_export_release

    echo ""
    echo "=== 桌面打包完成 ==="
    echo "输出位置: $EXPORT_PATH"
}

main "$@"
