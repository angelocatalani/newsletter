pub use errors::MalformedInput;
pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;

mod errors;
mod new_subscriber;
mod subscriber_email;
mod subscriber_name;
