use crate::core::layer::Layer;

#[derive(Default)]
pub struct LayerStack {
    stack: Vec<Box<dyn Layer>>,
}

impl LayerStack {
    pub fn new() -> LayerStack {
        LayerStack{
            stack: Vec::new(),
        }
    }

    pub fn push(&mut self, layer: Box<dyn Layer>) {
        self.stack.push(layer);
    }

    pub fn pop(&mut self) -> Option<Box<dyn Layer>> {
        self.stack.pop()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn Layer>> {
        self.stack.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn Layer>> {
        self.stack.iter_mut()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}