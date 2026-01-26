use godot::{classes::HttpRequest, prelude::*};

#[derive(GodotClass)]
#[class(init, base=HttpRequest)]
pub struct Weav3rHttpRequest {
    #[export]
    type_name: GString,
    #[base]
    base: Base<HttpRequest>,
}

#[godot_api]
impl Weav3rHttpRequest {
    #[allow(dead_code)]
    fn init(base: Base<HttpRequest>) -> Self {
        Self {
            type_name: "Weav3rHttpRequest".into(),
            base,
        }
    }

    #[func]
    pub fn send_request(&mut self, target_ids: GString) {
        let mut headers = PackedStringArray::new();
        headers.push("Connection: keep-alive");
        headers.push("Accept: text/x-component");
        headers.push("Accept-Encoding: gzip, deflate, br");
        headers.push("Accept-Language: zh-CN,zh;q=0.8");
        headers.push("Content-Type: text/plain;charset=UTF-8");
        headers.push("Next-Action: 40021a4315136e25e6fc16a05ca62ab3218111068a");
        headers.push("Cookie: cf_clearance=LGkK7gXt4rzJAEpcbm00jNjfFMdYzjEsFo8M63HG49A-1768298710-1.2.1.1-Qquf0_4B_Ei7ZCmews9rVovka9v0ushpbQTDxbC2pNiriRj9k.PvUeUv9FclLRc6y2.zRBrPUpaLh3u6cftrKohRgHsbn3YJZUu2cFjh5r4uVf6ieqLgu1e4C3l0iJkLcq0fVc0BtqnaLsAqoPn2c68WBB0zo3tQdmlu9ldEcryDQaNkc5n7IIMcZoydCjNPMoobIfz2ESlDX132FsDkOWFnej73oSkEKOBe124hdDw");

        let body_text = format!("[[{}]]", target_ids);
        let body = PackedByteArray::from(body_text.as_bytes());

        let err = self
            .base_mut()
            .request_raw_ex("https://weav3r.dev/favorites")
            .custom_headers(&headers)
            .method(godot::classes::http_client::Method::POST)
            .request_data_raw(&body)
            .done();
        if err != godot::global::Error::OK {
            godot_error!("HttpRequest failed: {:?}", err);
        }
    }
}
