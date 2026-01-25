use godot::prelude::*;

pub use nodex::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::Editor {
            godot_print!("MainLoop");
        }
    }
}
