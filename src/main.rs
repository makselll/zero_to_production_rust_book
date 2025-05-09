use std::net::TcpListener;
use zero_to_production_rust_book::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    run(listener)?.await
}
