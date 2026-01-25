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