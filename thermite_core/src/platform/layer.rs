use crate::platform::event::{Publisher, Subscriber};
use std::boxed::Box;
use std::ops::Deref;

pub trait Layer<E: Event>: Publisher<E> + Subscriber<E> {
    fn on_attach(&self);
    fn on_detach(&self);
    fn on_update(&self);
    // TODO: Enable/Disable layer
    fn id(&self) -> u32;
    fn debug_name(&self) -> &str;
}

pub struct LayerStack<E: Event> {
    layers: Vec<Box<dyn Layer<E>>>,
    layer_boundary_idx: usize,
}

impl<E: Event> Default for LayerStack<E> {
    fn default() -> Self {
        LayerStack {
            layers: vec![],
            layer_boundary_idx: 0,
        }
    }
}

impl<E: Event> LayerStack<E> {
    pub fn push_layer(&mut self, layer: Box<dyn Layer<E>>) {
        layer.on_attach();
        self.layers.insert(self.layer_boundary_idx, layer);
        self.layer_boundary_idx += 1;
    }

    pub fn pop_layer(&mut self, layer: Box<dyn Layer<E>>) -> Option<Box<dyn Layer<E>>> {
        if let Some(idx) = self.layers.iter().position(|l| l.id() == layer.id()) {
            let removed = self.layers.remove(idx);
            removed.on_detach();
            self.layer_boundary_idx -= 1;
            Some(removed)
        } else {
            None
        }
    }

    pub fn push_overlay(&mut self, overlay: Box<dyn Layer<E>>) {
        overlay.on_attach();
        self.layers.push(overlay);
    }

    pub fn pop_overlay(&mut self) -> Option<Box<dyn Layer<E>>> {
        if let Some(overlay) = self.layers.pop() {
            overlay.on_detach();
            Some(overlay)
        } else {
            None
        }
    }
}

impl<E: Event> Deref for LayerStack<E> {
    type Target = [Box<dyn Layer<E>>];

    fn deref(&self) -> &Self::Target {
        &self.layers
    }
}
