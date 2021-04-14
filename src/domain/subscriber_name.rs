use std::convert::TryFrom;

use unicode_segmentation::UnicodeSegmentation;

use crate::domain::errors::MalformedInput;

const FORBIDDEN_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
const MAX_LENGTH: usize = 256;

#[derive(Clone, Debug)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = MalformedInput;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        let is_empty_or_whitespace = name.trim().is_empty();
        let is_too_long = name.graphemes(true).count() > MAX_LENGTH;
        let contains_forbidden_characters = name.chars().any(|g| FORBIDDEN_CHARS.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(MalformedInput::InvalidName { name })
        } else {
            Ok(Self(name))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use claim::{
        assert_err,
        assert_ok,
    };
    use fake::faker::name::en::{
        FirstName,
        LastName,
        Name,
    };
    use fake::Fake;
    use quickcheck::Gen;

    use super::SubscriberName;
    use super::FORBIDDEN_CHARS;
    use super::MAX_LENGTH;

    #[test]
    fn empty_name_is_invalid() {
        assert_err!(SubscriberName::try_from("".to_string()));
    }
    #[test]
    fn whitespace_name_is_invalid() {
        assert_err!(SubscriberName::try_from(" ".repeat(MAX_LENGTH)));
        assert_err!(SubscriberName::try_from(" ".to_string()));
    }
    #[test]
    fn too_long_name_is_invalid() {
        assert_err!(SubscriberName::try_from("a".repeat(MAX_LENGTH + 1)));
    }

    #[derive(Clone, Debug)]
    struct ValidNameFixture {
        pub first_name: String,
        pub second_name: String,
        pub full_name: String,
    }
    impl quickcheck::Arbitrary for ValidNameFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            ValidNameFixture {
                first_name: FirstName().fake_with_rng(g),
                second_name: LastName().fake_with_rng(g),
                full_name: Name().fake_with_rng(g),
            }
        }
    }

    #[quickcheck_macros::quickcheck]
    fn name_with_forbidden_chars_is_invalid(valid_name: ValidNameFixture) {
        FORBIDDEN_CHARS.iter().for_each(|c| {
            let invalid_first_and_second_name =
                format!("{} {} {}", valid_name.first_name, c, valid_name.second_name);
            assert_err!(SubscriberName::try_from(invalid_first_and_second_name));

            let invalid_full_name = format!("{} {}", c, valid_name.full_name);
            assert_err!(SubscriberName::try_from(invalid_full_name));
        })
    }

    #[quickcheck_macros::quickcheck]
    fn valid_name_is_parsed_successfully(valid_name: ValidNameFixture) {
        assert_ok!(SubscriberName::try_from(valid_name.first_name));
        assert_ok!(SubscriberName::try_from(valid_name.second_name));
        assert_ok!(SubscriberName::try_from(valid_name.full_name));
    }
}
