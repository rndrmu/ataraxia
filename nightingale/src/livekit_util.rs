use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn spawn_ipv4_proxy() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move {
        loop {
            let Ok((client, _)) = listener.accept().await else { break };
            tokio::spawn(handle_conn(client));
        }
    });
    Ok(port)
}

async fn handle_conn(mut client: TcpStream) {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1];
    loop {
        if client.read_exact(&mut tmp).await.is_err() { return; }
        buf.push(tmp[0]);
        if buf.ends_with(b"\r\n\r\n") { break; }
        if buf.len() > 4096 { return; }
    }

    let req = String::from_utf8_lossy(&buf);
    let first_line = req.lines().next().unwrap_or("");
    let mut parts = first_line.split_whitespace();
    if parts.next() != Some("CONNECT") { return; }
    let target = match parts.next() { Some(t) => t, None => return };
    let (host, port_str) = match target.rsplit_once(':') { Some(p) => p, None => return };
    let port: u16 = match port_str.parse() { Ok(p) => p, Err(_) => return };

    let addrs: Vec<_> = match tokio::net::lookup_host(format!("{}:{}", host, port)).await {
        Ok(a) => a.collect(),
        Err(_) => return,
    };
    let addr = match addrs.iter().find(|a| a.is_ipv4()).or_else(|| addrs.first()) {
        Some(a) => *a,
        None => return,
    };

    let mut server = match TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(_) => { let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await; return; }
    };

    let _ = client.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await;
    let _ = tokio::io::copy_bidirectional(&mut client, &mut server).await;
}
