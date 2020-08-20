use log::info;
use std::sync::{Arc, Mutex};
use thermite_core::{
    input::{keyboard::KeyboardEvent, mouse::MouseEvent},
    platform::event::{EventBus, ThermiteEvent},
    thermite_logging,
};
use thermite_gfx::winit::{
    event::{ElementState, Event as WinitEvent, WindowEvent},
    event_loop::ControlFlow,
};
use thermite_gfx::{hal::hal_state::HALState, window::Window};

// TODO: Make this a Singleton
pub struct Application {
    event_bus: Arc<Mutex<EventBus<ThermiteEvent>>>,
    hal_state: HALState,
    window: Window<ThermiteEvent>,
}

impl Default for Application {
    fn default() -> Self {
        let window = Window::default();
        let hal_state = HALState::new(window.handle()).expect("Couldn't create HALState");
        Self {
            event_bus: Arc::new(Mutex::new(EventBus::default())),
            hal_state: hal_state,
            window: window,
        }
    }
}

impl Application {
    pub fn new(name: &str, size: [u32; 2]) -> Self {
        let window = Window::new(name, size).expect("Couldn't create window");
        let hal_state = HALState::new(window.handle()).expect("Couldn't create HALState");
        Self {
            event_bus: Arc::new(Mutex::new(EventBus::default())),
            hal_state: hal_state,
            window: window,
        }
    }

    fn init(&mut self) {
        thermite_logging::init().expect("Couldn't initialize logging");
    }

    pub fn run(&mut self) {
        self.init();
        let eb = self.event_bus.clone(); // Clone our rc pointer so the static closure can take ownership of it
        self.window
            .event_loop()
            .run(move |event, _, control_flow| match event {
                // Pre-event handling code
                WinitEvent::NewEvents(_) => (),
                // Custom events
                WinitEvent::UserEvent(_) => (),
                // Events coming strait from hardware devices
                WinitEvent::DeviceEvent { .. } => (),
                // Events emitted by the winit window
                WinitEvent::WindowEvent { event, .. } => match event {
                    // TODO: Would be nice to not have a monolithic handler...
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input.state {
                        ElementState::Pressed => {
                            let evt = KeyboardEvent::KeyPressed(input.into());
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt.into());
                        }
                        ElementState::Released => {
                            let evt = KeyboardEvent::KeyReleased(input.into());
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt.into());
                        }
                    },
                    WindowEvent::ModifiersChanged(modifiers_state) => {
                        eb.lock().unwrap().dispatch_event(
                            &KeyboardEvent::ModifiersChanged(modifiers_state.into()).into(),
                        )
                    }
                    WindowEvent::MouseInput { state, button, .. } => match state {
                        ElementState::Pressed => {
                            let evt = MouseEvent::ButtonPressed(button);
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt.into());
                        }
                        ElementState::Released => {
                            let evt = MouseEvent::ButtonReleased(button);
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt.into());
                        }
                    },
                    WindowEvent::MouseWheel { delta, .. } => {
                        let evt = MouseEvent::Scroll(delta.into());
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt.into());
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        let evt = MouseEvent::Motion(position.into());
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt.into());
                    }
                    WindowEvent::CursorEntered { .. } => {
                        let evt = MouseEvent::EnteredWindow;
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt.into());
                    }
                    WindowEvent::CursorLeft { .. } => {
                        let evt = MouseEvent::LeftWindow;
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt.into());
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
