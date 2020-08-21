/*
    ABSTRACT: Definitions of single-thread and thread-safe event bus datastructures
    and their supporting datatypes to delegate events (see event.rs) between publishers
    (see publish.rs) and subscribers (see subscribe.rs)
*/
use crate::messaging::{
    event::{Event, TSEvent},
    subscribe::{Subscriber, TSSubscriber},
};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::{Rc, Weak};
use std::sync::{Arc, RwLock, Weak as TSWeak};

/// The response given by a `Subscriber`'s `on_event` method, which can also act as a request to the `EventBus`.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum BusRequest {
    NoActionNeeded,
    Unsubscribe,
    DoNotPropagate,
    UnsubscribeAndDoNotPropagate,
}
unsafe impl Send for BusRequest {}
unsafe impl Sync for BusRequest {}

/// The end result of the `EventBus`'s `dispatch_event` method, which results in one of the following:
///
///     1. `Stopped`: The event was handled by some subscribers in the list, but propagation was halted before the end of the list.
///     2. `Finished`: The event was handled by every subscriber in the list.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum EventDispatchResult {
    Stopped,
    Finished,
}
unsafe impl Send for EventDispatchResult {}
unsafe impl Sync for EventDispatchResult {}

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
                    subscribers.swap_remove(idx); // TODO: If we move to layers, we can't arbitrarily alter the order here like this...
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

//===================================================== NON THREAD SAFE =====================================================//

/// Single-thread datastructure responsible for dispatching events from `Publisher`s to `Subscriber`s
///
/// This keeps the respective Pub/Sub systems decoupled from each other
///
/// This should be wrapped in a Rc<RefCell<EventBus>>
pub struct EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone,
    E: Event<T> + Eq + PartialEq + Hash + Clone,
{
    // We hold a std::rc::Weak (Rc which holds non-owning reference) to not prevent dropping and to avoid circular references to an Rc
    // We can deal with subscribers that get dropped by just removing them from our map if we find they did get dropped
    channels: HashMap<T, Vec<Weak<dyn Subscriber<T, E>>>>,
}

impl<T, E> Default for EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone,
    E: Event<T> + Eq + PartialEq + Hash + Clone,
{
    fn default() -> Self {
        Self {
            channels: HashMap::default(),
        }
    }
}

impl<T, E> EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone,
    E: Event<T> + Eq + PartialEq + Hash + Clone,
{
    /// Adds the given subscriber to a subscriber list to receive published messages of the given event variant
    pub fn subscribe<S: Subscriber<T, E> + 'static>(&mut self, subscriber: &Rc<S>, to_category: T) {
        if let Some(subscriber_list) = self.channels.get_mut(&to_category) {
            // We have an existing subscriber list for this category
            subscriber_list.push(Rc::downgrade(
                &(subscriber.clone() as Rc<dyn Subscriber<T, E>>),
            ));
            return;
        }
        // No subscriber list exists yet for this category, insert one
        self.channels.insert(
            to_category,
            vec![Rc::downgrade(
                &(subscriber.clone() as Rc<dyn Subscriber<T, E>>),
            )],
        );
    }

    pub fn unsubscribe<S: Subscriber<T, E>>(&mut self, subscriber: &S, from_category: T) {
        unimplemented!()
    }

    /// Removes all subscribers from the given category on this `EventBus`
    pub fn unsubscribe_all(&mut self, from_category: T) {
        self.channels.remove(&from_category);
    }

    /// Dispatches the given event to all subscribers of that event's category
    pub fn dispatch_event(&mut self, event: &E) {
        // Grab our list of subscribers for this event's category, if one exists
        if let Some(subscriber_list) = self.channels.get_mut(&event.category()) {
            // For every subscriber in that list, handle the event after which that subscriber will
            // tell the bus whether or not it should propagate the event to other subscribers, among other actions
            // TODO: In order for this to make sense, our subscribers need to be ordered in a fashion that makes sense for event propagation (layers)
            execute_bus_requests(subscriber_list, |weak_subscriber| {
                // Upgrade our weak rc pointer to a full Arc, obtain a write lock and handle the event
                if let Some(subscriber) = weak_subscriber.upgrade() {
                    subscriber.on_event(event)
                } else {
                    // No subscriber to act on, so do nothing for this iteration
                    BusRequest::NoActionNeeded
                    // TODO: Clean up dropped subscriber
                }
            });
        }
    }
}
//===================================================== END NON THREAD SAFE =====================================================//

