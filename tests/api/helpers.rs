use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero_to_production_rust_book::configuration::{get_configuration, DatabaseSettings, JaegerSettings};
use zero_to_production_rust_book::startup::{get_connection_pool, Application};
use zero_to_production_rust_book::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_layer = "info".to_string();
    let subscriber_name = "test".to_string();

    let jaeger_settings = JaegerSettings{address: "0.0.0.0".to_string(), port: 4317};

    if std::env::var("TEST_LOG").is_ok_and(|x| x.to_lowercase() == "true")  {
        let subscriber = get_subscriber(subscriber_name, default_filter_layer, std::io::stdout, &jaeger_settings);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_layer, std::io::sink, &jaeger_settings);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    dotenv::dotenv().ok();
    Lazy::force(&TRACING);
    
    let configuration = {
        let mut c = get_configuration().expect("Failed to get configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone()).await.expect("Failed to build application");
    let port = application.port();
    tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: get_connection_pool(&configuration.database),
    }
}


pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create a database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    } 
}