pub use winit::event::WindowEvent;
pub use crate::core::delta_time::DeltaTime;
use crate::render::renderer::Renderer;

pub trait Layer: Send + Sync {
    fn on_ready(&mut self);
    fn on_update(&mut self, delta: &DeltaTime);
    fn on_render(&mut self, renderer: &mut Renderer);
    fn on_physics_update(&mut self, delta: &DeltaTime);
    fn on_event(&mut self, event: &WindowEvent);
    fn on_close(&mut self);
}