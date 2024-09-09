use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::fs::File;
use std::str;
use std::net::SocketAddr;


async fn handle_connection(client_stream: TcpStream, target_address: &str,log_file: Arc<Mutex<File>>) {
    // 创建与目标服务器的连接
    let mut target_stream = match TcpStream::connect(target_address).await {
        Ok(stream) => stream,
        Err(e) => {
            // eprintln!("Failed to connect to target server: {}", e);
            writeln!(log_file,"Failed to connect to target server: {}",e).expect("Failed to write to log file");
            return;
        }
    };
    // 解析内容
    let (mut client_reader, mut client_writer) = client_stream.into_split();
    let (mut target_reader, mut target_writer) = target_stream.into_split();

    // 解析请求信息
    let client_to_target = tokio::spawn(async move {
        let mut buffer = [0; 4096];
        while let Ok(n) = client_reader.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            if let Ok(request_str) = str::from_utf8(&buffer[..n]) {
                // println!("Request to forward: {}", request_str);
                writeln!(log_file,"Request to forward: {}", request_str).expect("Failed to write to log file");

            }

            if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                // eprintln!("Failed to write to target server: {}", e);
                writeln!(log_file,"Failed to write to target server: {}", e).expect("Failed to write to log file");
                break;
            }
        }
    });

    // 接收响应和解析响应值
    let target_to_client = tokio::spawn(async move {
        let mut buffer = [0; 4096];
        while let Ok(n) = target_reader.read(&mut buffer).await {
            if n == 0 {
                break; // EOF
            }
            // 解析
            if let Ok(response_str) = str::from_utf8(&buffer[..n]) {
                // println!("Response from target: {}", response_str);
                writeln!(log_file,"Response from target: {}", response_str).expect("Failed to write to log file");

            }

            if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                // eprintln!("Failed to write to client: {}", e);
                writeln!(log_file,"Failed to write to client: {}", e).expect("Failed to write to log file");
                break;
            }
        }
    });

    let _ = tokio::try_join!(client_to_target, target_to_client);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 获取监听端口和目标服务器地址
    let port = 9996;
    let target_address = "aleo-asia.f2pool.com:4400";
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");
    // 创建 TCP 监听器
    let listener = TcpListener::bind(&addr).await?;

    println!("Proxy server listening on http://{}", addr);

    // 打开日志文件
    let log_file = File::create("server.log")?;
    let log_file = Arc::new(Mutex::new(log_file));
    // 接收客户端连接并处理
    loop {
        let (client_stream, _) = listener.accept().await?;
        let log_file = Arc::clone(&log_file);
        tokio::spawn(handle_connection(client_stream, target_address,log_file));
    }
}
