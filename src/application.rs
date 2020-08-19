use log::info;
use std::sync::{Arc, Mutex};
use thermite_core::{
    input::{keyboard::KeyboardEvent, mouse::MouseEvent},
    platform::event::EventBus,
    thermite_logging,
};
use thermite_gfx::winit::{
    event::{ElementState, Event as WinitEvent, WindowEvent},
    event_loop::ControlFlow,
};
use thermite_gfx::{hal::hal_state::HALState, window::Window};

// TODO: Make this a Singleton
pub struct Application {
    event_bus: Arc<Mutex<EventBus>>,
    hal_state: HALState,
    window: Window,
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

    fn init(&self) {
        thermite_logging::init().expect("Couldn't initialize logging");
    }

    pub fn run(&mut self) {
        self.init();
        let eb = self.event_bus.clone();
        self.window
            .event_loop()
            .run(move |event, _, control_flow| match event {
                WinitEvent::UserEvent(event) => (),
                WinitEvent::DeviceEvent { device_id, event } => (),
                WinitEvent::WindowEvent { window_id, event } => match event {
                    // TODO: Would be nice to not have a monolithic handler...
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    } => match input.state {
                        ElementState::Pressed => {
                            let evt = KeyboardEvent::KeyPressed(input.into());
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt);
                        }
                        ElementState::Released => {
                            let evt = KeyboardEvent::KeyReleased(input.into());
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt);
                        }
                    },
                    WindowEvent::ModifiersChanged(modifiers_state) => eb
                        .lock()
                        .unwrap()
                        .dispatch_event(&KeyboardEvent::ModifiersChanged(modifiers_state.into())),
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => match state {
                        ElementState::Pressed => {
                            let evt = MouseEvent::ButtonPressed(button);
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt);
                        }
                        ElementState::Released => {
                            let evt = MouseEvent::ButtonReleased(button);
                            info!("{:?}", evt);
                            eb.lock().unwrap().dispatch_event(&evt);
                        }
                    },
                    WindowEvent::MouseWheel {
                        device_id, delta, ..
                    } => {
                        let evt = MouseEvent::Scroll(delta);
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt);
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        ..
                    } => {
                        let evt = MouseEvent::Motion(position);
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt);
                    }
                    WindowEvent::CursorEntered { device_id } => {
                        let evt = MouseEvent::EnteredWindow;
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt);
                    }
                    WindowEvent::CursorLeft { device_id } => {
                        let evt = MouseEvent::LeftWindow;
                        info!("{:?}", evt);
                        eb.lock().unwrap().dispatch_event(&evt);
                    }
                    _ => (),
                },
                WinitEvent::NewEvents(start_cause) => (),
                WinitEvent::MainEventsCleared => (), //win.lock().unwrap().handle().request_redraw(), // TODO: <-- figure this out!
                WinitEvent::RedrawEventsCleared => (),
                WinitEvent::RedrawRequested(window_id) => (),
                WinitEvent::LoopDestroyed => (),
                WinitEvent::Resumed => (),
                WinitEvent::Suspended => (),
            });
    }
}
