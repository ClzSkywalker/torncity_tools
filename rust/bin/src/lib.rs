use godot::prelude::*;

pub use nodex::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_stage_init(stage: InitStage) {
        match stage {
            InitStage::Servers => {
                godot_print!("stage: Servers");
            },
            InitStage::Editor => godot_print!("stage: Editor"),
            _ => (),
        }
    }
}
