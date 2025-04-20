use std::collections::HashMap;
use std::net::TcpStream;
use std::io::Write;
use flate2::{Compression, write::GzEncoder};

pub struct Response{
    pub status_code: usize,
    headers: HashMap<String, String>,
    pub body: Vec<u8>
}

impl Response {
    pub fn new(status_code: usize, body: &str) -> Response{
        return Response{
            status_code,
            headers: HashMap::new(),
            body: body.as_bytes().to_vec()
        };
    }

    fn get_msg(&self) -> String{
        return match self.status_code {
            200 => "OK",
            201 => "Created",
            404 => "Not Found",
            400 => "Bad Request",
            _ => ""
        }.to_string()
    }

    pub fn add_header(&mut self, key: &str, val: &str) {
        self.headers.insert(key.to_string(), val.to_string());
    }

    pub fn send(self, mut stream: &TcpStream) {
        let mut header_string = String::new();
        for (key, value) in &self.headers{
            header_string.push_str(format!("{}: {}\r\n", key, value).as_str());
        }

        stream.write_all(format!("HTTP/1.1 {} {}\r\n{}\r\n", self.status_code, self.get_msg(), header_string).as_bytes()).unwrap();
        stream.write_all(&self.body).unwrap();
    }

    pub fn encode_body(&mut self, encoding: &str) -> bool{
        let supported_encodings: [&str;1] = ["gzip"];

        if !supported_encodings.contains(&encoding) {return false};
    
        if encoding == "gzip"{
            self.add_header("Content-Encoding", encoding);
            let mut gzip_encoder = GzEncoder::new(Vec::new(), Compression::default());
            gzip_encoder.write_all(&self.body).unwrap();
            self.body = gzip_encoder.finish().unwrap();
        }

        if self.body.len() > 0{
            self.add_header("Content-Length", self.body.len().to_string().as_str());
        }
        return true;
    }
}

pub fn create_200(content_type: &str, content: &str) -> Response {
    let mut res = Response::new(200, content);
    if content_type.len() > 0{
        res.add_header("Content-Type", content_type);
    }
    if content.len() > 0{
        res.add_header("Content-Length", content.len().to_string().as_str());
    }
    return res;
}

pub fn create_404() -> Response{
    return Response::new(404, "");
}
