#!/bin/bash
# package-android.sh - 打包 Android 版本
# 支持 Windows (MINGW/Git Bash), macOS, Linux

set -e

# =============================================================================
# 配置变量
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="${PROJECT_ROOT:-$(dirname "$SCRIPT_DIR")}"

# 构建配置
RUST_MANIFEST="${RUST_MANIFEST:-rust/Cargo.toml}"
RUST_CRATE="${RUST_CRATE:-bin}"
GODOT_BIN="${GODOT_BIN:-godot}"
GODOT_PROJECT_DIR="${GODOT_PROJECT_DIR:-godot}"
EXPORT_NAME="${EXPORT_NAME:-godot_rust_demo}"
EXPORT_DIR="${EXPORT_DIR:-dist}"

# Android 配置
ANDROID_PRESET="${ANDROID_PRESET:-Android}"
ANDROID_EXT="${ANDROID_EXT:-.apk}"
ANDROID_SUBDIR="${ANDROID_SUBDIR:-android}"
ANDROID_TARGET="${ANDROID_TARGET:-aarch64-linux-android}"
ANDROID_API="${ANDROID_API:-21}"

# 签名配置
DEBUG_KEYSTORE="${DEBUG_KEYSTORE:-${PROJECT_ROOT}/debug.keystore}"
DEBUG_KEYSTORE_ALIAS="${DEBUG_KEYSTORE_ALIAS:-androiddebugkey}"
DEBUG_KEYSTORE_PASS="${DEBUG_KEYSTORE_PASS:-android}"

# 导出路径
EXPORT_DIR_ABS="${PROJECT_ROOT}/${EXPORT_DIR}/${ANDROID_SUBDIR}"
EXPORT_PATH="${EXPORT_PATH:-${EXPORT_DIR_ABS}/${EXPORT_NAME}${ANDROID_EXT}}"

# 调试模式（设置 VERBOSE=1 开启详细输出）
VERBOSE="${VERBOSE:-0}"

cd "$PROJECT_ROOT"

# =============================================================================
# 工具函数
# =============================================================================

log() { echo ">>> $*"; }
log_detail() { [[ "$VERBOSE" == "1" ]] && echo "    $*" || true; }
log_error() { echo "错误: $*" >&2; }

# 检测主机操作系统
detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux";;
        Darwin*) echo "mac";;
        MINGW*|MSYS*|CYGWIN*) echo "windows";;
        *) echo "unknown";;
    esac
}

# 获取 NDK 主机平台标识
get_ndk_host() {
    case "$(uname -s)" in
        Linux*)  echo "linux-x86_64";;
        Darwin*) 
            # Mac 支持 arm64 和 x86_64
            if [[ "$(uname -m)" == "arm64" ]]; then
                echo "darwin-x86_64"  # NDK 仍使用 x86_64 目录（Rosetta 兼容）
            else
                echo "darwin-x86_64"
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows-x86_64";;
        *) echo "unknown";;
    esac
}

# 获取 NDK 工具链前缀
get_toolchain_prefix() {
    case "$ANDROID_TARGET" in
        aarch64-linux-android) echo "aarch64-linux-android";;
        armv7-linux-androideabi) echo "armv7a-linux-androideabi";;
        x86_64-linux-android) echo "x86_64-linux-android";;
        i686-linux-android) echo "i686-linux-android";;
        *) echo "";;
    esac
}

# 查找最新版本的 build-tools 目录
find_build_tools_dir() {
    [[ -z "$ANDROID_HOME" ]] && return
    local base="$ANDROID_HOME/build-tools"
    [[ ! -d "$base" ]] && return
    local version=$(ls -1 "$base" 2>/dev/null | sort -V | tail -1)
    [[ -n "$version" ]] && echo "$base/$version"
}

# 查找 zipalign
find_zipalign() {
    command -v zipalign &>/dev/null && { echo "zipalign"; return; }
    local dir=$(find_build_tools_dir)
    [[ -n "$dir" && -f "$dir/zipalign" ]] && { echo "$dir/zipalign"; return; }
    [[ -n "$dir" && -f "$dir/zipalign.exe" ]] && { echo "$dir/zipalign.exe"; return; }
}

