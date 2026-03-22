use std::net::{Ipv4Addr, UdpSocket};

#[tokio::main]
async fn main() {
    let addr = Ipv4Addr::new(239, 0, 0, 1);
    let port: u16 = 9899;

    // Exact same setup as hlidskjalf_core::setup_multicast_socket()
    let std_socket = UdpSocket::bind(("0.0.0.0", port)).expect("bind");
    std_socket
        .join_multicast_v4(&addr, &Ipv4Addr::LOCALHOST)
        .expect("join");
    std_socket.set_nonblocking(true).expect("nonblocking");

    let socket = tokio::net::UdpSocket::from_std(std_socket).expect("from_std");

    println!("Listening on 239.0.0.1:{port} (tokio async)...");
    println!("Send a test: python3 /tmp/test_multicast.py");

    let mut buf = [0u8; 65535];
    match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        socket.recv_from(&mut buf),
    )
    .await
    {
        Ok(Ok((len, src))) => {
            let data = std::str::from_utf8(&buf[..len]).unwrap_or("<invalid utf8>");
            println!("RECEIVED {len} bytes from {src}: {}", &data[..data.len().min(120)]);
        }
        Ok(Err(e)) => println!("RECV ERROR: {e}"),
        Err(_) => println!("TIMEOUT: no data received in 15 seconds"),
    }
}
