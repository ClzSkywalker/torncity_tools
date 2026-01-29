use godot::{classes::ResourceLoader, obj::WithBaseField, prelude::*};

pub trait INodeFunc: GodotClass + WithBaseField {
    fn node_path() -> &'static str;
}

/// Node工具类
pub trait INodeTool: GodotClass + WithBaseField + INodeFunc {
    /// 自身类型
    type SelfType: GodotClass + Inherits<Node> + INodeFunc;

    /// 根据路径获取子节点
    fn get_node_as<R>(&self, path: &str) -> Option<Gd<R>>
    where
        Self::Base: Inherits<Node>,
        R: GodotClass + Inherits<Node>,
    {
        match self
            .base()
            .clone()
            .upcast::<Node>()
            .get_node_or_null(path)
            .and_then(|node| node.try_cast::<R>().ok())
        {
            Some(r) => Some(r),
            None => {
                godot_error!(
                    "{}: Failed to get node: {},type: {}",
                    Self::node_path(),
                    path,
                    std::any::type_name::<R>()
                );
                None
            }
        }
    }

    /// 根据路径获取场景
    fn get_scene() -> Option<Gd<PackedScene>> {
        match ResourceLoader::singleton()
            .load(Self::node_path())
            .and_then(|res| res.try_cast::<PackedScene>().ok())
        {
            Some(scene) => Some(scene),
            None => {
                godot_error!("{}: Failed to load scene", Self::node_path());
                None
            }
        }
    }

    /// 获取场景实例
    fn get_scene_instance() -> Option<Gd<Self::SelfType>> {
        let scene = Self::get_scene()?;
        let Some(instance) = scene.instantiate() else {
            godot_error!("{}: Failed to instantiate scene", Self::node_path());
            return None;
        };
        let Ok(instance) = instance.try_cast::<Self::SelfType>() else {
            godot_error!(
                "{}: Instance is not {}",
                Self::node_path(),
                Self::SelfType::node_path()
            );
            return None;
        };
        Some(instance)
    }
}

impl<T> INodeTool for T
where
    T: INodeFunc + GodotClass + WithBaseField + Inherits<Node>,
    T::Base: Inherits<Node>,
{
    type SelfType = T;
}
