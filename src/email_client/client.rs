use std::time::Duration;

use reqwest::{
    Client,
    Url,
};
use serde::Serialize;

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClientError;

pub struct EmailClient {
    http_client: Client,
    base_url: Url,
    sender: SubscriberEmail,
    token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    text_body: &'a str,
    html_body: &'a str,
}

impl EmailClient {
    pub fn new(base_url: Url, sender: SubscriberEmail, token: String, timeout_secs: u64) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap_or_else(|error| {
                    panic!(
                        "unrecoverable error: {} creating mail client with parameters: base_url: \
                         {} sender: {:#?} token: {} timeout_secs: {}",
                        error, base_url, sender, token, timeout_secs,
                    )
                }),
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
        let request = EmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            text_body: text_content,
            html_body: html_content,
        };
        let smtp_response = self
            .http_client
            .post(self.base_url.join("email")?)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("X-Postmark-Server-Token", self.token.as_str())
            .json(&request)
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
    use std::convert::TryFrom;

    use claim::{
        assert_matches,
        assert_ok,
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{
        Paragraph,
        Sentence,
    };
    use fake::Fake;
    use reqwest::{
        StatusCode,
        Url,
    };
    use serde_json::{
        json,
        Value,
    };
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

    use crate::domain::SubscriberEmail;

    use super::*;

    fn email() -> SubscriberEmail {
        let sender_email: String = SafeEmail().fake();
        SubscriberEmail::try_from(sender_email).unwrap()
    }
    fn sentence() -> String {
        Sentence(1..2).fake()
    }
    fn paragraph() -> String {
        Paragraph(1..2).fake()
    }
    fn token() -> String {
        String::from("token")
    }
    fn body_from(
        sender: &SubscriberEmail,
        recipient: &SubscriberEmail,
        subject: &str,
        content: &str,
    ) -> Value {
        json!({
            "From": sender.as_ref(),
            "To": recipient.as_ref(),
            "Subject": subject,
            "TextBody": content,
            "HtmlBody": content
        })
    }

    #[tokio::test]
    async fn email_client_performs_the_correct_request() {
        let token = token();
        let subject = sentence();
        let content = paragraph();
        let sender = email();
        let recipient = email();

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/email"))
            .and(header("Accept", "application/json"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-Postmark-Server-Token", token.as_str()))
            .and(body_json(&body_from(
                &sender, &recipient, &subject, &content,
            )))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = EmailClient::new(Url::parse(&server.uri()).unwrap(), sender, token, 10);

        assert_ok!(
            email_client
                .send_email(recipient, &subject, &content, &content)
                .await
        );
    }

    #[tokio::test]
    async fn email_client_handles_error_response() {
        for status_code in [StatusCode::INTERNAL_SERVER_ERROR, StatusCode::NOT_FOUND].iter() {
            let server = MockServer::start().await;

            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status_code.as_u16()))
                .expect(1)
                .mount(&server)
                .await;

            let email_client =
                EmailClient::new(Url::parse(&server.uri()).unwrap(), email(), token(), 10);

            let response = email_client
                .send_email(email(), &sentence(), &paragraph(), &paragraph())
                .await;

            assert_matches!(
               response.unwrap_err(),
               EmailClientError::ErrorResponse {
                   canonical_reason,
                   code,
                   is_client_error,
                   is_server_error
               } if canonical_reason==status_code.canonical_reason().unwrap()
                    && code==status_code.to_string()
                    && is_client_error==status_code.is_client_error()
                    && is_server_error==status_code.is_server_error()
            );
        }
    }

    #[tokio::test]
    async fn email_client_handles_timeout() {
        let server = MockServer::start().await;
        let delay = 4;
        let timeout = 2;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(delay)))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = EmailClient::new(
            Url::parse(&server.uri()).unwrap(),
            email(),
            token(),
            timeout,
        );

        let response = email_client
            .send_email(email(), &sentence(), &paragraph(), &paragraph())
            .await;

        assert_matches!(
            response.unwrap_err(),
            EmailClientError::InvalidRequest {
                source
            } if source.is_timeout()
        );
    }
}
