use std::collections::HashMap;

use godot::{
    classes::{
        HttpRequest,
        class_macros::private::virtuals::Os::{PackedByteArray, PackedStringArray},
        http_client,
    },
    global::godot_error,
};

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    fn from_str(method: &str) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        }
    }
}

impl From<http_client::Method> for HttpMethod {
    fn from(method: http_client::Method) -> Self {
        match method {
            http_client::Method::GET => HttpMethod::GET,
            http_client::Method::POST => HttpMethod::POST,
            http_client::Method::PUT => HttpMethod::PUT,
            http_client::Method::DELETE => HttpMethod::DELETE,
            http_client::Method::PATCH => HttpMethod::PATCH,
            http_client::Method::HEAD => HttpMethod::HEAD,
            http_client::Method::OPTIONS => HttpMethod::OPTIONS,
            _ => HttpMethod::GET,
        }
    }
}

impl From<HttpMethod> for http_client::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => http_client::Method::GET,
            HttpMethod::POST => http_client::Method::POST,
            HttpMethod::PUT => http_client::Method::PUT,
            HttpMethod::DELETE => http_client::Method::DELETE,
            HttpMethod::PATCH => http_client::Method::PATCH,
            HttpMethod::HEAD => http_client::Method::HEAD,
            HttpMethod::OPTIONS => http_client::Method::OPTIONS,
        }
    }
}

#[derive(Clone)]
pub struct HttpTool {
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub method: HttpMethod,
    pub url: String,
}

impl std::fmt::Debug for HttpTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body_str = if self.body.is_empty() {
            String::from("<empty>")
        } else if let Ok(s) = String::from_utf8(self.body.clone()) {
            s
        } else {
            format!("<binary data, {} bytes>", self.body.len())
        };

        write!(f, "HttpTool {{\n")?;
        write!(f, "  params: {:?},\n", self.params)?;
        write!(f, "  headers: {:?},\n", self.headers)?;
        write!(f, "  body: {},\n", body_str)?;
        write!(f, "  method: {:?},\n", self.method)?;
        write!(f, "  url: {:?}\n", self.url)?;
        write!(f, "}}")
    }
}

