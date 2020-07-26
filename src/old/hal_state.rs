// use crate::thermite_core::resources;
// use crate::thermite_gfx::window::Window;
// use crate::thermite_gfx_backend;
// use core::mem::ManuallyDrop;
// use gfx_hal::{
//     self,
//     adapter::Adapter,
//     device::Device,
//     queue::QueueFamily,
//     window::{PresentationSurface, Surface},
//     Backend, Instance,
// };
// use log::info;

// // HALResources which must be manually deallocated in a specific order
// pub struct HALResources<B: Backend> {
//     _instance: <B as Backend>::Instance,
//     _surface: <B as Backend>::Surface,
//     _adapter: Adapter<B>,
//     logical_device: <B as Backend>::Device,
//     render_passes: Vec<<B as Backend>::RenderPass>,
//     pipeline_layouts: Vec<<B as Backend>::PipelineLayout>,
//     pipelines: Vec<<B as Backend>::GraphicsPipeline>,
//     command_pool: <B as Backend>::CommandPool,
//     submission_complete_fence: <B as Backend>::Fence,
//     rendering_complete_semaphore: <B as Backend>::Semaphore,
// }

// struct HALState<B: Backend> {
//     resources: ManuallyDrop<HALResources<B>>,
// }

// /* Cleanup code for HALResources. NOTE: We do not impl this on HALResources because our various
//    logical_device.destroy_* signatures take ownership (requires `self`), while `fn drop` takes `&mut self`.
//    We need an abstraction layer so we can move the resources from our &mut self into these methods via `ManuallyDrop.take`
// */
// impl<B: Backend> Drop for HALState<B> {
//     fn drop(&mut self) {
//         unsafe {
//             let HALResources {
//                 _instance,
//                 mut _surface,
//                 _adapter,
//                 logical_device,
//                 command_pool,
//                 render_passes,
//                 pipeline_layouts,
//                 pipelines,
//                 submission_complete_fence,
//                 rendering_complete_semaphore,
//             } = ManuallyDrop::take(&mut self.resources);

//             logical_device.destroy_semaphore(rendering_complete_semaphore);
//             logical_device.destroy_fence(submission_complete_fence);
//             for pipeline in pipelines {
//                 logical_device.destroy_graphics_pipeline(pipeline);
//             }
//             for pipeline_layout in pipeline_layouts {
//                 logical_device.destroy_pipeline_layout(pipeline_layout);
//             }
//             for render_pass in render_passes {
//                 logical_device.destroy_render_pass(render_pass);
//             }
//             logical_device.destroy_command_pool(command_pool);
//             _surface.unconfigure_swapchain(&logical_device);
//             _instance.destroy_surface(_surface);
//         }
//     }
// }

