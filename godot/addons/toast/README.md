# Toast Notification Plugin for Godot 4.x

A flexible toast notification library for Godot 4.x with Rust.

## Features

- Customizable styling and theming
- 9 position presets
- Fade animations
- Queue management (Unlimited, FIFO)
- Type-safe Rust API
- Auto-dismiss with close button support

## Installation

### 1. Build the Rust Extension

```bash
cd godot/addons/toast/rust
cargo build --release
```

### 2. Enable the Plugin

In Godot Editor:
- Go to `Project > Project Settings > Plugins`
- Enable "Toast Notification"

### 3. Configure AutoLoad

In `project.godot` add:

```ini
[autoload]
ToastManager="*res://addons/toast/scenes/toast_manager.tscn"
```

## Quick Start

### Basic Usage

```rust
use godot::prelude::*;
use toast::*;

let manager = get_toast_manager().unwrap();

// Show different types of toasts
manager.show("Download complete".to_string());
manager.show_with_type("Settings saved".to_string(), 1);  // Success
manager.show_with_type("Network unstable".to_string(), 2);  // Warning
manager.show_with_type("Connection failed".to_string(), 3);  // Error
```

### Custom Toast

```rust
use godot::prelude::*;
use godot::prelude::Color;
use toast::*;

let config = ToastConfig::new("Custom message")
    .with_type(ToastType::Success)
    .with_duration(ToastDuration::Long)
    .with_position(ToastPosition::BottomCenter)
    .with_width(400.0)
    .with_background_color(Color::from_rgba(0.5, 0.3, 0.8, 0.95))
    .with_corner_radius(12.0)
    .with_animation(AnimationConfig {
        enable: true,
        duration: 0.4,
    });

let manager = get_toast_manager().unwrap();
manager.show_with_config(config);
```

## API Reference

### ToastType

- `Info` (0) - Blue themed information toast
- `Success` (1) - Green themed success toast
- `Warning` (2) - Orange/yellow themed warning toast
- `Error` (3) - Red themed error toast
- `Custom` (4) - Use with custom styling

### ToastPosition

9 position presets:
- `TopLeft` (0), `TopCenter` (1), `TopRight` (2)
- `MiddleLeft` (3), `MiddleCenter` (4), `MiddleRight` (5)
- `BottomLeft` (6), `BottomCenter` (7), `BottomRight` (8)

### ToastDuration

- `Short` - 1.5 seconds
- `Medium` - 3.0 seconds
- `Long` - 5.0 seconds
- `Custom(f32)` - Custom duration in seconds

### QueueMode

- `Unlimited` (0) - Show all toasts
- `FIFO` (1) - First In First Out queue

### AnimationConfig

Simple fade animation:
- `enable` - Enable or disable animation
- `duration` - Animation duration in seconds (default: 0.3)

## ToastConfig Methods

```rust
ToastConfig::new(text)                    // Create new toast
    .with_type(ToastType::Info)           // Set toast type
    .with_duration(ToastDuration::Medium) // Set duration
    .with_position(ToastPosition::TopRight) // Set position
    .with_width(300.0)                    // Set width
    .with_background_color(Color)         // Set background color
    .with_text_color(Color)               // Set text color
    .with_font_size(16)                   // Set font size
    .with_corner_radius(8.0)              // Set corner radius
    .with_auto_dismiss(true)              // Enable/disable auto dismiss
    .with_show_close_button(true)         // Show/hide close button
    .with_animation(AnimationConfig)     // Set animation config
```

## ToastManager API

```rust
let manager = get_toast_manager().unwrap();

// Show toasts
manager.show("Simple message".to_string());
manager.show_with_type("Message".to_string(), type_id);
manager.show_with_position("Message".to_string(), position_id);
manager.show_with_duration("Message".to_string(), duration_seconds);
manager.show_with_config(config);

// Queue management
manager.dismiss_all();
manager.set_max_toasts(5);
manager.get_max_toasts();
manager.set_default_position(1);  // TopCenter
manager.get_default_position();
manager.set_queue_mode(0);  // Unlimited
manager.get_queue_mode();
```

## Advanced Usage

### Queue Management

```rust
let manager = get_toast_manager().unwrap();

// Set maximum visible toasts
manager.set_max_toasts(5);

// Set queue mode (0: Unlimited, 1: FIFO)
manager.set_queue_mode(1);

// Dismiss all toasts
manager.dismiss_all();
```

### Custom Animation

```rust
let animation = AnimationConfig {
    enable: true,
    duration: 0.5,
};

let config = ToastConfig::new("Animated toast")
    .with_animation(animation);
```

### Default Colors

Each toast type has a default background color:
- `Info`: RGBA(0.2, 0.5, 1.0, 0.9) - Blue
- `Success`: RGBA(0.2, 0.8, 0.3, 0.9) - Green
- `Warning`: RGBA(1.0, 0.7, 0.2, 0.9) - Orange
- `Error`: RGBA(0.9, 0.3, 0.3, 0.9) - Red
- `Custom`: RGBA(0.2, 0.2, 0.2, 0.9) - Dark gray

You can override these with `with_background_color()`.

## File Structure

```
addons/toast/
├── rust/                    # Rust source code
│   ├── src/
│   │   ├── lib.rs          # Entry point
│   │   ├── types.rs        # Type definitions
│   │   ├── config.rs       # ToastConfig
│   │   ├── toast.rs        # Toast node
│   │   ├── manager.rs      # ToastManager
│   │   └── entry.rs        # GDExtension entry
│   ├── Cargo.toml
│   └── target/release/libgodot_toast.rlib
├── scenes/
│   ├── toast.tscn          # Toast scene
│   └── toast_manager.tscn  # Manager scene
└── README.md
```

## Building

```bash
# Release build
cd godot/addons/toast/rust
cargo build --release

# Debug build
cargo build
```

The compiled library will be at:
- Linux/macOS: `target/release/libgodot_toast.rlib`
- Windows: `target/release/godot_toast.rlib`

## License

MIT OR Apache-2.0
