/*
    ABSTRACT: Definitions of single-thread and thread-safe generic publishers which
    utilize an intermediary event bus (see bus.rs) to send generic messages to their respective subscribers (see subscribe.rs)
*/
use crate::messaging::{
    bus::{EventBus, TSEventBus},
    event::{Event, TSEvent},
};
use std::hash::Hash;

/// A generic, single-thread `Publisher`, publishes events `E` of category `T` to a self-maintained list of `Subscribers` `S`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait Publisher<T, E>
where
    T: Eq + PartialEq + Hash + Clone,
    E: Event<T> + Eq + PartialEq + Hash + Clone,
{
    fn publish_event(&self, event: &E, bus: &mut EventBus<T, E>) {
        bus.dispatch_event(event);
    }
}

/// A generic, thread-safe `TSPublisher`, publishes events `E` of category `T` to a self-maintained list of `TSSubscribers` `S`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait TSPublisher<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync,
    E: TSEvent<T> + Eq + PartialEq + Hash + Clone + Send + Sync,
{
    fn publish_event(&self, event: &E, bus: &mut TSEventBus<T, E>) {
        bus.dispatch_event(event);
    }
}
