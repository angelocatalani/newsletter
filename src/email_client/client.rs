use crate::domain::SubscriberEmail;
use crate::email_client::EmailClientError;
use reqwest::{
    Client,
    Url,
};
use serde_json::json;

pub struct EmailClient {
    http_client: Client,
    base_url: Url,
    sender: SubscriberEmail,
    token: String,
}
impl EmailClient {
    pub fn new(base_url: Url, sender: SubscriberEmail, token: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            token,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        let body = json!({
            "From": self.sender.as_ref(),
            "To": recipient.as_ref(),
            "Subject": subject,
            "TextBody": html_content,
            "HtmlBody": text_content
        });
        let smtp_response = self
            .http_client
            .post(self.base_url.join("email")?)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("X-Postmark-Server-Token", self.token.as_str())
            .json(&body)
            .send()
            .await?;
        if smtp_response.status().is_success() {
            Ok(())
        } else {
            Err(EmailClientError::ErrorResponse {
                canonical_reason: smtp_response
                    .status()
                    .canonical_reason()
                    .unwrap_or("unknown_failure")
                    .to_string(),
                code: smtp_response.status().to_string(),
                is_client_error: smtp_response.status().is_client_error(),
                is_server_error: smtp_response.status().is_server_error(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::assert_ok;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{
        Paragraph,
        Sentence,
    };
    use fake::Fake;
    use reqwest::Url;
    use serde_json::json;
    use std::convert::TryFrom;
    use wiremock::matchers::body_json;
    use wiremock::matchers::{
        header,
        method,
        path,
    };
    use wiremock::{
        Mock,
        MockServer,
        ResponseTemplate,
    };

    #[tokio::test]
    async fn email_client_calls_smtp_server_with_the_correct_parameters() {
        let sender_email: String = SafeEmail().fake();
        let receiver_email: String = SafeEmail().fake();

        let sender = SubscriberEmail::try_from(sender_email).unwrap();
        let recipient = SubscriberEmail::try_from(receiver_email).unwrap();
        let token = String::from("token");

        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..2).fake();

        let body = json!({
            "From": sender.as_ref(),
            "To": recipient.as_ref(),
            "Subject": subject.as_str(),
            "TextBody": content.as_str(),
            "HtmlBody": content.as_str()
        });

        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/email"))
            .and(header("Accept", "application/json"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-Postmark-Server-Token", token.as_str()))
            .and(body_json(&body))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = EmailClient::new(Url::parse(&server.uri()).unwrap(), sender, token);

        assert_ok!(
            email_client
                .send_email(recipient, &subject, &content, &content)
                .await
        );
    }
}