# 查找 apksigner
find_apksigner() {
    command -v apksigner &>/dev/null && { echo "cmd:apksigner"; return; }
    local dir=$(find_build_tools_dir)
    if [[ -n "$dir" ]]; then
        # Mac/Linux: 优先使用可执行文件
        [[ -x "$dir/apksigner" ]] && { echo "cmd:$dir/apksigner"; return; }
        # 通用: 使用 jar 文件
        [[ -f "$dir/lib/apksigner.jar" ]] && { echo "jar:$dir/lib/apksigner.jar"; return; }
    fi
}

# =============================================================================
# 路径转换（仅 Windows 需要）
# =============================================================================

# 将 MINGW 路径转换为 Windows 路径（用于 Java）
to_native_path() {
    local path="$1"
    [[ "$(detect_os)" != "windows" ]] && { echo "$path"; return; }
    
    # 使用 cygpath（如果可用）
    if command -v cygpath &>/dev/null; then
        cygpath -w "$path" 2>/dev/null && return
    fi
    
    # 手动转换 /x/... 为 X:\...
    if [[ "$path" =~ ^/([a-zA-Z])/(.*) ]]; then
        echo "${BASH_REMATCH[1]^^}:\\${BASH_REMATCH[2]//\//\\}"
        return
    fi
    
    # 规范化已有的 Windows 路径
    [[ "$path" =~ ^[a-zA-Z]: ]] && { echo "${path//\//\\}"; return; }
    
    echo "$path"
}

# =============================================================================
# 签名工具执行
# =============================================================================

run_apksigner() {
    local ref="$1"; shift
    local type="${ref%%:*}"
    local path="${ref#*:}"
    
    case "$type" in
        cmd)
            "$path" "$@"
            ;;
        jar)
            local os=$(detect_os)
            if [[ "$os" == "windows" ]]; then
                # Windows: 转换所有路径参数
                local args=() prev=""
                for arg in "$@"; do
                    if [[ "$prev" == "--ks" || "$prev" == "--out" || "$arg" == *.apk ]]; then
                        args+=("$(to_native_path "$arg")")
                    else
                        args+=("$arg")
                    fi
                    prev="$arg"
                done
                java -jar "$(to_native_path "$path")" "${args[@]}"
            else
                java -jar "$path" "$@"
            fi
            ;;
        *)
            log_error "未知的 apksigner 类型: $type"
            return 1
            ;;
    esac
}

# =============================================================================
# 环境检查
# =============================================================================

print_check() {
    local status="$1" name="$2" detail="$3"
    case "$status" in
        ok)   echo "  ✓ $name: $detail";;
        warn) echo "  ⚠ $name: $detail";;
        *)    echo "  ✗ $name: $detail";;
    esac
}

