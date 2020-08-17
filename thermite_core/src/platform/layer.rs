pub trait Layer {
    fn on_attach();
    fn on_detach();
    fn on_update();
    fn on_event();

    fn get_name(&self) -> &str;
}
