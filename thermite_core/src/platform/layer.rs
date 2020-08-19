// use crate::platform::event::{Publisher, Subscriber};
use std::boxed::Box;
use std::ops::Deref;

pub trait Layer /*: Publisher + Subscriber*/ {
    fn on_attach(&self);
    fn on_detach(&self);
    fn on_update(&self);
    // TODO: Enable/Disable layer

    fn identifier(&self) -> u32;
    fn debug_name(&self) -> &str;
}

pub struct LayerStack {
    layers: Vec<Box<dyn Layer>>,
}

impl Default for LayerStack {
    fn default() -> Self {
        LayerStack { layers: vec![] }
    }
}

impl LayerStack {
    pub fn push(&mut self, layer: Box<dyn Layer>) {
        layer.on_attach();
        self.layers.push(layer);
    }

    pub fn pop(&mut self) {
        if let Some(layer) = self.layers.pop() {
            layer.on_detach();
        }
    }

    pub fn remove(&mut self, layer: &dyn Layer) {
        match self
            .layers
            .iter()
            .position(|l| l.identifier() == layer.identifier())
        {
            Some(idx) => {
                let layer = self.layers.remove(idx);
                layer.on_detach();
            }
            _ => (),
        }
    }
}

impl Deref for LayerStack {
    type Target = [Box<dyn Layer>];

    fn deref(&self) -> &Self::Target {
        &self.layers
    }
}
