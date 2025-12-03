mod new_layer;

use std::thread;
use std::time::Duration;
use log::info;
use winit::event_loop::{ControlFlow, EventLoop};
use azer::core::{logger, application::Application};
use crate::new_layer::NewLayer;

fn main() {
    // 日志模块
    logger::init_logger();
    info!("日志模块初始化成功！");

    // 窗口模块
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    info!("窗口模块初始化成功！");

    let mut app: Application = Application::new();
    app.push_layer(Box::new(NewLayer));

    event_loop.run_app(&mut app).unwrap();

    thread::sleep(Duration::from_secs(1));
}
