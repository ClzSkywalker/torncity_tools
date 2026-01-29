use godot::{classes::HttpRequest, prelude::*};
use tools::http::HttpTool;

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
        if self.base().get_http_client_status() != godot::classes::http_client::Status::DISCONNECTED
        {
            godot_print!("Weav3rHttpRequest: Already requesting.");
            return;
        }

        let mut http = HttpTool::default();
        http.set_url("https://weav3r.dev/favorites");
        http.add_header("Connection", "keep-alive");
        http.add_header("Accept", "text/x-component");
        http.add_header("Accept-Encoding", "gzip, deflate, br");
        http.add_header("Accept-Language", "zh-CN,zh;q=0.8");
        http.add_header("Content-Type", "text/plain;charset=UTF-8");
        http.add_header("Next-Action", "40021a4315136e25e6fc16a05ca62ab3218111068a");
        http.add_header("Cookie", "cf_clearance=LGkK7gXt4rzJAEpcbm00jNjfFMdYzjEsFo8M63HG49A-1768298710-1.2.1.1-Qquf0_4B_Ei7ZCmews9rVovka9v0ushpbQTDxbC2pNiriRj9k.PvUeUv9FclLRc6y2.zRBrPUpaLh3u6cftrKohRgHsbn3YJZUu2cFjh5r4uVf6ieqLgu1e4C3l0iJkLcq0fVc0BtqnaLsAqoPn2c68WBB0zo3tQdmlu9ldEcryDQaNkc5n7IIMcZoydCjNPMoobIfz2ESlDX132FsDkOWFnej73oSkEKOBe124hdDw");
        http.set_method(godot::classes::http_client::Method::POST);
        http.set_body(format!("[[{}]]", target_ids).as_bytes().to_vec());
        if let Err(err) = http.send_request(&mut self.base_mut()) {
            godot_error!("Weav3rHttpRequest failed: {:?}", err);
        }
    }
}
