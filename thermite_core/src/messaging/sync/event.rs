/*
    ABSTRACT: Definition thread-safe generic events to be handled by their respective publishers, subscribers, and event buses.
*/
use std::hash::Hash;

/// A generic, thread-safe `Event`, categorized by an enum category `T`.
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
/// unsafe impl Send for ThermiteEventType {}
/// unsafe impl Sync for ThermiteEventType {}
///
/// // ThermiteEvent == E
/// #[derive(Debug, Eq, PartialEq, Hash, Clone)]
/// pub enum ThermiteEvent {
///     Keyboard(KeyboardEvent),
///     Mouse(MouseEvent),
/// }
/// unsafe impl Send for ThermiteEvent {}
/// unsafe impl Sync for ThermiteEvent {}
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
    T: Eq + PartialEq + Hash + Clone + Send + Sync + 'static,
{
    fn category(&self) -> T;
}
