// // !NOTE: Heavily inspired by Lakelezz's hey_listen: https://github.com/Lakelezz/hey_listen
// use crate::input::{keyboard::KeyboardEvent, mouse::MouseEvent};
// use std::collections::HashMap;
// use std::hash::Hash;
// use std::sync::{Arc, RwLock, Weak};

// /// A generic `Event`, categorized by an enum category `T`, meant to be implemented as an enum by the module consumer.
// ///
// /// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categorie(s) an event can belong to.
// pub trait Event<T>
// where
//     T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
// {
//     fn category(&self) -> T;
// }

// /// A `Subscriber` is a construct which subscribes to a `Publisher` to receive events of type `E`.
// ///
// /// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
// pub trait Subscriber<E>
// where
//     E: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
// {
//     fn on_event(&self, event: &E) -> BusRequest;
// }

// /// A `Publisher` is a construct which publishes events `E` of category `T` to a self-maintained list of `Subscribers` `S`.
// ///
// /// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categories an event can belong to.
// ///
// /// - `E` is meant to be implemented by the module consumer as an enum, depicting the individual events which exist in the system. See `Event`.
// pub trait Publisher<T, E>
// where
//     T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
//     E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
// {
//     fn publish_event(&self, event: &E, bus: &mut EventBus<T, E>) {
//         bus.dispatch_event(event);
//     }
// }

// /// The response given by a `Subscriber`'s `on_event` method, which can also act as a request to the `EventBus`.
// #[derive(Debug)]
// pub enum BusRequest {
//     NoActionNeeded,
//     Unsubscribe,
//     DoNotPropagate,
//     UnsubscribeAndDoNotPropagate,
// }

// /// The end result of the `EventBus`'s `dispatch_event` method, which results in one of the following:
// ///
// ///     1. `Stopped`: The event was handled by some subscribers in the list, but propagation was halted before the end of the list.
// ///     2. `Finished`: The event was handled by every subscriber in the list.
// #[derive(Debug)]
// pub enum EventDispatchResult {
//     Stopped,
//     Finished,
// }

// /// Given a list of subscribers from the `EventBus`, this method runs a closure on every subscriber in that list.
// ///
// /// Each of those subscribers will return a resulting `BusRequest`, which we act on accordingly before returning a final `EventDispatchResult`.
// pub(crate) fn execute_bus_requests<T, F>(
//     subscribers: &mut Vec<T>,
//     mut function: F,
// ) -> EventDispatchResult
// where
//     F: FnMut(&T) -> BusRequest,
// {
//     let mut idx = 0;
//     loop {
//         if idx < subscribers.len() {
//             // Run our closure function on each subscriber
//             match function(&subscribers[idx]) {
//                 // A return value of None lets us simply move onto the next subscriber
//                 BusRequest::NoActionNeeded => idx += 1,
//                 // The rest are self explanatory
//                 BusRequest::Unsubscribe => {
//                     // swap_remove for O(1) operation
//                     subscribers.swap_remove(idx); // TODO: If we move to layers, we can't arbitrarily alter the order here like this...
//                 }
//                 BusRequest::DoNotPropagate => {
//                     return EventDispatchResult::Stopped;
//                 }
//                 BusRequest::UnsubscribeAndDoNotPropagate => {
//                     subscribers.swap_remove(idx);
//                     return EventDispatchResult::Stopped;
//                 }
//             }
//         } else {
//             // We've made it to the end of our subscriber list without stopping propagation
//             return EventDispatchResult::Finished;
//         }
//     }
// }

// /// Datastructure responsible for dispatching events from `Publisher`s to `Subscriber`s
// ///
// /// This keeps the respective Pub/Sub systems decoupled from each other
// pub struct EventBus<T, E>
// where
//     T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
//     E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
// {
//     // We hold a std::sync::Weak (Arc which holds non-owning reference) to not prevent dropping and to avoid circular references to an Arc
//     // We can deal with subscribers that get dropped by just removing them from our map if we find they did get dropped
//     channels: HashMap<T, Vec<Weak<RwLock<dyn Subscriber<E>>>>>,
// }

// impl<T, E> Default for EventBus<T, E>
// where
//     T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
//     E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
// {
//     fn default() -> Self {
//         Self {
//             channels: HashMap::default(),
//         }
//     }
// }

// impl<T, E> EventBus<T, E>
// where
//     T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
//     E: Event<T> + Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
// {
//     /// Adds the given subscriber to a subscriber list to receive published messages of the given event variant
//     pub fn subscribe<S: Subscriber<E> + 'static>(
//         &mut self,
//         subscriber: &Arc<RwLock<S>>,
//         to_category: T,
//     ) {
//         if let Some(subscriber_list) = self.channels.get_mut(&to_category) {
//             // We have an existing subscriber list for this category
//             subscriber_list.push(Arc::downgrade(
//                 &(subscriber.clone() as Arc<RwLock<dyn Subscriber<E>>>),
//             ));
//             return;
//         }
//         // No subscriber list exists yet for this category, insert one
//         self.channels.insert(
//             to_category,
//             vec![Arc::downgrade(
//                 &(subscriber.clone() as Arc<RwLock<dyn Subscriber<E>>>),
//             )],
//         );
//     }

//     // TODO: unsubscribe

//     // TODO: unsubscribe_all

//     /// Dispatches the given event to all subscribers of that event's category
//     pub fn dispatch_event(&mut self, event: &E) {
//         // Grab our list of subscribers for this event's category, if one exists
//         if let Some(subscriber_list) = self.channels.get_mut(&event.category()) {
//             // For every subscriber in that list, handle the event after which that subscriber will
//             // tell the bus whether or not it should propagate the event to other subscribers, among other actions
//             // TODO: In order for this to make sense, our subscribers need to be ordered in a fashion that makes sense for event propagation (layers)
//             execute_bus_requests(subscriber_list, |weak_subscriber| {
//                 // Upgrade our weak rc pointer to a full Arc, obtain a write lock and handle the event
//                 if let Some(subscriber_arc) = weak_subscriber.upgrade() {
//                     let subscriber = subscriber_arc
//                         .write() // TODO: Maybe try_write() instead for non-thread-blocking behavior?
//                         .expect("Couldn't write to subscriber");
//                     subscriber.on_event(event)
//                 } else {
//                     // No subscriber to act on, so do nothing for this iteration
//                     BusRequest::NoActionNeeded
//                 }
//             });
//         }
//     }
// }

// // ================================================ Thermite-specific Datastructures ================================================ //
// // ! In order to give a category to our events
// // ! This would normally be provided by the consumer crate
// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
// pub enum ThermiteEventType {
//     Input,
//     Window,
// }
// unsafe impl Send for ThermiteEventType {}
// unsafe impl Sync for ThermiteEventType {}

// // ! Wrapper enum required for generic handling and pattern matching of all structures implementing Event
// // ! This would normally be provided by the consumer crate
// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
// pub enum ThermiteEvent {
//     Keyboard(KeyboardEvent),
//     Mouse(MouseEvent),
// }
// unsafe impl Send for ThermiteEvent {}
// unsafe impl Sync for ThermiteEvent {}
// impl Event<ThermiteEventType> for ThermiteEvent {
//     fn category(&self) -> ThermiteEventType {
//         match self {
//             ThermiteEvent::Keyboard(_) => ThermiteEventType::Input,
//             ThermiteEvent::Mouse(_) => ThermiteEventType::Input,
//             // And more...
//         }
//     }
// }
// // ================================================ END Thermite-specific Datastructures ================================================ //
