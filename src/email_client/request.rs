use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct EmailRequest<'a> {
    pub messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Message<'a> {
    pub from: From<'a>,
    pub to: Vec<To<'a>>,
    pub subject: &'a str,
    #[serde(rename = "TextPart")]
    pub text_part: &'a str,
    #[serde(rename = "HTMLPart")]
    pub html_part: &'a str,
    #[serde(rename = "CustomID")]
    pub custom_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct From<'a> {
    pub email: &'a str,
    pub name: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct To<'a> {
    pub email: &'a str,
    pub name: &'a str,
}

impl<'a> EmailRequest<'a> {
    const MAIL_NAME: &'a str = "Newsletter";

    pub fn new(
        sender: &'a str,
        recipient: &'a str,
        subject: &'a str,
        html_part: &'a str,
        text_part: &'a str,
    ) -> Self {
        Self {
            messages: vec![Message {
                from: From {
                    email: sender,
                    name: Self::MAIL_NAME,
                },
                to: vec![To {
                    email: recipient,
                    name: Self::MAIL_NAME,
                }],
                subject,
                text_part,
                html_part,
                custom_id: Self::MAIL_NAME,
            }],
        }
    }
}
