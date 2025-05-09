use std::net::TcpListener;
use zero_to_production_rust_book::run;

#[tokio::test]
async fn health_check() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length()); }

fn spawn_app() -> String{
    let listener = TcpListener::bind("0.0.0.0:0").expect("Failed to bind address");
    let port = listener.local_addr().unwrap().port();
    
    let server = run(listener).expect("Failed to bind address");
    tokio::spawn(server);
    
    format!("http://0.0.0.0:{}", port)
}