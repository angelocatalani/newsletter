use crate::domain::errors::MalformedInput;
use std::convert::TryFrom;

pub struct SubscriberEmail(String);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = MalformedInput;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(SubscriberEmail(value))
    }
}
