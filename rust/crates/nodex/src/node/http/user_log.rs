use godot::{classes::HttpRequest, prelude::*};
use model::{
    error::MyError,
    user_log::{UserLogEntry, UserLogReq, UserLogResponse},
};
use tools::http::HttpTool;

#[derive(GodotClass)]
#[class(init, base=HttpRequest)]
pub struct UserLogHttpRequest {
    #[export]
    type_name: GString,
    #[base]
    base: Base<HttpRequest>,
}

impl UserLogHttpRequest {
    pub fn send_request(&mut self, req: UserLogReq) {
        if self.base().get_http_client_status() != godot::classes::http_client::Status::DISCONNECTED
        {
            godot_print!("Weav3rHttpRequest: Already requesting.");
            return;
        }

        if req.key.is_empty() {
            godot_error!("UserLogHttpRequest: Key is empty.");
            return;
        }

        let mut http = HttpTool::default();
        http.set_url("https://api.torn.com/v2/user/log");
        http.add_param("limit", req.limit.to_string().as_str());
        http.add_param("key", req.key.as_str());
        if let Some(target) = req.target {
            http.add_param("target", target.to_string().as_str());
        }
        if let Some(cat) = req.cat {
            http.add_param("cat", cat.to_string().as_str());
        }
        http.add_header("accept", "application/json");
        http.set_method(godot::classes::http_client::Method::GET);
        if let Err(err) = http.send_request(&mut self.base_mut()) {
            godot_error!("UserLogHttpRequest failed: {:?}", err);
        }
    }

    pub fn parse_response(
        _result: i64,
        response_code: i64,
        _headers: PackedStringArray,
        body: PackedByteArray,
    ) -> Result<Vec<UserLogEntry>, MyError> {
        if response_code != 200 {
            godot_error!(
                "UserLogHttpRequest: Failed to get response.code: {}",
                response_code
            );
            return Err(MyError::NetworkCode(
                response_code,
                String::from("Failed to get response."),
            ));
        }

        let response_text = String::from_utf8_lossy(body.as_slice());
        match UserLogResponse::from_json(&response_text) {
            Ok(resp) => Ok(resp.log),
            Err(e) => {
                godot_error!(
                    "UserLogHttpRequest: Failed to parse response: {:?},value: {}",
                    e,
                    response_text
                );
                Err(e)
            }
        }
    }
}
