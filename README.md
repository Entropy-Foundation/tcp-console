# TCP Console

`tcp-console` is a simple TCP-based interface designed for applications that need an external control channel. It provides a way to send commands to a running application to alter its behavior or fetch operational data. This project addresses these requirements by supporting both strongly-typed commands (structured data) and plain text commands for flexibility.

The console server listens for TCP connections on `localhost` (127.0.0.1) and handles incoming data by parsing commands and processing them. A response is optionally sent back to the client based on the command type.

---

## Features

- **Command Injection**: Allows external control of an application via TCP.
- **Supports Typed and Text Commands**: Accepts strongly-typed commands and plain text commands for quick use cases.
- **Async Networking**: Uses `tokio` for handling multiple simultaneous connections efficiently.
- **Examples Provided**: The `examples` directory contains a demonstration of both plain text and structured command handling. One of the command handlers is showcased to report data to the remote connection.

---

## Configuration Example

```rust
use tcp_console as console;
use tcp_console::Subscription;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = 3838;

    let console = console::Builder::new()
        .port(port)
        .welcome("Welcome to TCP console!")
        .subscribe(Services::Logger, Logger)?
        .subscribe(Services::Exec, Exec)?
        .build()?;

    console.spawn().await?;
    console.stop();

    Ok(())
}
```

In this example, `Logger` and `Exec` are types that implement the `Subscription` trait, enabling them to handle specific types of commands sent to the console.