use std::time::Duration;
use reqwest::Client;
use serde::Serialize;
use crate::damain::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
}

impl EmailClient {
    pub fn new(sender: SubscriberEmail, base_url: String, timeout: Duration) -> Self {
        Self {
            sender,
            http_client: Client::builder()
                .timeout(timeout)
                .build()
                .unwrap(),
            base_url,
        }
    }
    pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_content: &str, text_content: &str) -> Result<(), reqwest::Error>{
        let address = format!("{}/email", self.base_url);
        let body = SendEmailRequest{
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        let response = self.http_client
            .post(address)
            .json(&body)
            .send()
            .await?;
        
        response.error_for_status()?;
        Ok(())
    }
}


#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}


#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Sentence;
    use fake::faker::lorem::en::Paragraph;
    use serde_json::Value;
    use wiremock::Request;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use super::*;
    use wiremock::matchers::{any, path, method, header};
    
    struct SendEmailBodyMatcher;
    
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<Value, _> = serde_json::from_slice(&request.body);
            match result {
                Ok(body) => {
                    body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
                }
                _ => false,
            }
        }
    }

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }
    /// Generate a random subscriber email
    fn email() -> SubscriberEmail { 
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }
    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient { 
        EmailClient::new(email(), base_url, Duration::from_millis(200))
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        
        Mock::given(any())
            .and(method("POST"))
            .and(path("/email"))
            .and(header("Content-Type", "application/json"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        email_client.send_email(email(), &subject(), &content(), &content()).await.unwrap();
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        let response = email_client.send_email(email(), &subject(), &content(), &content()).await;
        
        assert_ok!(response);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        
        let response = email_client.send_email(email(), &subject(), &content(), &content()).await;
        
        assert_err!(response);
    }
}