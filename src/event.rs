use log::info;
use psbus::{
    rc::{Event, EventBus, Publisher, Subscriber},
    types::BusRequest,
};
use std::hash::Hash;
use thermite_core::input::{
    keyboard::{KeyCode, KeyboardModifiers},
    mouse::{PixelCoordinates, ScrollDelta},
};
use thermite_gfx::winit::event::MouseButton;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum KeyboardEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    ModifiersChanged(KeyboardModifiers),
}

impl From<KeyboardEvent> for ThermiteEvent {
    fn from(kb_evt: KeyboardEvent) -> Self {
        ThermiteEvent::Keyboard(kb_evt)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum MouseEvent {
    ButtonPressed(MouseButton),
    ButtonReleased(MouseButton),
    Scroll(ScrollDelta),
    Motion(PixelCoordinates),
    EnteredWindow,
    LeftWindow,
}

impl From<MouseEvent> for ThermiteEvent {
    fn from(m_evt: MouseEvent) -> Self {
        ThermiteEvent::Mouse(m_evt)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ThermiteEventType {
    Input,
    Window,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ThermiteEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

impl Event<ThermiteEventType> for ThermiteEvent {
    fn category(&self) -> ThermiteEventType {
        match self {
            ThermiteEvent::Keyboard(_) => ThermiteEventType::Input,
            ThermiteEvent::Mouse(_) => ThermiteEventType::Input,
            // And more...
        }
    }
}

pub struct TestSubscriber {
    pub id: Uuid,
}
impl Subscriber<ThermiteEventType, ThermiteEvent> for TestSubscriber {
    // ! Although we get a ThermiteEvent enum, it is guaranteed to be only of the category that we are subscribed to
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn on_event(&self, event: &ThermiteEvent) -> BusRequest {
        info!("Subscriber {} received event: {:?}", self.id, event);
        BusRequest::NoActionNeeded
    }
}

pub struct TestPublisher {}
impl Publisher<ThermiteEventType, ThermiteEvent> for TestPublisher {}

pub type ThermiteEventBus = EventBus<ThermiteEventType, ThermiteEvent>;
