use log::info;
use std::sync::{Arc, Mutex, RwLock};
use thermite_core::{
    input::{keyboard::KeyboardEvent, mouse::MouseEvent},
    platform::event::{
        BusRequest, EventBus, Publisher, Subscriber, ThermiteEvent, ThermiteEventType,
    },
    thermite_logging,
};
use thermite_gfx::window::Window;
use thermite_gfx::winit::{
    event::{ElementState, Event as WinitEvent, WindowEvent},
    event_loop::ControlFlow,
};

// ============================== TEST STRUCTS ============================== //
pub struct TestSubscriber {}
impl Subscriber<ThermiteEvent> for TestSubscriber {
    // ! Although we get a ThermiteEvent enum, it is guaranteed to be only of the category that we are subscribed to
    fn on_event(&self, event: &ThermiteEvent) -> BusRequest {
        info!("Test subscriber received event: {:?}", event);
        BusRequest::NoActionNeeded
    }
}

pub struct TestPublisher {}
impl Publisher<ThermiteEventType, ThermiteEvent> for TestPublisher {}
// ============================== END TEST STRUCTS ============================== //

type ThermiteEventBus = EventBus<ThermiteEventType, ThermiteEvent>;
// TODO: Make this a Singleton
pub struct Application {
    event_bus: Arc<Mutex<ThermiteEventBus>>,
    window: Window<ThermiteEvent>,
    publ: Arc<Mutex<TestPublisher>>, // TODO: Figure out how to get publishers and subscribers to operate from other parts of the application
    sub: Arc<RwLock<TestSubscriber>>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            event_bus: Arc::new(Mutex::new(EventBus::default())),
            window: Window::default(),
            publ: Arc::new(Mutex::new(TestPublisher {})),
            sub: Arc::new(RwLock::new(TestSubscriber {})),
        }
    }
}

impl Application {
    pub fn new(name: &str, size: [u32; 2]) -> Self {
        Self {
            event_bus: Arc::new(Mutex::new(EventBus::default())),
            window: Window::new(name, size).expect("Couldn't create window"),
            publ: Arc::new(Mutex::new(TestPublisher {})),
            sub: Arc::new(RwLock::new(TestSubscriber {})),
        }
    }

    fn init(&mut self) {
        thermite_logging::init().expect("Couldn't initialize logging");
        // Subscribe our subscriber to Input events
        self.event_bus
            .lock()
            .unwrap()
            .subscribe(&self.sub, ThermiteEventType::Input);
    }

    pub fn run(&mut self) {
        self.init();
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
                            publ.lock()
                                .unwrap()
                                .publish_event(&evt.into(), &mut eb.lock().unwrap());
                        }
                        ElementState::Released => {
                            let evt = KeyboardEvent::KeyReleased(input.into());
                            publ.lock()
                                .unwrap()
                                .publish_event(&evt.into(), &mut eb.lock().unwrap());
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
                            publ.lock()
                                .unwrap()
                                .publish_event(&evt.into(), &mut eb.lock().unwrap());
                        }
                        ElementState::Released => {
                            let evt = MouseEvent::ButtonReleased(button);
                            publ.lock()
                                .unwrap()
                                .publish_event(&evt.into(), &mut eb.lock().unwrap());
                        }
                    },
                    WindowEvent::MouseWheel { delta, .. } => {
                        let evt = MouseEvent::Scroll(delta.into());
                        publ.lock()
                            .unwrap()
                            .publish_event(&evt.into(), &mut eb.lock().unwrap());
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        // ! Leaving this commented out for now as it's really noisy
                        // let evt = MouseEvent::Motion(position.into());
                        // publ.lock()
                        //     .unwrap()
                        //     .publish_event(&evt.into(), &mut eb.lock().unwrap());
                    }
                    WindowEvent::CursorEntered { .. } => {
                        let evt = MouseEvent::EnteredWindow;
                        publ.lock()
                            .unwrap()
                            .publish_event(&evt.into(), &mut eb.lock().unwrap());
                    }
                    WindowEvent::CursorLeft { .. } => {
                        let evt = MouseEvent::LeftWindow;
                        publ.lock()
                            .unwrap()
                            .publish_event(&evt.into(), &mut eb.lock().unwrap());
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
