use zero_to_production_rust_book::configuration::get_configuration;
use zero_to_production_rust_book::startup::Application;
use zero_to_production_rust_book::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let configuration = get_configuration().expect("Failed to get configuration");

    let subscriber = get_subscriber("zero2prod".into(), "info".to_string(), std::io::stdout, &configuration.jaeger);
    init_subscriber(subscriber);

    Application::build(configuration).await?.run_until_stopped().await?;
    Ok(())
}