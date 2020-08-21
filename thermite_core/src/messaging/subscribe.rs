/*
    ABSTRACT: Definitions of single-thread and thread-safe generic subscribers which
    can subscribe to an intermediary event bus (see bus.rs) which dispatches relevant generic events
    that are published to them by one or more publishers (see publish.rs)
*/
use crate::messaging::{
    bus::BusRequest,
    event::{Event, TSEvent},
};
use std::hash::Hash;

/// A generic, single-thread `Subscriber`, subscribes to a `Publisher` to receive events of type `E`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait Subscriber<T, E>
where
    T: Eq + PartialEq + Hash + Clone,
    E: Event<T> + Eq + PartialEq + Hash + Clone,
{
    // TODO: Should subscribers have a UUID? For identification/unsubscription purposes.

    fn on_event(&self, event: &E) -> BusRequest;
}

/// A generic, thread-safe `TSSubscriber`, subscribes to a `TSPublisher` to receive events of type `E`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
///
/// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
pub trait TSSubscriber<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync,
    E: TSEvent<T> + Eq + PartialEq + Hash + Clone + Send + Sync,
{
    // TODO: Should subscribers have a UUID? For identification/unsubscription purposes.

    fn on_event(&self, event: &E) -> BusRequest;
}
