pub mod application;
use application::Application;

fn main() {
    let mut app = Application::new("Test Application", [800, 600]);
    app.run();
}
