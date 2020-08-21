use crate::messaging::event::ThermiteEvent;
use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, MouseScrollDelta};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ScrollDelta {
    x: i64,
    y: i64,
}

impl From<MouseScrollDelta> for ScrollDelta {
    fn from(msd: MouseScrollDelta) -> Self {
        match msd {
            MouseScrollDelta::LineDelta(x, y) => Self {
                x: x.round() as i64,
                y: y.round() as i64,
            },
            MouseScrollDelta::PixelDelta(logical_position) => Self {
                x: logical_position.x.round() as i64,
                y: logical_position.y.round() as i64,
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct PixelCoordinates {
    x: u64,
    y: u64,
}
impl From<PhysicalPosition<f64>> for PixelCoordinates {
    fn from(pp: PhysicalPosition<f64>) -> Self {
        Self {
            x: pp.x.round() as u64,
            y: pp.y.round() as u64,
        }
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
