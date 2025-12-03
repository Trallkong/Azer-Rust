use std::sync::{Arc, Mutex};
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::device::{Device, Queue};
use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use winit::window::Window;
use crate::api::shader::Shaders;
use crate::api::vulkan_context::VulkanContext;
use crate::render::render_triangle::Vertex2D;



pub struct VulkanHelper;

impl VulkanHelper {
    pub fn create_graphics_pipeline(window: Arc<Window>, context: Arc<Mutex<VulkanContext>>) -> Arc<GraphicsPipeline>   {
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let shaders = Shaders::load(context.lock().unwrap().device.clone()).unwrap();
        let vs = shaders.vs.entry_point("main").unwrap();
        let fs = shaders.fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex2D::per_vertex()
            .definition(&vs)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let device = context.lock().unwrap().device.clone();

        let layout = PipelineLayout::new(
            device,
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(context.lock().unwrap().device.clone()).unwrap()
        ).unwrap();

        let subpass = Subpass::from(context.lock().unwrap().render_pass.clone(), 0).unwrap();

        let pipeline = GraphicsPipeline::new(
            context.lock().unwrap().device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..ViewportState::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default()
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            }
        ).unwrap();

        pipeline
    }

    pub fn create_command_buffers(
        device: Arc<Device>,
        queue: Arc<Queue>,
        framebuffers: Vec<Arc<Framebuffer>>,
        pipeline: Arc<GraphicsPipeline>,
        vertex_buffer: Subbuffer<[Vertex2D]>) -> Vec<Arc<PrimaryAutoCommandBuffer>> {

        let allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));

        let mut command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>> = Vec::new();
        framebuffers.into_iter().for_each(|framebuffer| {
            let mut builder =
                AutoCommandBufferBuilder::primary(
                    allocator.clone(),
                    queue.queue_family_index(),
                    CommandBufferUsage::OneTimeSubmit,
                ).unwrap();

            unsafe {
                builder
                    .begin_render_pass(
                        RenderPassBeginInfo {
                            clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                        },
                        SubpassBeginInfo {
                            contents: SubpassContents::Inline,
                            ..SubpassBeginInfo::default()
                        },
                    )
                    .unwrap()
                    .bind_pipeline_graphics(pipeline.clone())
                    .unwrap()
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .unwrap()
                    .draw(
                        3, 1, 0, 0,
                    )
                    .unwrap()
                    .end_render_pass(SubpassEndInfo::default())
                    .unwrap();
            }

            let command_buffer = builder.build().unwrap();

            command_buffers.push(command_buffer);
        });

        command_buffers
    }

    pub fn create_frame_buffers(images: Vec<Arc<Image>>, render_pass: Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
        let mut framebuffers: Vec<Arc<Framebuffer>> = Vec::new();

        images.iter().for_each(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            let framebuffer = Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..FramebufferCreateInfo::default()
                }
            ).unwrap_or_else(|err| panic!("创建帧缓冲区失败: {}", err));

            framebuffers.push(framebuffer);
        });

        framebuffers
    }
}