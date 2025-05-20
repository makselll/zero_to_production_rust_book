use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_return_200_for_valid_form() {
    let app = spawn_app().await;
    
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

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
    
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];

    for (body, message) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
        assert_eq!(400, response.status().as_u16(), "{}", message);
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;
        assert_eq!(400, response.status().as_u16(),  "The API did not return a 200 OK when the payload was {}.", description);
    }
}