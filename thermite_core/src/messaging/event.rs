/*
    ABSTRACT: Definitions of single-thread and thread-safe generic events
    to be handled by their respective publishers, subscribers, and event buses.
*/
use crate::input::{keyboard::KeyboardEvent, mouse::MouseEvent};
use std::hash::Hash;

/// A generic, single-thread `Event`, categorized by an enum category `T`, meant to be implemented as an enum by the module consumer.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categorie(s) an event can belong to.
pub trait Event<T>
where
    T: Eq + PartialEq + Hash + Clone,
{
    fn category(&self) -> T;
}

/// A generic, thread-safe `TSEvent`, categorized by an enum category `T`, meant to be implemented as an enum by the module consumer.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categorie(s) an event can belong to.
pub trait TSEvent<T>
where
    T: Eq + PartialEq + Hash + Clone + Send + Sync,
{
    fn category(&self) -> T;
}

// ! In order to give a category to our events
// ! This would normally be provided by the consumer crate
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ThermiteEventType {
    Input,
    Window,
}
// unsafe impl Send for ThermiteEventType {}
// unsafe impl Sync for ThermiteEventType {}
// ! Wrapper enum required for generic handling and pattern matching of all structures implementing Event
// ! This would normally be provided by the consumer crate
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ThermiteEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}
// unsafe impl Send for ThermiteEvent {}
// unsafe impl Sync for ThermiteEvent {}
impl Event<ThermiteEventType> for ThermiteEvent {
    fn category(&self) -> ThermiteEventType {
        match self {
            ThermiteEvent::Keyboard(_) => ThermiteEventType::Input,
            ThermiteEvent::Mouse(_) => ThermiteEventType::Input,
            // And more...
        }
    }
}
