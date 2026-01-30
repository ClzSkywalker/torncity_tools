#!/bin/bash
# package-all.sh - 统一打包入口

set -e  # 遇到错误立即退出

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

run_desktop() {
    bash "$SCRIPT_DIR/package-desktop.sh"
}

run_android() {
    bash "$SCRIPT_DIR/package-android.sh"
}

run_ios() {
    bash "$SCRIPT_DIR/package-ios.sh"
}

run_mobile() {
    run_android
    run_ios
}

run_mac() {
    EXPORT_PRESET="macOS" bash "$SCRIPT_DIR/package-desktop.sh"
}

run_win() {
    EXPORT_PRESET="Windows Desktop" bash "$SCRIPT_DIR/package-desktop.sh"
}

run_linux() {
    EXPORT_PRESET="Linux" bash "$SCRIPT_DIR/package-desktop.sh"
}

usage() {
    echo "用法: $0 [release|desktop|mobile|android|ios|mac|win|linux|all]"
    echo "说明:"
    echo "  release/desktop  : 自动检测当前 OS 导出"
    echo "  mobile           : 依次打包 Android 和 iOS"
    echo "  mac/win/linux     : 强制指定桌面平台导出"
    echo "  android          : 打包 Android 版本"
    echo "  ios              : 打包 iOS 版本"
    echo "  all              : release + mobile"
}

main() {
    if [ "$#" -eq 0 ]; then
        usage
        exit 0
    fi

    case "$1" in
        release|desktop)
            run_desktop
            ;;
        mobile)
            run_mobile
            ;;
        android)
            run_android
            ;;
        ios)
            run_ios
            ;;
        mac)
            run_mac
            ;;
        win|windows)
            run_win
            ;;
        linux)
            run_linux
            ;;
        all)
            run_desktop
            run_mobile
            ;;
        *)
            usage
            exit 1
            ;;
    esac
}

main "$@"
