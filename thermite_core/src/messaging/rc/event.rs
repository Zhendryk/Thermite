/*
    ABSTRACT: Definition single-thread generic events to be handled by their respective publishers, subscribers, and event buses.
*/
use std::hash::Hash;

/// A generic, single-thread `Event`, categorized by an enum category `T`.
///
/// - `T` is meant to be implemented by the module consumer as an enum, depicting the various categorie(s) an event can belong to.
///
/// ### Example
///
/// ```rust
/// // ThermiteEventType == T
/// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// pub enum ThermiteEventType {
///     Input,
///     Window,
/// }
///
/// // ThermiteEvent == E
/// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// pub enum ThermiteEvent {
///     Keyboard(KeyboardEvent),
///     Mouse(MouseEvent),
/// }
///
/// impl Event<ThermiteEventType> for ThermiteEvent {
///     fn category(&self) -> ThermiteEventType {
///         match self {
///             ThermiteEvent::Keyboard(_) => ThermiteEventType::Input,
///             ThermiteEvent::Mouse(_) => ThermiteEventType::Input,
///             // And more...
///         }
///     }
/// }
/// ```
pub trait Event<T>
where
    // ! NOTE: 'static on trait object means that T does not contain any references with a lifetime less than 'static
    // ! See: https://stackoverflow.com/questions/40053550/the-compiler-suggests-i-add-a-static-lifetime-because-the-parameter-type-may-no
    T: Eq + PartialEq + Hash + Clone + 'static,
{
    fn category(&self) -> T;
}
