mod client;
pub use client::Client;

mod console;
pub use console::{Console, Error};

mod builder;
pub use builder::Builder;

mod subscription;
pub use subscription::{Subscription, SubscriptionError};

fn ensure_newline(mut input: String) -> String {
    if !input.ends_with('\n') {
        input.push('\n');
    }
    input
}
