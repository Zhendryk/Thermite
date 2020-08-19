use crate::platform::event::{Event, EventCategory};
use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, MouseScrollDelta};

#[derive(Debug)]
pub enum MouseEvent {
    ButtonPressed(MouseButton),
    ButtonReleased(MouseButton),
    Scroll(MouseScrollDelta),
    Motion(PhysicalPosition<f64>),
    EnteredWindow,
    LeftWindow,
}

impl Event for MouseEvent {
    fn category(&self) -> EventCategory {
        EventCategory::Mouse
    }

    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}
