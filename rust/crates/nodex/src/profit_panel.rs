use godot::{
    classes::{file_access::ModeFlags, *},
    global::Error,
    prelude::*,
};
use std::hash::{Hash, Hasher};
use tools::node::{INodeFunc, INodeTool};
use weav3r::profit::ProfitInfo;

const ICON_PATH: &str = "%Icon";
const ICON_REQUEST_PATH: &str = "IconRequest";
const ICON_CACHE_DIR: &str = "user://icon_cache";

#[derive(GodotClass)]
#[class(init,base=PanelContainer)]
pub struct ProfitPanel {
    pub item: ProfitInfo,
    #[base]
    base: Base<PanelContainer>,
    name_label: Option<Gd<Label>>,
    quantity_label: Option<Gd<Label>>,
    icon: Option<Gd<TextureRect>>,
    icon_request: Option<Gd<HttpRequest>>,
}

#[godot_api]
impl IPanelContainer for ProfitPanel {
    fn ready(&mut self) {
        self.name_label = self.get_node_as::<Label>("%Name");
        self.quantity_label = self.get_node_as::<Label>("%Quantity");
        self.icon = self.get_node_as::<TextureRect>("%Icon");
        self.icon_request = self.get_node_as::<HttpRequest>("IconRequest");

        self.load_icon();

        let mut vbox = self
            .base()
            .get_node_as::<VBoxContainer>("HBoxContainer/VBox");

        let Some(mut profit_item) = ProfitItem::get_scene_instance() else {
            godot_error!("ProfitPanel: Failed to instantiate profit_item_scene");
            return;
        };
        profit_item.bind_mut().set_value(
            "Final Profit".to_string(),
            self.item.profit_single_value,
            self.item.profit_total_value,
            self.item.profit_percentage,
        );
        vbox.add_child(Some(&profit_item.upcast::<Node>()));

        let Some(mut profit_item) = ProfitItem::get_scene_instance() else {
            godot_error!("ProfitPanel: Failed to get profit_item_scene");
            return;
        };
        profit_item.bind_mut().set_value(
            "Market Profit".to_string(),
            self.item.market_profit_single_value,
            self.item.market_profit_total_value,
            self.item.market_profit_percentage,
        );
        vbox.add_child(Some(&profit_item.upcast::<Node>()));

        let Some(mut profit_item) = ProfitItem::get_scene_instance() else {
            godot_error!("ProfitPanel: Failed to get profit_item_scene");
            return;
        };
        profit_item.bind_mut().set_value(
            "Avg Bazaar Profit".to_string(),
            self.item.avg_bazaar_profit_single_value,
            self.item.avg_bazaar_profit_total_value,
            self.item.avg_bazaar_profit_percentage,
        );
        vbox.add_child(Some(&profit_item.upcast::<Node>()));
    }
}

impl INodeFunc for ProfitPanel {
    fn node_path() -> &'static str {
        "res://scenes/profit_panel.tscn"
    }
}

impl ProfitPanel {
    pub fn set_item(&mut self, item: ProfitInfo) {
        self.item = item;
    }

    fn load_icon(&mut self) {
        let url = self.item.image.trim();
        if url.is_empty() {
            return;
        }
        if let Some(texture) = Self::load_texture_from_disk(url) {
            let mut icon = self.base().get_node_as::<TextureRect>(ICON_PATH);
            icon.set_texture(Some(&texture));
            return;
        }
        let mut request = self.base().get_node_as::<HttpRequest>(ICON_REQUEST_PATH);
        request
            .signals()
            .request_completed()
            .connect_other(self, Self::on_icon_request_completed);
        let err = request.request(url);
        if err != Error::OK {
            godot_error!("ProfitPanel: Icon request failed: {:?}", err);
        }
    }

    fn load_image_from_buffer(&self, body: &PackedByteArray) -> Option<Gd<Image>> {
        let ext = Self::get_url_extension(self.item.image.as_str());
        Self::decode_image_from_buffer(body, ext.as_str())
    }

