use godot::{classes::DirAccess, global};

pub fn create_folder(url: &str) -> Option<global::Error> {
    let e = DirAccess::make_dir_recursive_absolute(url);
    match e {
        global::Error::OK => None,
        _ => Some(e),
    }
}
