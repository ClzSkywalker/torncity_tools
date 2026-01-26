use godot::{obj::WithBaseField, prelude::*};

// pub struct NodeTool;

pub trait INodeFunc: GodotClass + WithBaseField {
    fn node_path() -> &'static str;
}

/// Node工具类
pub trait INodeTool: GodotClass + WithBaseField {
    fn get_node_as<R>(&self, path: &str) -> Option<Gd<R>>
    where
        Self::Base: Inherits<Node>,
        R: GodotClass + Inherits<Node>,
    {
        self.base()
            .clone()
            .upcast::<Node>()
            .get_node_or_null(path)
            .and_then(|node| node.try_cast::<R>().ok())
    }
}

impl<T> INodeTool for T
where
    T: GodotClass + WithBaseField,
    T::Base: Inherits<Node>,
{
}
