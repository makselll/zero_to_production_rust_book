use std::net::TcpListener;
use secrecy::ExposeSecret;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use zero_to_production_rust_book::configuration::get_configuration;
use zero_to_production_rust_book::startup::run;
use zero_to_production_rust_book::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let configuration = get_configuration().expect("Failed to get configuration");
    
    let subscriber = get_subscriber("zero2prod".into(), "info".to_string(), std::io::stdout, configuration.jaeger);
    init_subscriber(subscriber);

    let address = format!("{}:{}", configuration.application.address, configuration.application.port);
    let listener = TcpListener::bind(address)?;

    let db_pool = PgPoolOptions::new()
        .connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to database");
    run(listener, db_pool)?.await
}