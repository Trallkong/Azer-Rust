use crate::api::vulkan_context::VulkanContext;
use crate::render::render_triangle::RenderTriangle;
use std::sync::{Arc, Mutex};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::render_pass::Framebuffer;
use winit::window::Window;

pub struct Renderer {
    cmd_bf_builder: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    render_triangle: Box<RenderTriangle>,

    context: Arc<Mutex<VulkanContext>>,
}

impl Renderer {
    pub fn new(
        window: Arc<Window>,
        context: Arc<Mutex<VulkanContext>>,
    ) -> Self {

        let allocator = Arc::new(StandardCommandBufferAllocator::new(
            Arc::clone(&context.lock().unwrap().device),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));

        let builder =
            AutoCommandBufferBuilder::primary(
                allocator.clone(),
                context.lock().unwrap().queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            ).unwrap();


        let render_triangle = Box::new(
            RenderTriangle::new(
                Arc::clone(&window),
                Arc::clone(&context),
            ),
        );

        Self {
            cmd_bf_builder: Some(builder),
            render_triangle,
            context,
        }
    }

    pub fn recreate_builder(&mut self) {
        let cmd_bf_allocator = self.context.lock().unwrap().cmd_bf_allocator.clone();

        self.cmd_bf_builder = Some(AutoCommandBufferBuilder::primary(
            cmd_bf_allocator,
            self.context.lock().unwrap().queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        ).unwrap());
    }

    pub fn begin(
        &mut self,
        framebuffer: Arc<Framebuffer>,
        clear_color: [f32; 4],
    ) {
        let mut builder = self.cmd_bf_builder.take().unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some(clear_color.into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..SubpassBeginInfo::default()
                }
            )
            .unwrap()
        ;

        self.cmd_bf_builder = Some(builder);
    }

    pub fn end(
        &mut self,
    ) {
        let mut builder = self.cmd_bf_builder.take().unwrap();

        builder
            .end_render_pass(SubpassEndInfo::default())
            .unwrap();

        self.cmd_bf_builder = Some(builder);
    }

    pub fn submit(&mut self) -> Arc<PrimaryAutoCommandBuffer> {
        let builder = self.cmd_bf_builder.take().unwrap();
        let command_buffer = builder.build().unwrap();
        self.recreate_builder();
        command_buffer
    }

    pub fn draw_triangle(&mut self) {
        let builder = self.cmd_bf_builder.take().unwrap();
        self.cmd_bf_builder = Some(self.render_triangle.draw(builder));
    }

    pub fn recreate_pipeline(&mut self) {
        self.render_triangle.recreate_pipeline();
    }
}