use std::net::TcpListener;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing_actix_web::TracingLogger;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscriptions, subscriptions_confirm};


pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}


pub fn run(listener: TcpListener, db_pool: PgPool, email_client: EmailClient, base_url: String) -> std::io::Result<Server> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
            .route("/subscriptions/confirm", web::get().to(subscriptions_confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub struct Application {
    port: u16,
    server: Server,
}

pub struct ApplicationBaseUrl(pub String);


impl Application {
    pub async fn build(configuration: Settings) -> Result<Application, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration.email_client.sender().expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            sender_email,
            configuration.email_client.base_url,
            timeout
        );

        let address = format!( "{}:{}", configuration.application.address, configuration.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();

        let server = run(listener, connection_pool, email_client, configuration.application.base_url)?;
        Ok(Self {port, server})
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }

}

