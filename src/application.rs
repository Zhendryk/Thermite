use crate::event::*;
use std::cell::RefCell;
use std::rc::Rc;
use thermite_core::messaging::rc::publish::Publisher;
use thermite_core::{messaging::rc::bus::EventBus, thermite_logging};
use thermite_gfx::{
    window::Window,
    winit::{
        event::{ElementState, Event as WinitEvent, WindowEvent},
        event_loop::ControlFlow,
    },
};
use uuid::Uuid;

// TODO: Make this a Singleton
pub struct Application {
    event_bus: Rc<RefCell<ThermiteEventBus>>, // Single-threaded, for now
    window: Window<ThermiteEvent>,
    publ: Rc<TestPublisher>,
    sub: Rc<TestSubscriber>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            event_bus: Rc::new(RefCell::new(
                EventBus::<ThermiteEventType, ThermiteEvent>::default(),
            )),
            window: Window::default(),
            publ: Rc::new(TestPublisher {}),
            sub: Rc::new(TestSubscriber {
                id: Uuid::default(),
            }),
        }
    }
}

impl Application {
    pub fn new(name: &str, size: [u32; 2]) -> Self {
        Self {
            event_bus: Rc::new(RefCell::new(
                EventBus::<ThermiteEventType, ThermiteEvent>::default(),
            )),
            window: Window::new(name, size).expect("Couldn't create window"),
            publ: Rc::new(TestPublisher {}),
            sub: Rc::new(TestSubscriber {
                id: Uuid::default(),
            }),
        }
    }

    fn init(&mut self) {
        thermite_logging::init().expect("Couldn't initialize logging");
        // Subscribe our subscriber to Input events
        self.event_bus
            .try_borrow_mut()
            .expect("Couldn't borrow event bus as mutable")
            .subscribe(&self.sub, ThermiteEventType::Input);
    }

    pub fn run(&mut self) {
        self.init();
        // Event loop requires ownership of captured environment, just clone our rc pointers for it to take...
        let eb = self.event_bus.clone();
        let publ = self.publ.clone();
        self.window
            .event_loop()
            .run(move |event, _, control_flow| match event {
                // Pre-event handling code
                WinitEvent::NewEvents(_) => (),
                // Custom events
                WinitEvent::UserEvent(_) => (),
                // Events coming straight from hardware devices
                WinitEvent::DeviceEvent { .. } => (),
                // Events emitted by the winit window
                WinitEvent::WindowEvent { event, .. } => match event {
                    // TODO: Would be nice to not have a monolithic handler...
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input.state {
                        ElementState::Pressed => {
                            let evt = KeyboardEvent::KeyPressed(input.into());
                            publ.publish_event(
                                &evt.into(),
                                &mut eb
                                    .try_borrow_mut()
                                    .expect("Couldn't borrow the event bus as mutable"),
                            );
                        }
                        ElementState::Released => {
                            let evt = KeyboardEvent::KeyReleased(input.into());
                            publ.publish_event(
                                &evt.into(),
                                &mut eb
                                    .try_borrow_mut()
                                    .expect("Couldn't borrow the event bus as mutable"),
                            );
                        }
                    },
                    WindowEvent::ModifiersChanged(modifiers_state) => {
                        let evt = KeyboardEvent::ModifiersChanged(modifiers_state.into());
                        publ.publish_event(
                            &evt.into(),
                            &mut eb
                                .try_borrow_mut()
                                .expect("Couldn't borrow the event bus as mutable"),
                        );
                    }
                    WindowEvent::MouseInput { state, button, .. } => match state {
                        ElementState::Pressed => {
                            let evt = MouseEvent::ButtonPressed(button);
                            publ.publish_event(
                                &evt.into(),
                                &mut eb
                                    .try_borrow_mut()
                                    .expect("Couldn't borrow the event bus as mutable"),
                            );
                        }
                        ElementState::Released => {
                            let evt = MouseEvent::ButtonReleased(button);
                            publ.publish_event(
                                &evt.into(),
                                &mut eb
                                    .try_borrow_mut()
                                    .expect("Couldn't borrow the event bus as mutable"),
                            );
                        }
                    },
                    WindowEvent::MouseWheel { delta, .. } => {
                        let evt = MouseEvent::Scroll(delta.into());
                        publ.publish_event(
                            &evt.into(),
                            &mut eb
                                .try_borrow_mut()
                                .expect("Couldn't borrow the event bus as mutable"),
                        );
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        // ! Leaving this commented out for now as it's really noisy
                        // let evt = MouseEvent::Motion(position.into());
                        // publ.publish_event(
                        //     &evt.into(),
                        //     &mut eb
                        //         .try_borrow_mut()
                        //         .expect("Couldn't borrow the event bus as mutable"),
                        // );
                    }
                    WindowEvent::CursorEntered { .. } => {
                        let evt = MouseEvent::EnteredWindow;
                        publ.publish_event(
                            &evt.into(),
                            &mut eb
                                .try_borrow_mut()
                                .expect("Couldn't borrow the event bus as mutable"),
                        );
                    }
                    WindowEvent::CursorLeft { .. } => {
                        let evt = MouseEvent::LeftWindow;
                        publ.publish_event(
                            &evt.into(),
                            &mut eb
                                .try_borrow_mut()
                                .expect("Couldn't borrow the event bus as mutable"),
                        );
                    }
                    _ => (),
                },
                // Continuous dynamic graphics rendering (loop "main body")
                WinitEvent::MainEventsCleared => (),
                // Static graphics rendering (mainly for semi-static GUIs, etc.)
                WinitEvent::RedrawRequested(_) => (),
                // Rendering cleanup
                WinitEvent::RedrawEventsCleared => (),
                // Application resumed
                WinitEvent::Resumed => (),
                // Application suspended
                WinitEvent::Suspended => (),
                // Last event to be emitted, period.
                WinitEvent::LoopDestroyed => (),
            });
    }
}
