use log::info;
use winit::event::WindowEvent;
use azer::core::delta_time::DeltaTime;
use azer::core::layer::Layer;
use azer::render::renderer::Renderer;

pub struct NewLayer;

impl Layer for NewLayer {
    fn on_ready(&mut self) {
        info!("NewLayer ready");
    }

    fn on_update(&mut self, _delta: &DeltaTime) {
        // info!("NewLayer update");
    }

    fn on_render(&mut self, renderer: &mut Renderer) {
        info!("NewLayer rendering");
        renderer.draw_triangle();
    }

    fn on_physics_update(&mut self, _delta: &DeltaTime) {
        // info!("NewLayer physics update");
    }

    fn on_event(&mut self, event: &WindowEvent) {
        // info!("{:?}", event);
    }

    fn on_close(&mut self) {
        info!("NewLayer close");
    }
}