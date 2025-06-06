use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
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
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

pub async fn spawn_app() -> TestApp {
    dotenv::dotenv().ok();
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to get configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };
    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone()).await.expect("Failed to build application");
    let port = application.port();
    tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("http://0.0.0.0:{}", port),
        port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
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

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url) .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "0.0.0.0");
            confirmation_link.set_port(Some(self.port)).unwrap(); confirmation_link

        };
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks {
            html,
            plain_text
        }
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}