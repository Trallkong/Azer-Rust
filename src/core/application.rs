use std::sync::Arc;
use std::time::Instant;
use log::{info, warn};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::{HasRawWindowHandle, HasWindowHandle};
use winit::window::{Window, WindowId};
use crate::api::vulkan::Vulkan;
use crate::core::delta_time::DeltaTime;
use crate::core::layer::Layer;
use crate::core::layer_stack::LayerStack;
use crate::render::renderer::Renderer;

const FIXED_PHYSICS_STEP: f64 = 1.0/60.0; // 固定物理步长
const MAX_PHYSICS_STEPS: usize = 10; // 最大物理步次

#[derive (Default)]
pub struct Application {
    window: Option<Arc<Window>>,        // 窗口
    event: Option<WindowEvent>,         // 分发事件
    layer_stack: Option<LayerStack>,    // 层栈

    last_time: Option<Instant>,         // 上一帧的时间
    accumulated_time: f64,              // 物理步长累计时间

    vulkan: Option<Vulkan>,             // vulkan核心封装
    initialized: bool,                  // 初始化标志

    renderer: Option<Renderer>,         // 渲染器核心
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        warn!("Resuming application");

        if !self.initialized {
            self.initialized = true;
            // 初始化各层
            let mut layer_stack = self.layer_stack.take().unwrap();
            layer_stack.iter_mut().for_each(|layer| layer.on_ready());
            self.layer_stack = Some(layer_stack);

            // 初始化 Window
            let window_attribute = Window::default_attributes()
                .with_title("Azer")
                .with_inner_size(winit::dpi::PhysicalSize::new(1280,720));

            let window = Arc::new(event_loop.create_window(window_attribute.clone()).unwrap());
            let vulkan = Vulkan::new(window.clone());

            info!("Vulkan created");

            // 初始化 renderer
            self.renderer = Some(Renderer::new(
                Arc::clone(&window),
                Arc::clone(&vulkan.context),
            ));

            info!("renderer created");

            // 归还数据
            self.window = Some(window.clone());
            self.vulkan = Some(vulkan);

            info!("Vulkan resumed");
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        // 逻辑更新
        let mut layer_stack = self.layer_stack.take().unwrap();
        let current_time = Instant::now();
        let duration = current_time.duration_since(self.last_time.unwrap()).as_secs_f64().max(0.0);
        self.last_time = Some(current_time);
        physics_update(&mut layer_stack, duration, &mut self.accumulated_time);
        update(&mut layer_stack, duration);
        self.layer_stack = Some(layer_stack);

        // 事件监听
        match event {
            WindowEvent::CloseRequested => {
                warn!("检测到点击关闭按钮，开始清理，请不要退出应用！");

                // 清理层栈
                let mut layer_stack = self.layer_stack.take().unwrap();
                layer_stack.iter_mut().for_each(|layer| layer.on_close());
                layer_stack.clear();

                event_loop.exit(); // 关闭事件循环
                warn!("清理完毕！");
                return;
            },
            WindowEvent::RedrawRequested => {
                let mut vulkan = self.vulkan.take().unwrap();
                let window = self.window.take().unwrap();
                let mut renderer = self.renderer.take().unwrap();
                let mut layer_stack = self.layer_stack.take().unwrap();

                vulkan.recreate_swapchain(
                    window.clone(),
                    &mut renderer,
                    &mut layer_stack,
                );

                vulkan.submit();

                self.layer_stack = Some(layer_stack);
                self.renderer = Some(renderer);
                self.vulkan = Some(vulkan);
                self.window = Some(window);
                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::Resized(size) => {
                let mut vulkan = self.vulkan.take().unwrap();
                let window = self.window.take().unwrap();
                if size.width > 0 && size.height > 0 {
                    vulkan.window_resized = true;
                    vulkan.recreate_swapchain(
                        window.clone(),
                        self.renderer.as_mut().unwrap(),
                        self.layer_stack.as_mut().unwrap()
                    );
                }
                self.vulkan = Some(vulkan);
                self.window = Some(window);

                self.event = Some(WindowEvent::Resized(size));
            },
            WindowEvent::CursorMoved {
                device_id, position
            } => {
                self.event = Some(WindowEvent::CursorMoved {
                    device_id, position
                });
            },
            WindowEvent::MouseInput {
                device_id, state, button
            } => {
                self.event = Some(WindowEvent::MouseInput {
                    device_id, state, button
                });
            },
            WindowEvent::KeyboardInput {
                device_id, event, is_synthetic
            } => {
                self.event = Some(WindowEvent::KeyboardInput {
                    device_id, event, is_synthetic
                })
            },
            _ => ()
        }

        // 事件分发
        let mut layer_stack = self.layer_stack.take().unwrap();
        let current_event = self.event.take();
        match current_event {
            Some(event) => {
                layer_stack.iter_mut().for_each(|layer| layer.on_event(&event));
            },
            None => ()
        }
        self.layer_stack = Some(layer_stack);
    }
}

impl Application {
    pub fn new() -> Application {
        Application {
            window: None,
            event: None,
            layer_stack: Some(LayerStack::new()),
            last_time: Some(Instant::now()),
            accumulated_time: 0.0,
            vulkan: None,
            initialized: false,
            renderer: None,
        }
    }
    pub fn push_layer(&mut self, layer: Box<dyn Layer>) {
        let mut layer_stack = self.layer_stack.take().expect("请先初始化LayerStack");
        layer_stack.push(layer);
        self.layer_stack = Some(layer_stack);
    }
}

pub fn physics_update(layer_stack: &mut LayerStack, duration: f64, accumulated_time: &mut f64) {
    let mut step: usize = 0;
    *accumulated_time += duration;
    while *accumulated_time > FIXED_PHYSICS_STEP && step < MAX_PHYSICS_STEPS {
        step += 1;
        *accumulated_time -= FIXED_PHYSICS_STEP;
        layer_stack.iter_mut().for_each(|layer| {
            layer.on_physics_update(&DeltaTime::new(FIXED_PHYSICS_STEP));
        })
    }
}

pub fn update(layer_stack: &mut LayerStack, duration: f64) {
    layer_stack.iter_mut().for_each(|layer| {
        layer.on_update(&DeltaTime::new(duration));
    });
}