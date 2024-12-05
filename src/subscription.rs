use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
/// Trait describing how incoming messages on [Console] must be handled.
pub trait Subscription {
    /// Handles strongly-typed messages.
    ///
    /// Return optional [Bytes] that will be sent back to the message sender.
    async fn handle(&self, message: Bytes) -> Result<Option<Bytes>, SubscriptionError>;

    /// Handles free-form text messages.
    ///
    /// Returns an optional [String], which, if provided, will be sent back to the message sender.
    async fn weak_handle(&self, message: &str) -> Result<Option<String>, SubscriptionError>;
}

/// Convenience type to abstract away concrete implementations of [Subscription] errors.
pub type SubscriptionError = Box<dyn std::error::Error + Send + Sync>;

/// Convenience type to abstract away concrete implementations of [Subscription].
pub(crate) type BoxedSubscription = Box<dyn Subscription + Send + Sync>;
