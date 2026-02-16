use godot::prelude::*;

pub use nodex::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::MainLoop {
            godot_print!("InitStage: Scene");
            ToastManager::setup_toast_manager();
        }
    }
}
