use godot::{classes::*, global, prelude::*};
use tools::{
    image::{load_image_texture_from_buffer, load_image_texture_from_disk},
    node::{INodeFunc, INodeTool},
};

/// 加载网络图片
#[derive(GodotClass)]
#[class(init,base=PanelContainer)]
pub struct ImageHttpRequest {
    base: Base<PanelContainer>,
    #[export]
    pub url: GString,
    #[export]
    pub texture: Option<Gd<Texture2D>>,
    #[export]
    pub width: f32,
    #[export]
    pub height: f32,
    icon: Option<Gd<TextureRect>>,
    icon_request: Option<Gd<HttpRequest>>,
}

#[godot_api]
impl IPanelContainer for ImageHttpRequest {
    fn ready(&mut self) {
        self.icon = self.get_node_as::<TextureRect>("TextureRect");
        self.icon_request = self.get_node_as::<HttpRequest>("HTTPRequest");

        let Some(request) = self.icon_request.as_mut() else {
            godot_error!("ImageHttpRequest: Icon request node not found.");
            return;
        };
        let mut request = request.clone();
        request
            .clone()
            .signals()
            .request_completed()
            .connect_other(self, Self::on_icon_request_completed);

        let Some(icon) = self.icon.as_mut() else {
            return;
        };
        // 设置自定义最小尺寸
        icon.set_custom_minimum_size(Vector2::new(self.width, self.height));

        if let Some(texture) = self.texture.clone() {
            icon.set_texture(Some(&texture));
        }

        if self.url.is_empty() {
            return;
        }
        if let Some(texture) = load_image_texture_from_disk(self.url.to_string().as_str()) {
            icon.set_texture(Some(&texture));
            return;
        }
        let err = request.request(self.url.to_string().as_str());
        if err != global::Error::OK {
            godot_error!(
                "ImageHttpRequest: Icon request failed: {:?}, url: {}",
                err,
                self.url
            );
        }
    }
}

impl INodeFunc for ImageHttpRequest {
    fn node_path() -> &'static str {
        "res://node/icon_http_request.tscn"
    }
}

#[godot_api]
impl ImageHttpRequest {
    #[func]
    pub fn set_url_request(&mut self, url: GString) {
        self.url = url.clone();
        if let Some(request) = self.icon_request.as_mut() {
            let e = request.request(url.to_string().as_str());
            if e != global::Error::OK {
                godot_error!(
                    "ImageHttpRequest: Icon request failed: {:?}, url: {}",
                    e,
                    url
                );
            }
        } else {
            godot_error!("ImageHttpRequest: Icon request node not found.");
        }
    }

    #[func]
    fn on_icon_request_completed(
        &mut self,
        _result: i64,
        response_code: i64,
        _headers: PackedStringArray,
        body: PackedByteArray,
    ) {
        if response_code != 200 {
            godot_error!(
                "ImageHttpRequest: Icon request failed. code: {}",
                response_code
            );
            return;
        }
        let Some(texture) = load_image_texture_from_buffer(&self.url.to_string(), true, &body)
        else {
            return;
        };
        let Some(icon) = self.icon.as_mut() else {
            godot_error!("ImageHttpRequest: Icon node not found.");
            return;
        };

        // 设置自定义最小尺寸
        icon.set_custom_minimum_size(Vector2::new(self.width, self.height));

        icon.set_texture(Some(&texture));
    }
}
