use std::sync::{Arc, Mutex};
use log::{error, info};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{PrimaryAutoCommandBuffer};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags};
use vulkano::format::{Format};
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{acquire_next_image, Surface, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::{single_pass_renderpass, sync, Validated, VulkanError, VulkanLibrary};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::sync::GpuFuture;
use winit::window::Window;
use crate::api;
use crate::api::vulkan_helper::VulkanHelper;
use crate::core::layer_stack::LayerStack;
use crate::render::renderer::Renderer;
use crate::api::vulkan_context::VulkanContext;

pub struct Vulkan {
    pub context: Arc<Mutex<VulkanContext>>,
    pub window_resized: bool,
    pub recreate_swapchain: bool,
}

impl Vulkan {
    pub fn new(window: Arc<Window>) -> Vulkan {

        let library = VulkanLibrary::new()
            .unwrap_or_else(|err| panic!("无法创建VulkanLibrary: {}",err));

        let required_extensions = Surface::required_extensions(&window)
            .unwrap_or_else(|err| panic!("获取窗口所需扩展失败: {}", err));

        let extensions = InstanceExtensions {
            ..required_extensions
        };

        let instance = Instance::new(
            library.clone(),
            InstanceCreateInfo {
                enabled_extensions: extensions,
                ..InstanceCreateInfo::default()
            }
        ).unwrap_or_else(|err| panic!("无法创建Vulkan实例: {}", err));

        let surface = Surface::from_window(instance.clone(), window.clone())
            .unwrap_or_else(|err| panic!("创建Surface失败: {}", err));

        let mut target_device : Option<Arc<PhysicalDevice>> = None;
        let mut target_index: Option<u32> = None;
        // 遍历物理设备搜寻符合要求的设备
        for physical_device in instance.enumerate_physical_devices().unwrap() {
            println!("物理设备信息:");
            println!("设备名称: {}", physical_device.properties().device_name);
            println!("设备类型: {:?}", physical_device.properties().device_type);

            // 检测是否包含图形队列
            let required_index = get_required_queue_family_index(
                physical_device.as_ref(), QueueFlags::GRAPHICS);

            match required_index {
                Some(index) => {
                    println!("该设备支持图形队列，被选择为目标物理设备！");
                    target_device = Some(physical_device);
                    target_index = Some(index);
                    break;
                },
                None => {
                    println!("该设备不支持图形队列，跳过！");
                }
            }
        };

        if target_device.is_none() {
            panic!("没有找到适用于创建设备队列的物理设备！")
        }

        let queue_create_info = QueueCreateInfo {
            queue_family_index: target_index.unwrap(),
            queues: vec![1.0],
            ..QueueCreateInfo::default()
        };

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::default()
        };

        let device_create_info = DeviceCreateInfo {
            queue_create_infos: vec![queue_create_info],
            enabled_extensions: device_extensions,
            ..DeviceCreateInfo::default()
        };

        let (device, mut queues) = Device::new(target_device.unwrap(), device_create_info)
            .unwrap_or_else(|err| panic!("创建设备失败: {}",err));

        let queue = queues.next().unwrap();

        let swapchain = api::swapchain::SwapChain::new(
            Arc::clone(&device),
            Arc::clone(&surface),
            Arc::clone(&window),
        );

        let (swapchain, images) = (swapchain.swapchain.clone(), swapchain.images.clone());

        let allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(), StandardCommandBufferAllocatorCreateInfo::default()));

        let render_pass = create_render_pass(device.clone(), Format::R8G8B8A8_UNORM);

        let framebuffers: Vec<Arc<Framebuffer>> = VulkanHelper::create_frame_buffers(
            images.clone(),
            render_pass.clone(),
        );

        let memory_allocator =
            Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        Vulkan {
            context: Arc::new(Mutex::new(VulkanContext {
                device,
                queue,
                surface,
                swapchain,
                images,
                render_pass,
                framebuffers,
                command_buffers: vec![],
                memory_allocator,
                cmd_bf_allocator: allocator,
            })),
            window_resized: false,
            recreate_swapchain: false,
        }
    }

    pub fn submit(&mut self) {
        let swapchain;
        let queue;
        let command_buffers;
        let device;
        {
            let context = self.context.lock().unwrap();
            swapchain = context.swapchain.clone();
            queue = context.queue.clone();
            command_buffers = context.command_buffers.clone();
            device = context.device.clone();
        }

        let (image_i, suboptimal, acquire_future) =
            match acquire_next_image(swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(result) => result,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("获取下一张图像失败: {}",e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
            return;
        }

        let execution = sync::now(device.clone())
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffers[image_i as usize].clone())
            .unwrap()
            .then_swapchain_present(
                queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i),
            )
            .then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => {
                future.wait(None).unwrap();
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                error!("failed to flush future: {e}")
            }
        }
    }

    pub fn recreate_swapchain(&mut self, window: Arc<Window>, renderer: &mut Renderer, layer_stack: &mut LayerStack) {
        if self.window_resized || self.recreate_swapchain {

            let old_swapchain;
            {
                let context = self.context.lock().unwrap();
                old_swapchain = context.swapchain.clone();
            }

            self.recreate_swapchain = false;

            let new_dimensions = window.clone().inner_size();

            let (new_swapchain, new_images) = old_swapchain
                .recreate(SwapchainCreateInfo{
                    image_extent: new_dimensions.into(),
                    ..old_swapchain.create_info()
                })
                .expect("重建交换链失败！");

            let render_pass;
            {
                let mut context = self.context.lock().unwrap();
                context.swapchain = new_swapchain;
                context.images = new_images.clone();
                render_pass = context.render_pass.clone();
            }

            let framebuffers = VulkanHelper::create_frame_buffers(new_images, render_pass);

            {
                let mut context = self.context.lock().unwrap();
                context.framebuffers = framebuffers.clone();
            }

            if self.window_resized {
                self.window_resized = false;

                renderer.recreate_pipeline();

                let command_buffers = get_command_buffers(
                    renderer,
                    layer_stack,
                    framebuffers,
                );

                let mut context = self.context.lock().unwrap();
                context.command_buffers = command_buffers;
            }
        }
    }
}

