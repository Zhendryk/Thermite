/*
    ABSTRACT: Definition of thread-safe event bus datastructure and its supporting
    datatypes to delegate events (see event.rs) between publishers
    (see publish.rs) and subscribers (see subscribe.rs)
*/
use crate::messaging::{
    sync::{event::Event, subscribe::Subscriber},
    types::*,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock, Weak};

/// Thread-safe datastructure responsible for dispatching events from `Publisher`s to `Subscriber`s
///
/// This keeps the respective Pub/Sub systems decoupled from each other
///
/// This should be wrapped in a Arc<RwLock<EventBus>>
pub struct EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    // We hold a std::sync::Weak (Arc which holds non-owning reference) to not prevent dropping and to avoid circular references to an Arc
    // We can deal with subscribers that get dropped by just removing them from our map if we find they did get dropped
    channels: HashMap<T, Vec<Weak<RwLock<dyn Subscriber<T, E>>>>>,
}

impl<T, E> Default for EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            channels: HashMap::default(),
        }
    }
}

impl<T, E> EventBus<T, E>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
    E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    /// Adds the given subscriber to a subscriber list to receive published messages of the given event variant
    pub fn subscribe<S: Subscriber<T, E> + 'static>(
        &mut self,
        subscriber: &Arc<RwLock<S>>,
        to_category: T,
    ) {
        if let Some(subscriber_list) = self.channels.get_mut(&to_category) {
            // We have an existing subscriber list for this category, push a new subscriber to it
            subscriber_list.push(Arc::downgrade(
                &(subscriber.clone() as Arc<RwLock<dyn Subscriber<T, E> + 'static>>),
            ));
            return;
        }
        // No subscriber list exists yet for this category, create one
        self.channels.insert(
            to_category,
            vec![Arc::downgrade(
                &(subscriber.clone() as Arc<RwLock<dyn Subscriber<T, E> + 'static>>),
            )],
        );
    }

    /// Unsubscribes the given subscriber from the given category on this `EventBus` (non-blocking)
    ///
    /// Automatically removes any dropped subscribers in the channel the given event belongs to, if the bus finds any
    ///
    /// **NOTE:** If a read-lock cannot be obtained, the subscriber will *NOT* be unsubscribed, as it cannot be identified without first obtaining a read-lock.
    pub fn unsubscribe<S: Subscriber<T, E> + 'static>(&mut self, subscriber: &S, from_category: T) {
        let mut cleanup_required = false;
        if let Some(subscriber_list) = self.channels.get_mut(&from_category) {
            if let Some(idx) = subscriber_list.iter().position(|weak_sub| {
                if let Some(subscriber_arc) = weak_sub.upgrade() {
                    match subscriber_arc.try_read() {
                        Ok(sub) => sub.id() == subscriber.id(),
                        Err(_) => false, // TODO: Look into more elegant handling, for now just skip
                    }
                } else {
                    // We dropped a subscriber, need to clean up
                    cleanup_required = true;
                    false
                }
            }) {
                // We can swap_remove for O(1) performance here because we don't care about ordering
                subscriber_list.swap_remove(idx);
            }

            if cleanup_required {
                subscriber_list.retain(|susbcriber| Weak::clone(susbcriber).upgrade().is_some());
            }
        }
    }

    /// Removes all subscribers from the given category `T` on this `EventBus`
    pub fn unsubscribe_all(&mut self, from_category: T) {
        self.channels.remove(&from_category);
    }

    /// Dispatches the given event to all subscribers of that event's category
    ///
    /// Automatically removes any dropped subscribers in the channel the given event belongs to, if the bus finds any
    pub fn dispatch_event(&mut self, event: &E) -> EventDispatchResult {
        let mut result = EventDispatchResult::NotNeeded;
        // Grab our list of subscribers for this event's category, if one exists
        if let Some(subscriber_list) = self.channels.get_mut(&event.category()) {
            let mut cleanup_required = false;
            // Attempt to have all subscribers handle the dispatched event and return requests to the event bus (non-blocking)
            result = execute_bus_requests(subscriber_list, |weak_subscriber| {
                if let Some(subscriber_arc) = weak_subscriber.upgrade() {
                    match subscriber_arc.try_read() {
                        Ok(subscriber) => subscriber.on_event(event),
                        Err(_) => BusRequest::DispatchFailed,
                    }
                } else {
                    // Found an invalid reference to a subscriber (which was probably dropped by the owner)
                    cleanup_required = true;
                    BusRequest::NoActionNeeded
                }
            });

            if cleanup_required {
                subscriber_list.retain(|susbcriber| Weak::clone(susbcriber).upgrade().is_some());
            }
        }

        result
    }
}

// TODO: Add OrdEventBus

// TODO: Add testing