//===================================================== THREAD SAFE =====================================================//

/// Thread-safe datastructure responsible for dispatching events from `TSPublisher`s to `TSSubscriber`s
///
/// This keeps the respective Pub/Sub systems decoupled from each other
///
/// This should be wrapped in a Arc<RwLock<TSEventBus>>
pub struct TSEventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync,
    E: TSEvent<T> + Eq + PartialEq + Hash + Clone + Send + Sync,
{
    // We hold a std::sync::Weak (Arc which holds non-owning reference) to not prevent dropping and to avoid circular references to an Arc
    // We can deal with subscribers that get dropped by just removing them from our map if we find they did get dropped
    channels: HashMap<T, Vec<TSWeak<RwLock<dyn TSSubscriber<T, E>>>>>,
}

impl<T, E> Default for TSEventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync,
    E: TSEvent<T> + Eq + PartialEq + Hash + Clone + Send + Sync,
{
    fn default() -> Self {
        Self {
            channels: HashMap::default(),
        }
    }
}

impl<T, E> TSEventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync,
    E: TSEvent<T> + Eq + PartialEq + Hash + Clone + Send + Sync,
{
    /// Adds the given subscriber to a subscriber list to receive published messages of the given event variant
    pub fn subscribe<S: TSSubscriber<T, E> + 'static>(
        &mut self,
        subscriber: &Arc<RwLock<S>>,
        to_category: T,
    ) {
        if let Some(subscriber_list) = self.channels.get_mut(&to_category) {
            // We have an existing subscriber list for this category
            subscriber_list.push(Arc::downgrade(
                &(subscriber.clone() as Arc<RwLock<dyn TSSubscriber<T, E>>>),
            ));
            return;
        }
        // No subscriber list exists yet for this category, insert one
        self.channels.insert(
            to_category,
            vec![Arc::downgrade(
                &(subscriber.clone() as Arc<RwLock<dyn TSSubscriber<T, E>>>),
            )],
        );
    }

    pub fn unsubscribe<S: TSSubscriber<T, E>>(&mut self, subscriber: &S, from_category: T) {
        unimplemented!()
    }

    /// Removes all subscribers from the given category on this `TSEventBus`
    pub fn unsubscribe_all(&mut self, from_category: T) {
        self.channels.remove(&from_category);
    }

    /// Dispatches the given event to all subscribers of that event's category
    pub fn dispatch_event(&mut self, event: &E) {
        // Grab our list of subscribers for this event's category, if one exists
        if let Some(subscriber_list) = self.channels.get_mut(&event.category()) {
            // For every subscriber in that list, handle the event after which that subscriber will
            // tell the bus whether or not it should propagate the event to other subscribers, among other actions
            // TODO: In order for this to make sense, our subscribers need to be ordered in a fashion that makes sense for event propagation (layers)
            execute_bus_requests(subscriber_list, |weak_subscriber| {
                // Upgrade our weak rc pointer to a full Arc, obtain a write lock and handle the event
                if let Some(subscriber_arc) = weak_subscriber.upgrade() {
                    let subscriber = subscriber_arc
                        .write() // TODO: Maybe try_write() instead for non-thread-blocking behavior?
                        .expect("Couldn't write to subscriber");
                    subscriber.on_event(event)
                } else {
                    // No subscriber to act on, so do nothing for this iteration
                    BusRequest::NoActionNeeded
                    // TODO: Clean up dropped subscriber
                }
            });
        }
    }
}

//===================================================== END THREAD SAFE =====================================================//
