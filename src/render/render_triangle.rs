use crate::api::vulkan_context::VulkanContext;
use crate::api::vulkan_helper::VulkanHelper;
use std::sync::{Arc, Mutex};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::vertex_input::Vertex;
use vulkano::pipeline::GraphicsPipeline;
use winit::window::Window;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex2D {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub struct RenderTriangle {
    pub graphics_pipeline: Arc<GraphicsPipeline>,
    pub vertex_buffer: Subbuffer<[Vertex2D]>,

    pub window: Arc<Window>,
    pub context: Arc<Mutex<VulkanContext>>
}

impl RenderTriangle {
    pub fn new(
        window: Arc<Window>,
        context: Arc<Mutex<VulkanContext>>,
    ) -> RenderTriangle {

        let allocator =
            Arc::new(StandardMemoryAllocator::new_default(Arc::clone(&context.lock().unwrap().device)));

        let vbo =
            RenderTriangle::get_vertex_buffer(Arc::clone(&allocator));

        let pipeline = VulkanHelper::create_graphics_pipeline(
            Arc::clone(&window),
            Arc::clone(&context),
        );

        RenderTriangle {
            graphics_pipeline: pipeline,
            vertex_buffer: vbo,
            window,
            context
        }
    }

    pub fn draw(&self, mut cmd_bf_builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) -> AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> {
        unsafe {
            cmd_bf_builder
                .bind_pipeline_graphics(Arc::clone(&self.graphics_pipeline))
                .unwrap()
                .bind_vertex_buffers(0, self.vertex_buffer.clone())
                .unwrap()
                .draw(3, 1, 0, 0)
                .unwrap();
        }

        cmd_bf_builder
    }

    pub fn get_vertex_buffer(allocator: Arc<StandardMemoryAllocator>) -> Subbuffer<[Vertex2D]> {
        let vertices = vec![
            Vertex2D { position: [-0.5, 0.5] },
            Vertex2D { position: [0.5, 0.5] },
            Vertex2D { position: [0.0, -0.5] },
        ];

        let vertex_buffer = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..BufferCreateInfo::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..AllocationCreateInfo::default()
            },
            vertices
        ).unwrap_or_else(|_| panic!("Failed to create vertex buffer"));

        vertex_buffer
    }

    pub fn recreate_pipeline(&mut self) {
        self.graphics_pipeline = VulkanHelper::create_graphics_pipeline(
            Arc::clone(&self.window),
            Arc::clone(&self.context),
        );
    }
}