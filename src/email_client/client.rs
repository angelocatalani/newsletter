use std::time::Duration;

use anyhow::Context;
use reqwest::{
    Client,
    Url,
};

use crate::domain::SubscriberEmail;
use crate::email_client::request::EmailRequest;
use derivative::Derivative;

#[derive(Derivative, Debug)]
pub struct EmailClient {
    http_client: Client,
    base_url: Url,
    sender: SubscriberEmail,
    #[derivative(Debug = "ignore")]
    token: String,
}

impl EmailClient {
    pub fn new(
        base_url: Url,
        sender: SubscriberEmail,
        token: String,
        timeout_secs: u64,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .context(format!(
                    "Error creating mail client with:\nbase_url: {}\nsender: {}\ntoken: \
                     {}***\ntimeout_secs: {}",
                    base_url,
                    sender.as_ref(),
                    &token[0..2],
                    timeout_secs
                ))?,
            base_url,
            sender,
            token,
        })
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_part: &str,
        text_part: &str,
    ) -> Result<(), anyhow::Error> {
        let smtp_response = self
            .http_client
            .post(self.base_url.join("send")?)
            .header("Content-Type", "application/json")
            .header("Authorization", self.token.as_str())
            .json(&EmailRequest::new(
                self.sender.as_ref(),
                recipient.as_ref(),
                subject,
                html_part,
                text_part,
            ))
            .send()
            .await?;
        smtp_response.error_for_status().context(format!(
            "Error sending email to: {} with subject: {}",
            recipient.as_ref(),
            subject
        ))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use claim::assert_ok;
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

    #[tokio::test]
    async fn email_client_performs_the_correct_request() {
        let token = token();
        let subject = sentence();
        let content = paragraph();
        let sender = email();
        let recipient = email();

        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/send"))
            .and(header("Content-Type", "application/json"))
            .and(header("Authorization", token.as_str()))
            .and(body_json(&EmailRequest::new(
                sender.as_ref(),
                recipient.as_ref(),
                &subject,
                &content,
                &content,
            )))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let email_client = EmailClient::new(
            Url::parse(&server.uri()).unwrap(),
            sender,
            token.clone(),
            10,
        )
        .unwrap();

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
                EmailClient::new(Url::parse(&server.uri()).unwrap(), email(), token(), 10).unwrap();

            let response = email_client
                .send_email(email(), &sentence(), &paragraph(), &paragraph())
                .await;

            assert!(response.is_err());
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
        )
        .unwrap();

        let response = email_client
            .send_email(email(), &sentence(), &paragraph(), &paragraph())
            .await;

        assert!(response.is_err());
    }
}
