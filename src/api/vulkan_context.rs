use std::sync::Arc;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{CommandBuffer, PrimaryAutoCommandBuffer};
use vulkano::device::{Device, Queue};
use vulkano::image::Image;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{Surface, Swapchain};

pub struct VulkanContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub cmd_bf_allocator: Arc<StandardCommandBufferAllocator>,
}