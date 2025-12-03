use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::{Image, ImageUsage};
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo};
use winit::window::Window;

pub struct SwapChain {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
}

impl SwapChain {
    pub fn new(
        device: Arc<Device>,
        surface: Arc<Surface>,
        window: Arc<Window>,
    ) -> Self {

        let swapchain_create_info = SwapchainCreateInfo {
            image_format: Format::R8G8B8A8_UNORM,
            image_extent: window.inner_size().into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            present_mode: PresentMode::Fifo,
            ..SwapchainCreateInfo::default()
        };

        let (swapchain, images) =
            Swapchain::new(
                Arc::clone(&device),
                Arc::clone(&surface),
                swapchain_create_info
            )
                .unwrap_or_else(|err| panic!("图像交换链创建失败: {}", err));

        Self{
            swapchain,
            images,
        }
    }
}