    fn decode_image_from_buffer(body: &PackedByteArray, ext: &str) -> Option<Gd<Image>> {
        let mut image = Image::new_gd();
        let err = match ext {
            "png" => image.load_png_from_buffer(body),
            "jpg" | "jpeg" => image.load_jpg_from_buffer(body),
            "webp" => image.load_webp_from_buffer(body),
            "svg" => image.load_svg_from_buffer(body),
            _ => image.load_png_from_buffer(body),
        };
        if err != Error::OK {
            godot_error!("ProfitPanel: Failed to decode icon: {:?}", err);
            return None;
        }
        Some(image)
    }

    fn load_texture_from_disk(url: &str) -> Option<Gd<ImageTexture>> {
        let path = Self::get_cache_path(url);
        let bytes = FileAccess::get_file_as_bytes(path.as_str());
        if bytes.is_empty() {
            return None;
        }
        let ext = Self::get_url_extension(path.as_str());
        let image = Self::decode_image_from_buffer(&bytes, ext.as_str())?;
        ImageTexture::create_from_image(&image)
    }

    fn ensure_cache_dir() {
        if let Some(mut dir) = DirAccess::open("user://") {
            let _ = dir.make_dir("icon_cache");
        } else {
            let _ = DirAccess::make_dir_absolute(ICON_CACHE_DIR);
        }
    }

    fn get_cache_path(url: &str) -> String {
        let hash = Self::hash_url(url);
        let ext = Self::get_url_extension(url);
        format!("{}/{}.{}", ICON_CACHE_DIR, hash, ext)
    }

    fn get_url_extension(url: &str) -> String {
        let lower = url.to_lowercase();
        let no_query = lower.split('?').next().unwrap_or("");
        match no_query.rsplit('.').next() {
            Some(ext @ ("png" | "jpg" | "jpeg" | "webp" | "svg")) => ext.to_string(),
            _ => "png".to_string(),
        }
    }

    fn hash_url(url: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        hasher.finish()
    }
}

#[godot_api]
impl ProfitPanel {
    #[func]
    fn on_icon_request_completed(
        &mut self,
        _result: i64,
        response_code: i64,
        _headers: PackedStringArray,
        body: PackedByteArray,
    ) {
        if response_code != 200 {
            godot_error!("ProfitPanel: Icon request failed. code: {}", response_code);
            return;
        }
        let Some(image) = self.load_image_from_buffer(&body) else {
            return;
        };
        let Some(texture) = ImageTexture::create_from_image(&image) else {
            godot_error!("ProfitPanel: Failed to create texture from image.");
            return;
        };
        let url = self.item.image.trim();
        if !url.is_empty() {
            Self::ensure_cache_dir();
            let path = Self::get_cache_path(url);
            if let Some(mut file) = FileAccess::open(path.as_str(), ModeFlags::WRITE) {
                let _ = file.store_buffer(&body);
            }
        }
        let mut icon = self.base().get_node_as::<TextureRect>(ICON_PATH);
        icon.set_texture(Some(&texture));
    }
}

#[derive(GodotClass)]
#[class(init,base=Control)]
pub struct ProfitItem {
    pub label: String,
    pub single_value: i32,
    pub total_value: i64,
    pub percentage: f32,
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for ProfitItem {
    fn ready(&mut self) {
        self.base()
            .get_node_as::<Label>("%Label")
            .set_text(format!("Name:{}", self.label).as_str());
        self.base()
            .get_node_as::<Label>("%ProfitSingleValue")
            .set_text(format!("Value:{}", self.single_value).as_str());
        self.base()
            .get_node_as::<Label>("%ProfitTotalValue")
            .set_text(format!("Profit:{}", self.total_value).as_str());
        self.base()
            .get_node_as::<Label>("%ProfitPercent")
            .set_text(format!("Percentage:{:.2}%", self.percentage).as_str());
    }
}

impl INodeFunc for ProfitItem {
    fn node_path() -> &'static str {
        "res://scenes/profit_item.tscn"
    }
}

impl ProfitItem {
    pub fn set_value(
        &mut self,
        label: String,
        single_value: i32,
        total_value: i64,
        percentage: f32,
    ) {
        self.label = label;
        self.single_value = single_value;
        self.total_value = total_value;
        self.percentage = percentage;
    }
}
