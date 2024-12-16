use crate::console::{Console, Error};
use crate::ensure_newline;
use crate::subscription::{BoxedSubscription, Subscription};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use tokio::net::ToSocketAddrs;

/// A builder for [Console].
pub struct Builder<Services, A> {
    subscriptions: HashMap<Services, BoxedSubscription>,
    bind_address: Option<A>,
    welcome: Option<String>,
    accept_only_localhost: bool,
}

impl<Services, A> Builder<Services, A>
where
    Services: Eq + Hash + Debug,
    A: ToSocketAddrs,
{
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            bind_address: None,
            welcome: None,
            accept_only_localhost: false,
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

    pub fn bind_address(mut self, bind_address: A) -> Self {
        self.bind_address = Some(bind_address);
        self
    }

    pub fn welcome(mut self, message: &str) -> Self {
        self.welcome = Some(message.to_owned());
        self
    }

    pub fn accept_only_localhost(mut self) -> Self {
        self.accept_only_localhost = true;
        self
    }

    pub fn build(self) -> Result<Console<Services, A>, Error> {
        let Some(bind_address) = self.bind_address else {
            return Err(Error::NoBindAddress);
        };

        Ok(Console::new(
            self.subscriptions,
            bind_address,
            ensure_newline(self.welcome.unwrap_or_default()),
            self.accept_only_localhost,
        ))
    }
}

impl<Services, A> Default for Builder<Services, A>
where
    Services: Eq + Hash + Debug,
    A: ToSocketAddrs,
{
    fn default() -> Self {
        Self::new()
    }
}
