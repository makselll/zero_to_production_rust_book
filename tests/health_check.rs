use std::net::TcpListener;
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero_to_production_rust_book::configuration::{get_configuration, DatabaseSettings, JaegerSettings};
use zero_to_production_rust_book::email_client::EmailClient;
use zero_to_production_rust_book::startup::run;
use zero_to_production_rust_book::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_layer = "info".to_string();
    let subscriber_name = "test".to_string();
    
    let jaeger_settings = JaegerSettings{address: "0.0.0.0".to_string(), port: 4317};
    
    if std::env::var("TEST_LOG").is_ok_and(|x| x.to_lowercase() == "true")  {
        let subscriber = get_subscriber(subscriber_name, default_filter_layer, std::io::stdout, jaeger_settings);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_layer, std::io::sink, jaeger_settings);
        init_subscriber(subscriber);
    }
});


struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    dotenv::dotenv().ok();
    Lazy::force(&TRACING);
    
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    
    let mut configuration = get_configuration().expect("Failed to get configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;
    
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.sender().expect("Failed to get email sender"),
        configuration.email_client.base_url,
        timeout
    );


    let server = run(listener, db_pool.clone(), email_client).expect("Failed to bind address");
    tokio::spawn(server);
    
    TestApp {
        address: format!("http://0.0.0.0:{}", port),
        db_pool,
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

#[tokio::test]
async fn health_check() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length()); }


#[tokio::test]
async fn subscribe_return_200_for_valid_form() {
    let app = spawn_app().await;
    assert_eq!(std::env::var("TEST_LOG").is_ok(), true, "TEST_LOG must be set to true");
    
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    let saved = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch subscription");

    assert_eq!(200, response.status().as_u16());
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (body, message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, response.status().as_u16(), "{}", message);
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_invalid() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new(); let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address)) .header("Content-Type", "application/x-www-form-urlencoded") .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(400, response.status().as_u16(),  "The API did not return a 200 OK when the payload was {}.", description);
    }
}