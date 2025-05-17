use crate::damain::subscriber_name::SubscriberName;
use crate::damain::SubscriberEmail;

#[derive(Debug)]
pub struct NewSubscriber {
    pub name: SubscriberName,
    pub email: SubscriberEmail,
}
