use validator::ValidateEmail;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<Self, String> {
        if email.validate_email() {
            Ok(Self(email))
        } else {
            Err(format!("Invalid email: {}", email))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use proptest::prelude::*;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!( SubscriberEmail::parse(email));
    }

    #[test]
    fn email_symbol_mussing_rejected() {
        let email = "qwer.com".to_string();
        assert_err!( SubscriberEmail::parse(email));
    }

    #[test]
    fn subject_missing_rejected() {
        let email = "@gmail.com".to_string();
        assert_err!( SubscriberEmail::parse(email));
    }

    fn safe_email_strategy() -> impl Strategy<Value = String> {
        any::<u8>().prop_map(|_| SafeEmail().fake::<String>())
    }

    proptest! {
        #[test]
        fn valid_email_accepted(email in safe_email_strategy()) {
            assert_ok!( SubscriberEmail::parse(email));
        }
    }
}