preflight_check() {
    echo "=== 检查构建环境 ==="
    echo ""
    
    local has_error=false has_warning=false
    
    # 环境变量
    echo "[环境变量]"
    if [[ -n "$ANDROID_HOME" && -d "$ANDROID_HOME" ]]; then
        print_check ok "ANDROID_HOME" "$ANDROID_HOME"
    else
        print_check error "ANDROID_HOME" "${ANDROID_HOME:-未设置}"
        has_error=true
    fi
    
    if [[ -n "$ANDROID_NDK_HOME" && -d "$ANDROID_NDK_HOME" ]]; then
        print_check ok "ANDROID_NDK_HOME" "$ANDROID_NDK_HOME"
    else
        print_check error "ANDROID_NDK_HOME" "${ANDROID_NDK_HOME:-未设置}"
        has_error=true
    fi
    echo ""
    
    # Build Tools
    echo "[Android SDK]"
    local build_tools=$(find_build_tools_dir)
    if [[ -n "$build_tools" ]]; then
        print_check ok "build-tools" "$(basename "$build_tools")"
    else
        print_check error "build-tools" "未找到"
        has_error=true
    fi
    
    local zipalign=$(find_zipalign)
    [[ -n "$zipalign" ]] && print_check ok "zipalign" "$zipalign" || { print_check error "zipalign" "未找到"; has_error=true; }
    
    local apksigner=$(find_apksigner)
    if [[ -n "$apksigner" ]]; then
        print_check ok "apksigner" "$apksigner"
    elif command -v jarsigner &>/dev/null; then
        print_check warn "apksigner" "未找到，将使用 jarsigner"
        has_warning=true
    else
        print_check error "apksigner" "未找到"
        has_error=true
    fi
    echo ""
    
    # Java
    echo "[Java]"
    if command -v java &>/dev/null; then
        print_check ok "java" "$(java -version 2>&1 | head -1)"
    else
        [[ "$apksigner" == jar:* ]] && { print_check error "java" "未找到"; has_error=true; } || print_check warn "java" "未找到"
    fi
    
    if command -v keytool &>/dev/null; then
        print_check ok "keytool" "$(command -v keytool)"
    elif [[ -f "$DEBUG_KEYSTORE" ]]; then
        print_check warn "keytool" "未找到，但 keystore 已存在"
    else
        print_check error "keytool" "未找到"
        has_error=true
    fi
    echo ""
    
    # Rust
    echo "[Rust]"
    command -v cargo &>/dev/null && print_check ok "cargo" "$(cargo --version)" || { print_check error "cargo" "未找到"; has_error=true; }
    
    if command -v rustup &>/dev/null; then
        rustup target list --installed 2>/dev/null | grep -q "$ANDROID_TARGET" \
            && print_check ok "target" "$ANDROID_TARGET" \
            || { print_check warn "target" "$ANDROID_TARGET 未安装，将自动安装"; has_warning=true; }
    else
        print_check error "rustup" "未找到"
        has_error=true
    fi
    echo ""
    
    # Godot
    echo "[Godot]"
    command -v "$GODOT_BIN" &>/dev/null \
        && print_check ok "godot" "$("$GODOT_BIN" --version 2>/dev/null | head -1)" \
        || { print_check error "godot" "未找到: $GODOT_BIN"; has_error=true; }
    
    [[ -f "${GODOT_PROJECT_DIR}/export_presets.cfg" ]] \
        && print_check ok "export_presets" "存在" \
        || { print_check error "export_presets" "不存在"; has_error=true; }
    echo ""
    
    # Keystore
    echo "[签名]"
    [[ -f "$DEBUG_KEYSTORE" ]] \
        && print_check ok "keystore" "$DEBUG_KEYSTORE" \
        || { print_check warn "keystore" "将自动生成"; has_warning=true; }
    echo ""
    
    # 结果
    if [[ "$has_error" == true ]]; then
        echo "=== 检查失败 ==="
        echo "环境变量示例："
        echo "  export ANDROID_HOME=~/Library/Android/sdk        # macOS"
        echo "  export ANDROID_HOME=\$HOME/Android/Sdk            # Linux"
        echo "  export ANDROID_HOME=/c/Users/xxx/AppData/Local/Android/Sdk  # Windows"
        echo "  export ANDROID_NDK_HOME=\$ANDROID_HOME/ndk/<version>"
        echo ""
        echo "详细说明: doc/tool_install.md"
        exit 1
    fi
    
    [[ "$has_warning" == true ]] && echo "=== 检查通过（有警告）===" || echo "=== 检查通过 ==="
    echo ""
}

# =============================================================================
# 构建步骤
# =============================================================================

setup_ndk_toolchain() {
    local host=$(get_ndk_host)
    local prefix=$(get_toolchain_prefix)
    local toolchain="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$host"
    
    [[ ! -d "$toolchain" ]] && { log_error "NDK 工具链不存在: $toolchain"; exit 1; }
    
    local clang="$toolchain/bin/${prefix}${ANDROID_API}-clang"
    local ar="$toolchain/bin/llvm-ar"
    
    # Windows 需要 .cmd 后缀
    [[ "$(detect_os)" == "windows" && -f "${clang}.cmd" ]] && clang="${clang}.cmd"
    
    [[ ! -f "$clang" ]] && { log_error "找不到 clang: $clang"; exit 1; }
    
    # 设置环境变量
    local target_upper=$(echo "$ANDROID_TARGET" | tr '[:lower:]-' '[:upper:]_')
    export "CC_${target_upper}=$clang"
    export "AR_${target_upper}=$ar"
    export "CARGO_TARGET_${target_upper}_LINKER=$clang"
    
    log "NDK 工具链已配置"
    log_detail "CC: $clang"
}

check_rust_target() {
    if ! rustup target list --installed | grep -q "$ANDROID_TARGET"; then
        log "安装 Rust target: $ANDROID_TARGET"
        rustup target add "$ANDROID_TARGET"
    fi
}

