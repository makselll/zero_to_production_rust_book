use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero_to_production_rust_book::configuration::get_configuration;
use zero_to_production_rust_book::startup::run;
use zero_to_production_rust_book::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    let subscriber = get_subscriber("zero2prod".into(), "info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to get configuration");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    
    let db_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to database");
    run(listener, db_pool)?.await
}