use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::TcpStream};
use anyhow::Result;
use std::io::{stdin, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    let mut name = String::new();
    reader.read_line(&mut name).await?;
    println!("Hello {}", name.trim());
    
    std::io::stdout().flush()?;
    let mut input_name = String::new();
    stdin().read_line(&mut input_name)?;
    write_half.write_all(input_name.as_bytes()).await?;
    
    loop {
        let mut buffer = String::new();
        let bytes_read = reader.read_line(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        print!("{}", buffer);
        if buffer.contains("'s move") {
            let mut input = String::new();
            print!("> ");
            std::io::stdout().flush()?;
            stdin().read_line(&mut input)?;
            write_half.write_all(input.as_bytes()).await?;
        }
    }
    Ok(())
}