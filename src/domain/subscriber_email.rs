use std::convert::TryFrom;

use validator::validate_email;

#[derive(Clone, Debug)]
pub struct SubscriberEmail(String);

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(email: String) -> Result<Self, Self::Error> {
        if validate_email(email.clone()) {
            Ok(SubscriberEmail(email))
        } else {
            Err(format!("Invalid email: {}", email))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use claim::assert_ok;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Gen;

    use super::SubscriberEmail;

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Self(SafeEmail().fake_with_rng(g))
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_name_is_parsed_successfully(valid_email: ValidEmailFixture) {
        assert_ok!(SubscriberEmail::try_from(valid_email.0));
    }
}
