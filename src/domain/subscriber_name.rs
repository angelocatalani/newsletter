use crate::domain::errors::MalformedInput;
use std::convert::TryFrom;

pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = MalformedInput;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(SubscriberName(value))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
