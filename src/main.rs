use std::net::TcpListener;
use sqlx::PgPool;
use zero_to_production_rust_book::configuration::get_configuration;
use zero_to_production_rust_book::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    
    
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    run(listener, db_pool)?.await
}
