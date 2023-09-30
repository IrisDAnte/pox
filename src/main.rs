use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        println!("{} {}",socket.local_addr().unwrap().to_string(), socket.peer_addr().unwrap().to_string());
        println!("client connection opened");

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            let mut buffer = [0; 1024];
            let n = reader.read(&mut buffer).await.unwrap();
            let zam = String::from_utf8_lossy(&buffer[..n]);

            println!("{}", zam);

            let target_address = zam
                .split("host: ")
                .nth(1)
                .unwrap()
                .split("\r\n")
                .next()
                .unwrap();

            let mut target_socket = TcpStream::connect(format!("{}:80", target_address)).await.unwrap();
            println!("target connection opened");

            target_socket.write_all(&buffer[..n]).await.unwrap();

            let (mut target_reader, mut target_writer) = target_socket.split();

            let client_to_target = async {
                let mut buffer = [0; 1024];
                loop {
                    let n = reader.read(&mut buffer).await.unwrap();
                    if n == 0 {
                        return;
                    }
                    target_writer.write_all(&buffer[..n]).await.unwrap();
                }
            };

            let target_to_client = async {
                let mut buffer = [0; 1024];
                loop {
                    let n = target_reader.read(&mut buffer).await.unwrap();
                    if n == 0 {
                        return;
                    }
                    writer.write_all(&buffer[..n]).await.unwrap();
                }
            };

            tokio::join!(client_to_target, target_to_client);

            println!("target connection closed");
            println!("client connection closed");
        });
    }
}
