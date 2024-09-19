use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use std::fs::File;
use std::str;
use std::net::SocketAddr;
use fern::Dispatch;
use log::{info, error};
mod test4;

async fn handle_connection(client_stream: TcpStream, target_address: &str) {
    let mut target_stream = match TcpStream::connect(target_address).await {
        Ok(stream) => stream,
        Err(e) => {
            // eprintln!("Failed to connect to target server: {}", e);
            info!("Failed to connect to target server: {}", e);
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
                info!("Request to forward: {}", request_str);
            }

            if let Err(e) = target_writer.write_all(&buffer[..n]).await {
                // eprintln!("Failed to write to target server: {}", e);
                info!("Failed to write to target server: {}", e);
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
                info!( "Response from target: {}", response_str);
            }

            if let Err(e) = client_writer.write_all(&buffer[..n]).await {
                // eprintln!("Failed to write to client: {}", e);
                info!("Failed to write to client: {}", e);
                break;
            }
        }
    });

    let _ = tokio::try_join!(client_to_target, target_to_client);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 创建文件和日志配置
    let file = File::create("app.log")?;
    let file = Arc::new(Mutex::new(file));

    let file_clone = Arc::clone(&file);

    Dispatch::new()
        .format(move |out, message, record| {
            let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
            let level = record.level();
            let msg = message.to_string();
            let mut file = file_clone.lock().unwrap();
            writeln!(file, "[{}][{}] {}", timestamp, level, msg).ok();
            out.finish(format_args!("{} - {}", timestamp, msg))
        })
        .chain(std::io::stdout()) // 输出到标准输出
        .apply().expect("初始化错误");


    // 获取监听端口和目标服务器地址
    let port = 32632;
    let target_address = "aleo-asia.f2pool.com:4400";
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("Invalid address");
    // 创建 TCP 监听器
    let listener = TcpListener::bind(&addr).await?;

    info!("Proxy server listening on http://{}", addr);
    // 接收客户端连接并处理
    loop {
        let (client_stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(client_stream, target_address));
    }
}

