插件：
1. rust_tools

```text
project-root/
├── godot/                     # Godot 工程目录
│   ├── project.godot
│   ├── addons/
│   ├── scenes/
│   ├── scripts/               # GDScript(可选)
│   ├── assets/
│   └── native/                # 放 .gdextension / 产物
│       ├── myext.gdextension
│       └── bin/               # Rust 输出的 .dll/.so/.dylib
│
├── rust/                      # Rust workspace 根
│   ├── Cargo.toml             # workspace root
│   ├── gdext_game/            # 你的 Godot 扩展 crate（必需）
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── crates/                # 你自定义的库 crate
│       ├── logic/
│       ├── component/
│       └── utils/
│
└── tools/                     # 构建脚本/CI(可选)
```

mac: debug
> https://godot-rust.github.io/book/toolchain/debugging.html


cfg: 
Windows: %APPDATA%\Godot\app_userdata\[project_name]\settings.cfg
实际路径通常是：C:\Users\[用户名]\AppData\Roaming\Godot\app_userdata\[项目名]\settings.cfg
macOS: ~/Library/Application\ Support/Godot/app_userdata/Torn\ Trade/settings.cfg
实际路径：/Users/[用户名]/Library/Application Support/Godot/app_userdata/[项目名]/settings.
Linux: ~/.local/share/godot/app_userdata/[project_name]/settings.cfg
实际路径：/home/[用户名]/.local/share/godot/app_userdata/[项目名]/settings.cfg
Web (HTML5) 平台：使用 IndexedDB 存储虚拟文件系统
Android
主要路径：
/data/data/[包名]/files/settings.cfg
示例：
/data/data/com.godotengine.example/files/settings.cfg
替代路径（某些配置下）：
内部共享存储/Android/data/[包名]/files/settings.cfg
iOS
主要路径（应用沙盒内）：
[应用沙盒]/Documents/settings.cfg
或
[应用沙盒]/Library/Application Support/settings.cfg