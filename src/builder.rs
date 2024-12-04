use crate::console::{Console, Error};
use crate::subscription::{BoxedSubscription, Subscription};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// A builder for [Console].
pub struct Builder<Services> {
    subscriptions: HashMap<Services, BoxedSubscription>,
    port: Option<u16>,
    welcome: Option<String>,
}

impl<Services> Builder<Services>
where
    Services: Eq + Hash + Debug,
{
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            port: None,
            welcome: None,
        }
    }

    pub fn subscribe<S>(mut self, service_id: Services, subscription: S) -> Result<Self, Error>
    where
        S: Subscription + Send + Sync + 'static,
    {
        // `HashMap::entry(x)` consumes its argument, while we might need this string afterwards.
        let service_id_string = format!("{service_id:?}");

        match self.subscriptions.entry(service_id) {
            Entry::Occupied(_) => Err(Error::ServiceIdUsed(service_id_string)),
            Entry::Vacant(entry) => {
                entry.insert(Box::new(subscription));
                Ok(self)
            }
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn welcome(mut self, message: &str) -> Self {
        self.welcome = Some(message.to_owned());
        self
    }

    pub fn build(self) -> Result<Console<Services>, Error> {
        let Some(port) = self.port else {
            return Err(Error::NoPort);
        };

        Ok(Console::new(
            self.subscriptions,
            port,
            format!("{}\n", self.welcome.unwrap_or_default()),
        ))
    }
}

impl<Services> Default for Builder<Services>
where
    Services: Eq + Hash + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}