generate_keystore() {
    [[ -f "$DEBUG_KEYSTORE" ]] && return
    
    log "生成 debug keystore..."
    keytool -genkey -v \
        -keystore "$DEBUG_KEYSTORE" \
        -alias "$DEBUG_KEYSTORE_ALIAS" \
        -keyalg RSA -keysize 2048 -validity 10000 \
        -storepass "$DEBUG_KEYSTORE_PASS" \
        -keypass "$DEBUG_KEYSTORE_PASS" \
        -dname "CN=Android Debug,O=Android,C=US"
}

# Android 需要使用 godot 0.4.2（0.4.3+ 有 Android 加载 bug）
GODOT_ANDROID_VERSION="0.4.2"
GODOT_DEFAULT_VERSION="0.4.5"

switch_godot_version() {
    local version="$1"
    local cargo_toml="${RUST_MANIFEST}"
    
    log "切换 godot 版本到 $version..."
    
    local os=$(detect_os)
    if [[ "$os" == "mac" ]]; then
        sed -i '' "s/godot = \"[^\"]*\"/godot = \"$version\"/" "$cargo_toml"
    else
        sed -i "s/godot = \"[^\"]*\"/godot = \"$version\"/" "$cargo_toml"
    fi
}

build_rust() {
    [[ "$SKIP_RUST_BUILD" == "1" ]] && { log "跳过 Rust 构建"; return; }
    
    # Android 使用 0.4.2 版本
    switch_godot_version "=$GODOT_ANDROID_VERSION"
    
    log "构建 Rust (target: $ANDROID_TARGET, godot: $GODOT_ANDROID_VERSION)..."
    cargo build -p "$RUST_CRATE" --target="$ANDROID_TARGET" --release --manifest-path "$RUST_MANIFEST"
    
    # 恢复默认版本
    switch_godot_version "$GODOT_DEFAULT_VERSION"
}

# 检查 Rust 动态库是否存在
check_rust_lib() {
    local src_lib="rust/target/${ANDROID_TARGET}/release/libbin.so"
    [[ ! -f "$src_lib" ]] && { log_error "Rust 动态库不存在: $src_lib"; exit 1; }
    log "Rust 动态库检查通过"
}

