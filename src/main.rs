use anyhow::*;
use futures::StreamExt;
use quinn::{ Certificate, ClientConfigBuilder, Endpoint, NewConnection };
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io::{ stdin };

#[tokio::main]
async fn main() -> Result<(), Error> {


    // QUICの設定
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut client_config = ClientConfigBuilder::default();
    client_config.add_certificate_authority(Certificate::from_der(&std::fs::read("key/server-crt.pem.der")?)?)?;
    let mut endpoint_builder = Endpoint::builder();
    endpoint_builder.default_client_config(client_config.build());
    let (endpoint, _incoming) = endpoint_builder.bind(&"0.0.0.0:0".parse().unwrap())?;

    loop {

        // CUIチャット
    let mut input = String::from("");
    stdin().read_line(&mut input).ok();
    let result = input.trim().to_string();  

    if result == "off" {
        break;
    }

    // サーバ接続
    let NewConnection {
        connection,
        mut uni_streams,
        ..
    } = endpoint.connect(&socket, "localhost")?.await?;
    println!("connected: addr={}", connection.remote_address());

    // メッセージ
    //let msg = "HELLO QUIC PROTOCOL!!";
    let msg = result;
    let mut send_stream = connection.open_uni().await?;
    send_stream.write(msg.as_bytes()).await?;
    send_stream.finish().await?;


    // 返信読み込み
    if let Some(uni_stream) = uni_streams.next().await {
        let uni_stream = uni_stream?;
        let data = uni_stream.read_to_end(0xFF).await?;
        println!("received\"{}\"", String::from_utf8_lossy(&data));
    } else {
        bail!("cannot open uni stream!!");
    }

    endpoint.wait_idle().await;

    }
    

    Ok(())
}
