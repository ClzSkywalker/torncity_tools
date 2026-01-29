use std::collections::HashMap;

use godot::{
    classes::{
        HttpRequest,
        class_macros::private::virtuals::Os::{PackedByteArray, PackedStringArray},
        http_client,
    },
    global::godot_error,
};

#[derive(Debug, Clone)]
pub struct HttpTool {
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub method: http_client::Method,
    pub url: String,
}

impl Default for HttpTool {
    fn default() -> Self {
        Self {
            params: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            method: http_client::Method::GET,
            url: String::new(),
        }
    }
}

impl HttpTool {
    pub fn add_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }
    pub fn set_headers(&mut self, headers: HashMap<String, String>) {
        self.headers = headers;
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    pub fn set_method(&mut self, method: http_client::Method) {
        self.method = method;
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub fn send_request(&self, request: &mut HttpRequest) -> Result<(), godot::global::Error> {
        if self.url.is_empty() {
            godot_error!("HttpTool: URL is empty.");
            return Err(godot::global::Error::ERR_INVALID_PARAMETER);
        }
        let mut headers = PackedStringArray::new();
        for (key, value) in &self.headers {
            headers.push(format!("{}: {}", key, value).as_str());
        }

        let body = PackedByteArray::from(self.body.clone());

        let err = request
            .request_raw_ex(self.url.as_str())
            .custom_headers(&headers)
            .method(self.method)
            .request_data_raw(&body)
            .done();
        if err != godot::global::Error::OK {
            godot_error!(
                "HttpTool: Failed to send request params: {:?}, err: {:?}",
                self,
                err
            );
            return Err(err);
        }
        Ok(())
    }
}
