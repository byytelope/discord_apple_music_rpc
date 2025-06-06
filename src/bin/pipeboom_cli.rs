use pipeboom::ipc::commands::{IpcCommand, IpcMessage, IpcResponse};
use std::env;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command>", args[0]);
        eprintln!("Commands: start, stop, current-song, status, shutdown");
        return Ok(());
    }

    let command = match args[1].as_str() {
        "start" => IpcCommand::Start,
        "stop" => IpcCommand::Stop,
        "current-song" => IpcCommand::CurrentSong,
        "status" => IpcCommand::Status,
        "shutdown" => IpcCommand::Shutdown,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            return Ok(());
        }
    };

    let socket_path = std::env::temp_dir().join("pipeboom.sock");
    send_command(socket_path, command).await?;

    Ok(())
}

async fn send_command(
    socket_path: PathBuf,
    command: IpcCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let message = IpcMessage { command };
    let message_json = serde_json::to_string(&message)?;

    stream.write_all(message_json.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    let response: IpcResponse = serde_json::from_str(response.trim())?;
    println!("{:#?}", response);

    Ok(())
}
