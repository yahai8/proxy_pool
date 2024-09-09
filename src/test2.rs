use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use serde_json::Value;

// 处理TCP请求
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];  // 定义一个缓冲区
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            // 将数据转为字符串，处理有效载荷
            let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);

            println!("Received data: {}", received_data);

            // 尝试将数据解析为 JSON (Stratum 协议基于 JSON-RPC)
            if let Ok(json_data) = serde_json::from_str::<Value>(&received_data) {
                println!("Parsed JSON: {}", json_data);

                // 根据接收到的数据，你可以进行进一步的逻辑处理，比如过滤特定消息
                // 或发送回复。例如，发送一个 "OK" 响应:
                let response = r#"{"id":1,"result":true,"error":null}"#;
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            } else {
                println!("Failed to parse JSON");
            }
        }
        Err(e) => {
            println!("Failed to read from socket: {:?}", e);
        }
    }
}

pub fn f2_proxy() -> std::io::Result<()> {
    // 创建一个监听 TCP 连接的 TcpListener
    let listener = TcpListener::bind("0.0.0.0:9996")?;  // 监听9996端口

    println!("Listening on port 9996");

    // 接收连接请求
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                handle_client(stream);  // 处理接收到的连接
            }
            Err(e) => {
                println!("Connection failed: {:?}", e);
            }
        }
    }

    Ok(())
}