impl Default for HttpTool {
    fn default() -> Self {
        Self {
            params: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            method: HttpMethod::GET,
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
        self.method = HttpMethod::from(method);
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub fn from_curl(curl_command: &str) -> Result<Self, String> {
        let mut tool = HttpTool::default();
        let mut method = HttpMethod::GET;
        let mut url: Option<String> = None;
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut data: Option<String> = None;
        let mut form_data: Vec<String> = Vec::new();

        let tokens: Vec<String> = tokenize_curl(curl_command);

        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            match token.as_str() {
                "curl" | "curl.exe" => {}
                "-X" | "--request" => {
                    if i + 1 < tokens.len() {
                        method = HttpMethod::from_str(&tokens[i + 1]);
                        i += 1;
                    }
                }
                "-H" | "--header" => {
                    if i + 1 < tokens.len() {
                        if let Some((key, value)) = parse_header(&tokens[i + 1]) {
                            headers.insert(key, value);
                        }
                        i += 1;
                    }
                }
                "-d" | "--data" | "--data-ascii" | "--data-binary" => {
                    if i + 1 < tokens.len() {
                        data = Some(tokens[i + 1].clone());
                        i += 1;
                    }
                }
                "-F" | "--form" => {
                    if i + 1 < tokens.len() {
                        form_data.push(tokens[i + 1].clone());
                        i += 1;
                    }
                }
                "-u" | "--user" => {
                    if i + 1 < tokens.len() {
                        let auth = &tokens[i + 1];
                        let auth_header = format!("Authorization: Basic {}", base64_encode(auth));
                        if let Some((key, value)) = parse_header(&auth_header) {
                            headers.insert(key, value);
                        }
                        i += 1;
                    }
                }
                "-A" | "--user-agent" => {
                    if i + 1 < tokens.len() {
                        headers.insert("User-Agent".to_string(), tokens[i + 1].clone());
                        i += 1;
                    }
                }
                "-b" | "--cookie" => {
                    if i + 1 < tokens.len() {
                        headers.insert("Cookie".to_string(), tokens[i + 1].clone());
                        i += 1;
                    }
                }
                "--compressed" => {
                    headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
                }
                "-k" | "--insecure" => {}
                "-s" | "--silent" | "-v" | "--verbose" | "-i" | "--include" | "-L"
                | "--location" | "-f" | "--fail" | "--max-time" | "--connect-timeout" => {}
                _ => {
                    if token.starts_with("-") && !token.starts_with("--") {
                        let flag = token.strip_prefix("-").unwrap_or("");
                        for c in flag.chars() {
                            match c {
                                'X' => {
                                    if i + 1 < tokens.len() {
                                        method = HttpMethod::from_str(&tokens[i + 1]);
                                        i += 1;
                                    }
                                }
                                'H' => {
                                    if i + 1 < tokens.len() {
                                        if let Some((key, value)) = parse_header(&tokens[i + 1]) {
                                            headers.insert(key, value);
                                        }
                                        i += 1;
                                    }
                                }
                                'd' => {
                                    if i + 1 < tokens.len() {
                                        data = Some(tokens[i + 1].clone());
                                        i += 1;
                                    }
                                }
                                'v' | 'i' | 's' | 'k' | 'L' | 'f' => {}
                                _ => {}
                            }
                        }
                    } else if !token.starts_with("--") && url.is_none() {
                        url = Some(token.clone());
                    }
                }
            }
            i += 1;
        }

        if let Some(url_str) = url {
            tool.url = url_str;
        } else {
            return Err("URL not found in curl command".to_string());
        }

        if let Some(data_str) = data {
            tool.body = data_str.into_bytes();
        } else if !form_data.is_empty() {
            let mut body_parts = Vec::new();
            for form in &form_data {
                body_parts.push(form.clone());
            }
            tool.body = body_parts.join("&").into_bytes();
            headers.insert(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            );
        }

        tool.headers = headers;

        if !tool.body.is_empty() && method == HttpMethod::GET {
            tool.method = HttpMethod::POST;
        } else {
            tool.method = method;
        }

        Ok(tool)
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
            .method(http_client::Method::from(self.method.clone()))
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

fn tokenize_curl(curl_command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    for ch in curl_command.chars() {
        if escape_next {
            current_token.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_double_quote || in_single_quote => {
                escape_next = true;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' | '\n' | '\r' if !in_single_quote && !in_double_quote => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

fn parse_header(header: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = header.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}

fn base64_encode(input: &str) -> String {
    let mut encoded = String::new();
    let input_bytes = input.as_bytes();
    let mut i = 0;

    while i + 3 <= input_bytes.len() {
        let chunk = &input_bytes[i..i + 3];
        encoded.push(encode_base64_char((chunk[0] >> 2) as u32));
        encoded.push(encode_base64_char(
            (((chunk[0] & 0x03) << 4) | (chunk[1] >> 4)) as u32,
        ));
        encoded.push(encode_base64_char(
            (((chunk[1] & 0x0F) << 2) | (chunk[2] >> 6)) as u32,
        ));
        encoded.push(encode_base64_char((chunk[2] & 0x3F) as u32));
        i += 3;
    }

    if i + 2 == input_bytes.len() {
        let chunk = &input_bytes[i..i + 2];
        encoded.push(encode_base64_char((chunk[0] >> 2) as u32));
        encoded.push(encode_base64_char(
            (((chunk[0] & 0x03) << 4) | (chunk[1] >> 4)) as u32,
        ));
        encoded.push(encode_base64_char(((chunk[1] & 0x0F) << 2) as u32));
        encoded.push('=');
    } else if i + 1 == input_bytes.len() {
        let chunk = &input_bytes[i..i + 1];
        encoded.push(encode_base64_char((chunk[0] >> 2) as u32));
        encoded.push(encode_base64_char(((chunk[0] & 0x03) << 4) as u32));
        encoded.push('=');
        encoded.push('=');
    }

    encoded
}

fn encode_base64_char(value: u32) -> char {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    BASE64_CHARS[(value & 0x3F) as usize] as char
}

#[cfg(test)]
mod http_test {
    use super::*;

    #[test]
    fn test_curl() {
        let curl_command = r#"curl -X POST 'https://weav3r.dev/favorites' -H 'User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36 Edg/144.0.0.0' -H 'Accept: text/x-component' -H 'accept-language: en' -H 'content-type: text/plain;charset=UTF-8' -H 'dnt: 1' -H 'next-action: 40b56cda62a77de9e1724496c1e9fdea42e89ab88a' -H 'Cookie: __tf_verified=1770523246123.d9523b2d4ad89d39d12ffb5149f3f389c07d041a96110aaa958c6e1ceed6cd32; _ga=GA1.1.302505827.1770436850; cf_clearance=w.lAzd_L1NsCH.uSgBVOHmVFGk_gzv90AGKBgEP5NBA-1770436848-1.2.1.1-9inhhxy4h4GnLkeLOJhKCkGtZJMXwTNm6eu0pRfZfPzB3jFInbUZSwZDQFgvmqcspU5Lzp7wxcrag8tDeQbTzw1jNM7TLs.EC7sdEXcLQEbfcuESOG5lotPBIys94zlsqIf8h5IaYYanXWGAH1eM6EQPEVUzS.zetr0EKsiqOprrLJMZZ9C5zFKVJPmlmS1yPzDGpoQf532zhcJX4BUTmuAB10WCmJmlxT_qG9eZ7E4; _ga_PF693NSPW1=GS2.1.s1770436850$o1$g1$t1770436880$j30$l0$h0' -d '[
    [
        385,
        183
    ]
]'"#;
        let tool = HttpTool::from_curl(curl_command);
        assert!(tool.is_ok());
        let tool = tool.unwrap();
        println!("tool info:{:?}", tool);
        assert_eq!(tool.method, HttpMethod::POST);
        assert_eq!(tool.url, "https://weav3r.dev/favorites");
    }
}
