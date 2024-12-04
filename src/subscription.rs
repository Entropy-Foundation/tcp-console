use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
/// Trait describing how incoming messages on [Console] must be handled.
pub trait Subscription {
    async fn handle(&self, message: Bytes) -> Result<Option<String>, SubscriptionError>;
    async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError>;
}

/// Convenience type to abstract away concrete implementations of [Subscription] errors.
pub type SubscriptionError = Box<dyn std::error::Error + Send + Sync>;

/// Convenience type to abstract away concrete implementations of [Subscription].
pub(crate) type BoxedSubscription = Box<dyn Subscription + Send + Sync>;
