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
    pub fn send_request(&mut self, target_ids: GString, next_action: String) {
        if self.base().get_http_client_status() != godot::classes::http_client::Status::DISCONNECTED
        {
            godot_print!("Weav3rHttpRequest: Already requesting.");
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
        http.add_header("Cookie", "__tf_verified=1770611010478.e74392adabbaf29579a9e4f62881c8e478c12fc35d286bd365b5a1ff8b5a8f3a; _ga=GA1.1.513731005.1770524621; cf_clearance=OZlKaHRTcyP1STPGJGSOv2As0_ojXe_Zs0yqOyJgVI0-1770524621-1.2.1.1-qbGhAh1vXpKnYedj03Ouv1UylV6Hwhd0IWvzbngnef5CUJaKZcw480VZFwhs6rpZrR0MPfycUDwD.VZuMzli1LGRcp.1JcQDGYEE_S3JuFp_ifx79H3ETkpsTHJVoKCNAmBsnm9xwmMDFKmGMrR0mV1adeCf2a58P9uXwgvN6b1CAVLKmH4p8yr7ASpnH9D1Qbc23MXTtAuYYJwGi5QV.Xe8BhlR0T.Bj7J7dpxJK9A; _ga_PF693NSPW1=GS2.1.s1770524620$o1$g1$t1770524626$j54$l0$h0");
        http.set_body(format!("[[{}]]", target_ids).as_bytes().to_vec());
        // godot_print!("Weav3rHttpRequest: Sending request with body: {}", target_ids);
        let request_result = http.send_request(&mut self.base_mut());
        if let Err(err) = request_result {
            godot_error!("Weav3rHttpRequest failed: {:?}", err);
            self.start_time = None;
        }
    }

    #[func]
    pub fn on_request_completed(&mut self,code :i64) {
        if let Some(start_time) = self.start_time.take() {
            let duration = start_time.elapsed();
            godot_print!(
                "Weav3rHttpRequest: Request completed in {:.3}s",
                duration.as_secs_f64()
            );
        }
        
    }
}
