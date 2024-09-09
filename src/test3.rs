use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// 转发请求到目标地址
fn forward_to_target(client_stream: TcpStream, target_stream: Arc<Mutex<TcpStream>>) {
    let client_stream = Arc::new(Mutex::new(client_stream));

    // 创建两个线程，一个负责从客户端读取并转发到目标地址，另一个负责从目标读取并转发给客户端
    let client_to_target = {
        let client_stream = Arc::clone(&client_stream);
        let target_stream = Arc::clone(&target_stream);
        thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                let bytes_read = client_stream.lock().unwrap().read(&mut buffer).expect("Error reading from client");
                if bytes_read == 0 {
                    break;
                }

                // 解析和打印请求数据
                let request_data = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Request data: {}", request_data);

                // 转发数据到目标服务器
                target_stream.lock().unwrap().write_all(&buffer[..bytes_read]).expect("Error writing to target");
            }
        })
    };

    let target_to_client = {
        let client_stream = Arc::clone(&client_stream);
        thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                let bytes_read = target_stream.lock().unwrap().read(&mut buffer).expect("Error reading from target");
                if bytes_read == 0 {
                    break;
                }

                // 转发目标服务器的响应回到客户端
                client_stream.lock().unwrap().write_all(&buffer[..bytes_read]).expect("Error writing to client");
            }
        })
    };

    client_to_target.join().expect("Client to target thread failed");
    target_to_client.join().expect("Target to client thread failed");
}

pub fn f2_proxy_v2() -> std::io::Result<()> {
    let proxy_address = "0.0.0.0:9996";  // 监听代理的地址
    let target_address = "aleo-asia.f2pool.com:4400";  // 目标服务器地址

    let listener = TcpListener::bind(proxy_address)?;

    println!("Proxy server listening on {}", proxy_address);

    for stream in listener.incoming() {
        match stream {
            Ok(client_stream) => {
                // 新建线程来处理每个客户端连接
                let target_stream = TcpStream::connect(target_address).expect("Could not connect to target");

                let target_stream = Arc::new(Mutex::new(target_stream));
                thread::spawn(move || {
                    forward_to_target(client_stream, Arc::clone(&target_stream));
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
