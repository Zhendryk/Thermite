pub mod application;
use application::Application;
pub mod event;

fn main() {
    let mut app = Application::new("Test Application", [800, 600]);
    app.run();
}
