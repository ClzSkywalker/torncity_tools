use godot::{classes::HttpRequest, prelude::*};
use std::time::Instant;
use tools::http::HttpTool;

#[derive(GodotClass)]
#[class(init, base=HttpRequest)]
pub struct Weav3rHttpRequest {
    #[export]
    type_name: GString,
    #[base]
    base: Base<HttpRequest>,
    start_time: Option<Instant>,
}

#[godot_api]
impl Weav3rHttpRequest {
    #[allow(dead_code)]
    fn init(base: Base<HttpRequest>) -> Self {
        Self {
            type_name: "Weav3rHttpRequest".into(),
            base,
            start_time: None,
        }
    }

    #[func]
    pub fn send_request(&mut self, target_ids: GString, next_action: String, cookie: String) {
        if self.base().get_http_client_status() != godot::classes::http_client::Status::DISCONNECTED
        {
            godot_print!("Weav3rHttpRequest: Already requesting.");
            return;
        }

        if target_ids.is_empty() {
            godot_warn!("Weav3rHttpRequest: target_ids is empty.");
            return;
        }
        if next_action.is_empty() {
            godot_warn!("Weav3rHttpRequest: next_action is empty.");
            return;
        }
        if cookie.is_empty() {
            godot_warn!("Weav3rHttpRequest: cookie is empty.");
            return;
        }

        self.start_time = Some(Instant::now());

        let mut http = HttpTool::default();
        http.set_url("https://weav3r.dev/favorites");
        // http.set_url("http://127.0.0.1:8666/favorites");
        http.set_method(godot::classes::http_client::Method::POST);
        http.add_header("Connection", "keep-alive");
        http.add_header("Accept", "text/x-component");
        http.add_header("Accept-Encoding", "gzip, deflate, br");
        http.add_header("Accept-Language", "zh-CN,zh;q=0.8");
        http.add_header("Content-Type", "text/plain;charset=UTF-8");
        http.add_header("Next-Action", next_action.as_str());
        http.add_header("Cookie", cookie.as_str());
        http.set_body(format!("[[{}]]", target_ids).as_bytes().to_vec());
        let request_result = http.send_request(&mut self.base_mut());
        if let Err(err) = request_result {
            godot_error!("Weav3rHttpRequest failed: {:?}", err);
            self.start_time = None;
        }
    }

    #[func]
    pub fn on_request_completed(&mut self,_code :i64) {
        if let Some(start_time) = self.start_time.take() {
            let duration = start_time.elapsed();
            godot_print!(
                "Weav3rHttpRequest: Request completed in {:.3}s",
                duration.as_secs_f64()
            );
        }
        
    }
}
