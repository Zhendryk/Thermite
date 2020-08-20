use crate::input::{keyboard::KeyboardEvent, mouse::MouseEvent};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock, Weak};
// NOTE: Heavily inspired by Lakelezz's hey_listen: https://github.com/Lakelezz/hey_listen

// !Wrapper enum required for generic handling and pattern matching of all structures implementing Event
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ThermiteEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}
unsafe impl Send for ThermiteEvent {}
unsafe impl Sync for ThermiteEvent {}

/// A `Subscriber` is one who subscribes to and receives events of type `E`.
///
/// `E` is meant to be implemented by the module consumer as an enum with the appropriate traits.
pub trait Subscriber<E>
where
    E: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn on_event(&self, event: &E) -> BusRequest;
}

/// A `Publisher` is one who publishes events of type `E`, which are passed to `Subscriber`s via the `EventBus`.
///
/// `E` is meant to be implemented by the module consumer as an enum with the appropriate traits.
pub trait Publisher<E>
where
    E: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn publish_event(&self, event: &E, bus: &mut EventBus<E>) {
        bus.dispatch_event(event);
    }
}

/// The response given by a `Subscriber`'s `on_event` method, which can also act as a request to the `EventBus`.
#[derive(Debug)]
pub enum BusRequest {
    NoActionNeeded,
    Unsubscribe,
    DoNotPropagate,
    UnsubscribeAndDoNotPropagate,
}

/// The end result of the `EventBus`'s `dispatch_event` method, which results in one of the following:
///
///     1. `Stopped`: The event was handled by some subscribers in the list, but propagation was halted before the end of the list.
///     2. `Finished`: The event was handled by every subscriber in the list.
#[derive(Debug)]
pub enum EventDispatchResult {
    Stopped,
    Finished,
}

/// Given a list of subscribers from the `EventBus`, this method runs a closure on every subscriber in that list.
///
/// Each of those subscribers will return a resulting `BusRequest`, which we act on accordingly before returning a final `EventDispatchResult`.
pub(crate) fn execute_bus_requests<T, F>(
    subscribers: &mut Vec<T>,
    mut function: F,
) -> EventDispatchResult
where
    F: FnMut(&T) -> BusRequest,
{
    let mut idx = 0;
    loop {
        if idx < subscribers.len() {
            // Run our closure function on each subscriber
            match function(&subscribers[idx]) {
                // A return value of None lets us simply move onto the next subscriber
                BusRequest::NoActionNeeded => idx += 1,
                // The rest are self explanatory
                BusRequest::Unsubscribe => {
                    // swap_remove for O(1) operation
                    subscribers.swap_remove(idx);
                }
                BusRequest::DoNotPropagate => {
                    return EventDispatchResult::Stopped;
                }
                BusRequest::UnsubscribeAndDoNotPropagate => {
                    subscribers.swap_remove(idx);
                    return EventDispatchResult::Stopped;
                }
            }
        } else {
            // We've made it to the end of our subscriber list without stopping propagation
            return EventDispatchResult::Finished;
        }
    }
}

/// Datastructure responsible for dispatching events from `Publisher`s to `Subscriber`s
pub struct EventBus<E>
where
    E: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    sink: HashMap<E, Vec<Weak<RwLock<dyn Subscriber<E>>>>>,
}

impl<E> Default for EventBus<E>
where
    E: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            sink: HashMap::default(),
        }
    }
}

impl<E> EventBus<E>
where
    E: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    /// Adds the given subscriber to a subscriber list to receive published messages of the given event variant
    pub fn subscribe<S: Subscriber<E> + 'static>(
        &mut self,
        subscriber: &Arc<RwLock<S>>,
        to_variant: E,
    ) {
        if let Some(subscriber_list) = self.sink.get_mut(&to_variant) {
            subscriber_list.push(Arc::downgrade(
                &(subscriber.clone() as Arc<RwLock<dyn Subscriber<E>>>),
            ));
            return;
        }
        self.sink.insert(
            to_variant,
            vec![Arc::downgrade(
                &(subscriber.clone() as Arc<RwLock<dyn Subscriber<E>>>),
            )],
        );
    }

    // TODO: unsubscribe

    // TODO: unsubscribe_all

    /// Dispatches the given event to all subscribers to that event's variant
    pub fn dispatch_event(&mut self, event: &E) {
        if let Some(subscriber_list) = self.sink.get_mut(event) {
            execute_bus_requests(subscriber_list, |weak_subscriber| {
                if let Some(subscriber_arc) = weak_subscriber.upgrade() {
                    let subscriber = subscriber_arc
                        .write() // TODO: Maybe try_write() instead for non-blocking?
                        .expect("Couldn't write to subscriber");
                    subscriber.on_event(event)
                } else {
                    BusRequest::NoActionNeeded
                }
            });
        }
    }
}
