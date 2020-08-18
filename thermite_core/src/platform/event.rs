use std::boxed::Box;

// TODO: Thread-safety
// TODO: Compiler-optional logging
// TODO: Compiler-optional profiling
// TODO: Non-blocking event handling (queues)

pub trait Subscriber {
    fn identifier(&self) -> u32;
    fn on_event(&self, event: &dyn Event);
}

pub trait Publisher {
    fn publish_event(&self, event: &dyn Event, bus: &EventBus) {
        bus.dispatch_event(event);
    }
}

pub trait Event {
    fn category(&self) -> &EventCategory;
    fn to_str(&self) -> &str;
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum EventCategory {
    Test,
    Window,
    Mouse,
    Keyboard,
}

/// `EventBus` driving event handling between a set of `Publisher`s and `Subscriber`s
pub struct EventBus {
    sinks: std::collections::HashMap<EventCategory, Vec<Box<dyn Subscriber>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        EventBus {
            sinks: std::collections::HashMap::default(),
        }
    }
}

impl EventBus {
    /// Subscribes the given `Subscriber` to receive events of the given `EventCategory` from the `EventBus`
    pub fn subscribe(&mut self, subscriber: Box<dyn Subscriber>, to_category: EventCategory) {
        let incoming_id = subscriber.identifier();
        match self.sinks.get_mut(&to_category) {
            Some(subscriber_list) => {
                if !subscriber_list
                    .iter()
                    .any(|s| s.identifier() == incoming_id)
                {
                    println!(
                        "Subscribed subscriber<id={}> to {:?} events!",
                        incoming_id, to_category
                    );
                    subscriber_list.push(subscriber);
                } else {
                    println!(
                        "Subscriber<id={}> is already subscribed to {:?} events!",
                        incoming_id, to_category
                    );
                }
            }
            None => {
                println!(
                    "Subscribed subscriber<id={}> to {:?} events!",
                    incoming_id, to_category
                );
                self.sinks.insert(to_category, vec![subscriber]);
            }
        }
    }

    /// Unsubscribes the given `Subscriber` from receiving events of the given `EventCategory` from the `EventBus`
    pub fn unsubscribe(&mut self, subscriber: &impl Subscriber, from_category: EventCategory) {
        match self.sinks.get_mut(&from_category) {
            Some(subscriber_list) => {
                println!(
                    "Subscriber<id={}> unsubscribed from {:?} events!",
                    subscriber.identifier(),
                    from_category
                );
                subscriber_list.retain(|s| s.identifier() != subscriber.identifier());
            }
            _ => (),
        }
    }

    /// Dispatches the given `Event` to all tasks subscribed to that `Event`'s `EventCategory`
    pub(crate) fn dispatch_event(&self, event: &dyn Event) {
        match self.sinks.get(event.category()) {
            Some(subscriber_list) => {
                for subscriber in subscriber_list.iter() {
                    println!(
                        "Event<category={:?}> dispatched to subscriber<id={}>",
                        event.category(),
                        subscriber.identifier()
                    );
                    subscriber.on_event(event);
                }
            }
            _ => (),
        }
    }
}

// EXAMPLE SUBSCRIBER
// pub struct TestSubscriber {}
// impl Subscriber for TestSubscriber {
//     fn identifier(&self) -> u32 {
//         1
//     }
//     fn on_event(&self, event: &dyn Event) {
//         println!(
//             "Subscriber({}) received event: {}",
//             self.identifier(),
//             event.to_str()
//         );
//     }
// }

// EXAMPLE PUBLISHER
// pub struct TestPublisher {}
// impl Publisher for TestPublisher {}

// EXAMPLE EVENT
// pub struct TestEvent {}
// impl Event for TestEvent {
//     fn category(&self) -> &EventCategory {
//         &EventCategory::Test
//     }
//     fn to_str(&self) -> &str {
//         "TestEvent<Category=Test>"
//     }
// }

// EXAMPLE USAGE
// fn main() {
//     let mut bus = EventBus::default();
//     let subscriber = TestSubscriber {};
//     bus.subscribe(Box::new(subscriber), EventCategory::Test);
//     let publisher = TestPublisher {};
//     publisher.publish_event(&TestEvent {}, &bus);
// }
