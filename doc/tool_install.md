# 工具安装指南

## Windows Make 安装

以管理员身份运行CMD，安装Chocolatey包管理器：
```cmd
@"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -InputFormat None -ExecutionPolicy Bypass -Command "iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))" && SET "PATH=%PATH%;%ALLUSERSPROFILE%\chocolatey\bin"
```

安装GNU Make：
```cmd
choco install make
```

## Android 构建工具安装

Android 打包需要以下工具：
- **zipalign** - APK 对齐工具（Android 11+ 强制要求）
- **apksigner** - APK 签名工具
- **Android NDK** - Rust 交叉编译需要

这些工具都包含在 Android SDK 中。

### 方法一：通过 Android Studio 安装（推荐）

1. 下载并安装 [Android Studio](https://developer.android.com/studio)
2. 打开 Android Studio → Settings → Languages & Frameworks → Android SDK
3. 在 **SDK Tools** 标签页中勾选：
   - Android SDK Build-Tools（包含 zipalign、apksigner）
   - Android SDK Command-line Tools
   - NDK (Side by side)
4. 点击 Apply 安装

安装完成后，工具位于：
- Windows: `C:\Users\<用户名>\AppData\Local\Android\Sdk`
- macOS: `~/Library/Android/sdk`
- Linux: `~/Android/Sdk`

### 方法二：通过命令行安装（无需 Android Studio）

1. 下载 [Command line tools only](https://developer.android.com/studio#command-line-tools-only)

2. 解压到指定目录，例如 `C:\android-sdk`

3. 设置环境变量：
```cmd
set ANDROID_HOME=C:\android-sdk
set PATH=%PATH%;%ANDROID_HOME%\cmdline-tools\latest\bin
```

4. 安装 Build Tools 和 NDK：
```bash
sdkmanager "build-tools;34.0.0" "ndk;26.1.10909125"
```

### 环境变量配置

在系统环境变量中添加（Windows）：

```
ANDROID_HOME = C:\Users\<用户名>\AppData\Local\Android\Sdk
ANDROID_NDK_HOME = %ANDROID_HOME%\ndk\<版本号>
PATH 追加: %ANDROID_HOME%\build-tools\<版本号>
```

在 `.bashrc` 或 `.zshrc` 中添加（Linux/macOS）：

```bash
export ANDROID_HOME=~/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<版本号>
export PATH=$PATH:$ANDROID_HOME/build-tools/<版本号>
```

### 验证安装

```bash
# 检查 zipalign
zipalign -h

# 检查 apksigner
apksigner --help

# 检查 NDK
echo $ANDROID_NDK_HOME
ls $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/
```

### 常见问题

**Q: 找不到 zipalign 命令？**

A: zipalign 在 `build-tools/<版本号>/` 目录下，需要将该目录添加到 PATH，或者设置 `ANDROID_HOME` 环境变量让脚本自动查找。

**Q: 打包时提示 linker `cc` not found？**

A: 需要设置 `ANDROID_NDK_HOME` 环境变量指向 NDK 安装目录。

**Q: APK 安装失败，提示 resources.arsc 对齐问题？**

A: Android 11+ 要求 APK 必须经过 zipalign 对齐。确保 `ANDROID_HOME` 已设置，打包脚本会自动调用 zipalign。

## magick ico 生成工具

https://imagemagick.org/script/download.php