/// 判断某个物理设备是否符合需求并返回队列索引
///
/// @param physical_device 从枚举获得的物理设备
///
/// @param required_flag 所需设备类型的标识
///
/// @param required_flag 所需设备类型的标识
///
/// @return 设备队列索引（Option包裹）
///
fn get_required_queue_family_index(physical_device: &PhysicalDevice, required_flag: QueueFlags) -> Option<u32> {
    let properties = physical_device.queue_family_properties();
    for (i, properties) in properties.iter().enumerate() {
        if properties.queue_flags.contains(required_flag) {
            return Some(i as u32);
        }
    }
    None
}


/// 创建一个RenderPass（Arc包裹）
///
/// @param device 可用设备
///
/// @param format 格式
///
/// @return RenderPass（Arc包裹）
///
fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<RenderPass> {
    let render_pass = single_pass_renderpass!(
        device.clone(),
        attachments: {
            foo: {
                format: format,
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [foo],
            depth_stencil: {},
        }
    )
        .unwrap_or_else(|err| panic!("创建渲染令牌: {}", err));
    render_pass
}

/// 创建一组CommandBuffer（Arc包裹）
///
/// @param allocator 命令缓冲区分配器
///
/// @param queue 设备队列
///
/// @param pipeline 可选的图像管线，若为None即是清屏
///
/// @param framebuffers
///
/// @return 一组命令缓冲区（Arc包裹）
///
fn get_command_buffers(
    renderer: &mut Renderer,
    layer_stack: &mut LayerStack,
    framebuffers: Vec<Arc<Framebuffer>>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    let mut command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>> = Vec::new();

    framebuffers.into_iter().for_each(|framebuffer| {
        renderer.begin(
            framebuffer.clone(),
            [0.1,0.1,0.1,1.0]
        );

        layer_stack.iter_mut().for_each(|layer| {
            layer.on_render(renderer);
        });

        renderer.end();

        let command_buffer = renderer.submit();
        command_buffers.push(command_buffer);
    });

    command_buffers
}