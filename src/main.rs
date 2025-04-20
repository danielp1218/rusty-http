#[allow(unused_imports)]
use flate2::{Compression, write::GzEncoder};
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::collections::HashMap;
use std::thread;
use std::env;
use std::fs;
use http_server::{Response, create_200, create_404};

fn handle_req(request_type: &str, path: &str, headers: &HashMap<String, String>, body: Vec<u8>) -> Response {
    let mut res: Response = Response::new(500, "Internal Server Error");
    match path{
        "/" => {res = Response::new(200, "")},

        _ if path.starts_with("/echo/") => {
            let echo = path.strip_prefix("/echo/").unwrap();
            res = create_200("text/plain", echo);
        },

        _ if path.starts_with("/user-agent") =>{
            if let Some(user_agent) = headers.get("user-agent"){
                res = create_200("text/plain", user_agent);
            } else{
                //do something
                println!("couldnt find user-agent");
            }
        }

        _ if path.starts_with("/files/") => {
            let file_path = path.strip_prefix("/files/").unwrap();
            if request_type == "POST"{
                fs::write(file_path, body).unwrap();
                res = Response::new(201, "");
            } else if request_type == "GET" {
                if let Ok(content) = fs::read_to_string(file_path){
                    res = create_200( "application/octet-stream", content.as_str());
                } else{
                    res = create_404();
                }
            } else{
                res = create_404();
            }
        }
        _ => {res = create_404()}
    }

    if let Some(encodings) = headers.get("accept-encoding") {
        let encoding_list: Vec<&str> = encodings.split(", ").collect();
        
        for encoding in encoding_list {
            if res.encode_body(encoding) {break};
        }
    }
    
    if let Some(closed) = headers.get("connection"){
        res.add_header("Connection", closed.as_str());
    }

    return res;

}


fn handle_connection(mut stream: TcpStream){
    println!("accepted new connection");
    let mut buf = [0 ; 512];

    while let Ok(req_size) = stream.read(&mut buf) {
        println!("req size: {}", req_size);
        if let Some(pos) = buf.windows(4).position(|x| x==b"\r\n\r\n"){
            let body = buf[pos+4..req_size].to_vec();
            let header_str = String::from_utf8_lossy(&buf[..pos]).to_string();

            println!("Headers:\n{}", header_str);
            println!("body:{}", String::from_utf8_lossy(&body));

            let mut lines = header_str.lines();
            let mut request: std::str::Split<'_, &str> = lines.next().unwrap().split(" ");
            let request_type = request.next().unwrap();
            let path = request.next().unwrap();
            let mut headers: HashMap<String, String> = HashMap::new();

            for line in lines {
                if line.trim().len() > 0 {
                    if let Some(ind) = line.find(": ") {
                        let (str1, str2) = line.split_at(ind);
                        headers.insert(str1.trim().to_ascii_lowercase(), str2.strip_prefix(":").unwrap().trim().to_ascii_lowercase());
                    }
                }
            }

            handle_req(request_type.to_uppercase().as_str(), path, &headers, body).send(&stream);

            if let Some(closed) = headers.get("connection"){
                if closed == "close"{
                    break;
                }
            }
        } else{
            Response::new(400, "").send(&stream);
        }
        
        
    }
    
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut arg_map: HashMap<String, String> = HashMap::new();
    let mut i = 1;
    while i < args.len() {
        if args[i].starts_with("--") {
            arg_map.insert(args[i].clone(), args[i + 1].clone());
            i += 1;
        }
        i += 1;
    }

    if let Some(dir) = arg_map.get("--directory"){
        env::set_current_dir(dir).unwrap();
        println!("Directory: {}", dir);
    }

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_connection(stream));
                
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