# 将库文件注入到 APK 的 lib 目录（Android 标准原生库位置）
inject_libs_to_apk() {
    local apk_file="$1"
    local src_lib="rust/target/${ANDROID_TARGET}/release/libbin.so"
    local temp_dir=$(mktemp -d)
    local os=$(detect_os)
    
    log "注入动态库到 APK..."
    
    # 创建 Android 标准原生库目录结构
    mkdir -p "$temp_dir/lib/arm64-v8a"
    cp "$src_lib" "$temp_dir/lib/arm64-v8a/libbin.so"
    
    # 获取绝对路径
    local abs_apk=$(cd "$(dirname "$apk_file")" && pwd)/$(basename "$apk_file")
    
    # 进入临时目录并添加文件到 APK
    cd "$temp_dir"
    
    if [[ "$os" == "windows" ]]; then
        # Windows: 使用 PowerShell 或 7z
        if command -v 7z &>/dev/null; then
            7z a -tzip "$abs_apk" lib/ > /dev/null
        else
            # 使用 PowerShell
            local abs_temp=$(pwd)
            powershell.exe -Command "
                \$apk = '$abs_apk' -replace '/', '\\';
                \$temp = '$abs_temp' -replace '/', '\\';
                Add-Type -AssemblyName System.IO.Compression.FileSystem;
                \$zip = [System.IO.Compression.ZipFile]::Open(\$apk, 'Update');
                \$files = Get-ChildItem -Path \"\$temp\\lib\" -Recurse -File;
                foreach (\$file in \$files) {
                    \$relPath = \$file.FullName.Substring(\$temp.Length + 1);
                    [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(\$zip, \$file.FullName, \$relPath) | Out-Null;
                }
                \$zip.Dispose();
            "
        fi
    else
        # Mac/Linux: 使用 zip
        zip -r "$abs_apk" lib/
    fi
    
    cd "$PROJECT_ROOT"
    
    # 清理
    rm -rf "$temp_dir"
    
    log "动态库注入完成"
}

sign_apk() {
    local unsigned="$1" signed="$2"
    local aligned="${unsigned%.apk}_aligned.apk"
    
    [[ ! -f "$unsigned" ]] && { log_error "APK 不存在: $unsigned"; exit 1; }
    
    # zipalign
    local zipalign=$(find_zipalign)
    if [[ -n "$zipalign" ]]; then
        log "执行 zipalign..."
        "$zipalign" -p -f 4 "$unsigned" "$aligned"
    else
        log "警告: zipalign 未找到，跳过对齐"
        cp "$unsigned" "$aligned"
    fi
    
    # 签名
    local apksigner=$(find_apksigner)
    mkdir -p "$(dirname "$signed")"
    
    # 获取绝对路径
    local abs_keystore=$(cd "$(dirname "$DEBUG_KEYSTORE")" && pwd)/$(basename "$DEBUG_KEYSTORE")
    local abs_aligned=$(cd "$(dirname "$aligned")" && pwd)/$(basename "$aligned")
    local abs_signed=$(cd "$(dirname "$signed")" && pwd)/$(basename "$signed")
    
    if [[ -n "$apksigner" ]]; then
        log "签名 APK..."
        run_apksigner "$apksigner" sign \
            --ks "$abs_keystore" \
            --ks-key-alias "$DEBUG_KEYSTORE_ALIAS" \
            --ks-pass "pass:$DEBUG_KEYSTORE_PASS" \
            --key-pass "pass:$DEBUG_KEYSTORE_PASS" \
            --out "$abs_signed" \
            "$abs_aligned"
        
        log "验证签名..."
        run_apksigner "$apksigner" verify "$abs_signed" || log "警告: 签名验证失败"
    else
        log "使用 jarsigner 签名（不推荐）..."
        cp "$abs_aligned" "$abs_signed"
        jarsigner -sigalg SHA256withRSA -digestalg SHA-256 \
            -keystore "$abs_keystore" \
            -storepass "$DEBUG_KEYSTORE_PASS" \
            "$abs_signed" "$DEBUG_KEYSTORE_ALIAS"
        
        # jarsigner 后需要重新 zipalign
        if [[ -n "$zipalign" ]]; then
            local temp="${abs_signed%.apk}_temp.apk"
            "$zipalign" -p -f 4 "$abs_signed" "$temp"
            mv "$temp" "$abs_signed"
        fi
    fi
    
    rm -f "$aligned"
    log "签名完成: $signed"
}

export_godot() {
    local presets="${GODOT_PROJECT_DIR}/export_presets.cfg"
    local unsigned="${EXPORT_DIR_ABS}/${EXPORT_NAME}_unsigned${ANDROID_EXT}"
    
    [[ ! -f "$presets" ]] && { log_error "export_presets.cfg 不存在"; exit 1; }
    
    # 临时禁用内置签名
    log "导出 Godot 项目..."
    local os=$(detect_os)
    if [[ "$os" == "mac" ]]; then
        sed -i '' 's/package\/signed=true/package\/signed=false/' "$presets"
    else
        sed -i.bak 's/package\/signed=true/package\/signed=false/' "$presets"
    fi
    
    mkdir -p "$EXPORT_DIR_ABS"
    "$GODOT_BIN" --headless --path "$GODOT_PROJECT_DIR" --export-release "$ANDROID_PRESET" "$unsigned" || {
        # 恢复配置
        [[ "$os" == "mac" ]] || mv "${presets}.bak" "$presets"
        exit 1
    }
    
    # 恢复配置
    if [[ "$os" == "mac" ]]; then
        sed -i '' 's/package\/signed=false/package\/signed=true/' "$presets"
    else
        mv "${presets}.bak" "$presets"
    fi
    
    # 注入动态库到 APK
    inject_libs_to_apk "$unsigned"
    
    # 签名
    generate_keystore
    sign_apk "$unsigned" "$EXPORT_PATH"
    rm -f "$unsigned"
}

# =============================================================================
# 主流程
# =============================================================================

main() {
    echo "=== Android APK 打包 ==="
    echo "平台: $(detect_os) | 目标: $ANDROID_TARGET"
    echo "输出: $EXPORT_PATH"
    echo ""
    
    preflight_check
    
    echo "=== 开始构建 ==="
    check_rust_target
    setup_ndk_toolchain
    build_rust
    check_rust_lib
    export_godot
    
    echo ""
    echo "=== 打包完成 ==="
    echo "APK: $EXPORT_PATH"
}

main "$@"