// impl<B: Backend> HALState<B> {
//     pub fn new(window: &Window) -> Result<Self, &'static str> {
//         let (instance, mut surface, adapter) = {
//             // Create our graphics backend instance handle
//             let instance = thermite_gfx_backend::Instance::create(window.title(), 1)
//                 .expect("Could not create thermite_gfx_backend instance!");
//             // Grab a surface (abstracted communication layer between gfx API and windowing service) using our window
//             let mut surface = unsafe {
//                 instance
//                     .create_surface(window.handle())
//                     .expect("Could not create surface for window!")
//             };
//             // Adapter which supports the gfx API we're using, most normally represents a hardware gpu
//             // We search for one which supports the graphics queue family type and our surface supports
//             let adapter = instance
//                 .enumerate_adapters()
//                 .into_iter()
//                 .find(|a| {
//                     a.queue_families.iter().any(|qf| {
//                         qf.queue_type().supports_graphics() && surface.supports_queue_family(qf)
//                     })
//                 })
//                 .ok_or("Couldn't find a graphical adapter!")?;
//             (instance, surface, adapter)
//         };
//         let (logical_device, queue_group) = {
//             // Need a family of command queues which supports graphics and is supported by our window surface
//             let queue_family = adapter
//                 .queue_families
//                 .iter()
//                 .find(|qf| surface.supports_queue_family(qf) && qf.queue_type().supports_graphics())
//                 .ok_or("No compatible queue family found!")?;
//             let mut gpu = unsafe {
//                 use gfx_hal::adapter::PhysicalDevice;
//                 // Open/connect to the physical gpu device, which (hopefully) gives us back a handle to the gpu
//                 adapter
//                     .physical_device
//                     .open(&[(&queue_family, &[1.0])], gfx_hal::Features::empty()) // TODO: Check out additional features
//                     .map_err(|_| "Couldn't open Physical Device!")?
//             };
//             (
//                 gpu.device, // Logical Device (interface to the physical gpu)
//                 gpu.queue_groups // Command queues provided by the device
//                     .pop()
//                     .ok_or("Couldn't pop queue from queue group")?,
//             )
//         };
//         let surface_color_format = {
//             use gfx_hal::format::{ChannelType, Format};
//             // Grab all of the surface formats our window surface supports, or an empty vec
//             let supported_formats = surface
//                 .supported_formats(&adapter.physical_device)
//                 .unwrap_or(vec![]);
//             // Set our default to the first supported format, or if none are supported, we choose SRGB so we get gamma correction for free
//             let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);
//             // Find a supported format which supports SRGB, or revert to our default
//             supported_formats
//                 .into_iter()
//                 .find(|format| format.base_format().1 == ChannelType::Srgb)
//                 .unwrap_or(default_format)
//         };
//         let (command_pool, mut command_buffer) = unsafe {
//             use gfx_hal::command::Level;
//             use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
//             let mut command_pool = logical_device
//                 .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
//                 .expect("Out of memory");
//             let command_buffer = command_pool.allocate_one(Level::Primary);
//             (command_pool, command_buffer)
//         };
//         let renderpass = {
//             use gfx_hal::image::Layout;
//             use gfx_hal::pass::{
//                 Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc,
//             };
//             let color_attachment = Attachment {
//                 format: Some(surface_color_format),
//                 samples: 1,
//                 ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
//                 stencil_ops: AttachmentOps::DONT_CARE,
//                 layouts: Layout::Undefined..Layout::Present,
//             };
//             let subpass = SubpassDesc {
//                 colors: &[(0, Layout::ColorAttachmentOptimal)],
//                 depth_stencil: None,
//                 inputs: &[],
//                 resolves: &[],
//                 preserves: &[],
//             };
//             unsafe {
//                 logical_device
//                     .create_render_pass(&[color_attachment], &[subpass], &[])
//                     .expect("Out of memory")
//             }
//         };
//         // TODO: Look into seeing if we need this or not
//         let caps = surface.capabilities(&adapter.physical_device);
//         info!("{:?}", caps);
//         info!("Present Modes: {:?}", caps.present_modes);
//         info!("Composite Alphas: {:?}", caps.composite_alpha_modes);
//         let swapchain_config = gfx_hal::window::SwapchainConfig::from_caps(
//             &caps,
//             surface_color_format,
//             *caps.extents.end(),
//         );
//         let (swapchain, back_buffer) = unsafe {
//             logical_device
//                 .create_swapchain(&mut surface, swapchain_config, None)
//                 .map_err(|_| "Failed to create swapchain!")?
//         };
//         let pipeline_layout = unsafe {
//             logical_device
//                 .create_pipeline_layout(&[], &[])
//                 .expect("Out of memory")
//         };
//         let pipeline = unsafe {
//             create_pipeline::<thermite_gfx_backend::Backend>(
//                 &logical_device,
//                 &renderpass,
//                 &pipeline_layout,
//                 "vulkan.vert.spv",
//                 "vulkan.frag.spv",
//             )
//         };
//         let submission_complete_fence = logical_device.create_fence(true).expect("Out of memory");
//         let rendering_complete_semaphore =
//             logical_device.create_semaphore().expect("Out of memory");
//         // let hal_state = HALState::<thermite_gfx_backend::Backend> {
//         //     resources: ManuallyDrop::new(HALResources::<thermite_gfx_backend::Backend> {
//         //         _instance: instance,
//         //         _surface: surface,
//         //         _adapter: adapter,
//         //         logical_device: logical_device,
//         //         render_passes: vec![renderpass],
//         //         pipeline_layouts: vec![pipeline_layout],
//         //         pipelines: vec![pipeline],
//         //         command_pool: command_pool,
//         //         submission_complete_fence: submission_complete_fence,
//         //         rendering_complete_semaphore: rendering_complete_semaphore,
//         //     }),
//         // };
//         let hs = Self {
//             resources: ManuallyDrop::new( HALResources {
//                 _instance: instance,
//                 _surface: surface,
//                 _adapter: adapter,
//                 logical_device: logical_device,
//                 render_passes: vec![renderpass],
//                 pipeline_layouts: vec![pipeline_layout],
//                 pipelines: vec![pipeline],
//                 command_pool: command_pool,
//                 submission_complete_fence: submission_complete_fence,
//                 rendering_complete_semaphore: rendering_complete_semaphore,
//             })
//         }
//     }
// }

