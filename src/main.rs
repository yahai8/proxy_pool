use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io;
use std::str;
use std::net::SocketAddr;


async fn handle_connection(client_stream: TcpStream, target_address: &str, user_keys: Arc<Mutex<HashMap<String, String>>>, key: String) {
    // 创建与目标服务器的连接
    let mut target_stream = match TcpStream::connect(target_address).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Failed to connect to target server: {}", e);
            return;
        }
    };

    // 将用户的 key 存储在 HashMap 中
    {
        let mut user_keys = user_keys.lock().unwrap();
        user_keys.insert(key.clone(), key);
    }

    // Split streams into read and write halves
    let (mut client_reader, mut client_writer) = client_stream.into_split();
    let (mut target_reader, mut target_writer) = target_stream.into_split();

    // Forward data from client to target and parse request
    let client_to_target = tokio::spawn(async move {
        let mut buffer = [0; 4096];
        while let Ok(n) = client_reader.read(&mut buffer).await {
            if n == 0 {
                break; // EOF
            }
            // Parse and print request data (for debugging)
            if let Ok(request_str) = str::from_utf8(&buffer[..n]) {
                println!("Request to forward: {}", request_str);
            }

            if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                eprintln!("Failed to write to target server: {}", e);
                break;
            }
        }
    });

    // Forward data from target to client and parse response
    let target_to_client = tokio::spawn(async move {
        let mut buffer = [0; 4096];
        while let Ok(n) = target_reader.read(&mut buffer).await {
            if n == 0 {
                break; // EOF
            }
            // Parse and print response data (for debugging)
            if let Ok(response_str) = str::from_utf8(&buffer[..n]) {
                println!("Response from target: {}", response_str);
            }

            if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                eprintln!("Failed to write to client: {}", e);
                break;
            }
        }
    });

    // Wait for both tasks to complete
    let _ = tokio::try_join!(client_to_target, target_to_client);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 获取监听端口和目标服务器地址
    let port = 9996;
    let target_address = "aleo-asia.f2pool.com:4400";
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");

    // 创建一个 HashMap 用于存储用户的 key
    let user_keys: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    // 创建 TCP 监听器
    let listener = TcpListener::bind(&addr).await?;

    println!("Proxy server listening on http://{}", addr);

    // 接收客户端连接并处理
    loop {
        let (client_stream, _) = listener.accept().await?;
        let user_keys = Arc::clone(&user_keys);

        // 提取 key（假设通过环境变量传递）
        let key = match std::env::var("KEY") {
            Ok(val) => val,
            Err(_) => "default_key".to_string(),
        };

        tokio::spawn(handle_connection(client_stream, target_address, user_keys, key));
    }
}