// unsafe fn create_pipeline<B: gfx_hal::Backend>(
//     device: &B::Device,
//     renderpass: &B::RenderPass,
//     pipeline_layout: &B::PipelineLayout,
//     vertex_shader: &str,
//     fragment_shader: &str,
// ) -> B::GraphicsPipeline {
//     use crate::thermite_gfx::shader;
//     use gfx_hal::pass::Subpass;
//     use gfx_hal::pso::{
//         BlendState, ColorBlendDesc, ColorMask, EntryPoint, Face, GraphicsPipelineDesc,
//         GraphicsShaderSet, Primitive, Rasterizer, Specialization,
//     };
//     let spirv_resource = resources::Resource::new(std::path::Path::new("assets/shaders/spirv_out"))
//         .expect("Failed to open spirv shader resource");
//     let vs = shader::Shader::new(&spirv_resource, vertex_shader).expect("");
//     let fs = shader::Shader::new(&spirv_resource, fragment_shader).expect("");
//     let vertex_shader_module = device
//         .create_shader_module(&vs.data.expect("Couldn't get vertex shader data"))
//         .expect("Couldn't load vertex shader module");
//     let fragment_shader_module = device
//         .create_shader_module(&fs.data.expect("Couldn't get fragment shader data"))
//         .expect("Couldn't load fragment shader module");
//     let (vs_entry, fs_entry) = (
//         EntryPoint {
//             entry: "main",
//             module: &vertex_shader_module,
//             specialization: Specialization::default(),
//         },
//         EntryPoint {
//             entry: "main",
//             module: &fragment_shader_module,
//             specialization: Specialization::default(),
//         },
//     );
//     let shader_entries = GraphicsShaderSet {
//         vertex: vs_entry,
//         hull: None,
//         domain: None,
//         geometry: None,
//         fragment: Some(fs_entry),
//     };
//     let mut pipeline_desc = GraphicsPipelineDesc::new(
//         shader_entries,
//         Primitive::TriangleList,
//         Rasterizer {
//             cull_face: Face::BACK,
//             ..Rasterizer::FILL
//         },
//         pipeline_layout,
//         Subpass {
//             index: 0,
//             main_pass: renderpass,
//         },
//     );
//     pipeline_desc.blender.targets.push(ColorBlendDesc {
//         mask: ColorMask::ALL,
//         blend: Some(BlendState::ALPHA),
//     });
//     let pipeline = device
//         .create_graphics_pipeline(&pipeline_desc, None)
//         .expect("Failed to create graphics pipeline!");
//     device.destroy_shader_module(vertex_shader_module);
//     device.destroy_shader_module(fragment_shader_module);
//     pipeline
// }
