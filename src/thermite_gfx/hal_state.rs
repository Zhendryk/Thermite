// use crate::thermite_gfx::window::Window;
// use crate::thermite_gfx_backend;

// pub struct HALState {
//     // Top level
//     instance: u32,
//     surface: u32,
//     adapter: u32,
//     queue_group: u32,
//     device: u32,
//     // GPU / Swapchain
//     swapchain: u32,
//     back_buffer: u32,
//     render_area: u32,
//     frames_in_flight: u32,
//     fences: u32,
//     semaphores: u32,
//     // Renderpass
//     render_pass: u32,
//     // Rendering targets
//     image_views: u32,
//     framebuffers: u32,
//     // Command issuing
//     command_pool: u32,
//     command_buffers: u32,
//     // Misc
//     current_frame: u32,
// }

// impl HALState {
//     pub fn new(window: &Window) -> Result<Self, &'static str> {
//         use gfx_hal::{device::Device, queue::QueueFamily, window::Surface, Instance};
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
//             // Adapter which supports the gfx API we're using, most normally a hardware gpu
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
//                     .open(&[(&queue_family, &[1.0])], gfx_hal::Features::empty())
//                     .map_err(|_| "Couldn't open Physical Device!")?
//             };
//             (
//                 gpu.device, // Logical Device (interface to the physical gpu)
//                 gpu.queue_groups // Command queues provided by the device
//                     .pop()
//                     .ok_or("Couldn't pop queue from queue group")?,
//             )
//         };
//         let (command_pool, mut command_buffer) = unsafe {
//             use gfx_hal::command::Level;
//             use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
//             // Command pools manage the memory that is used to store command buffers
//             let mut command_pool = logical_device
//                 .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
//                 .expect("Out of memory");
//             // Just allocating one primary command buffer in our command pool for now
//             let command_buffer = command_pool.allocate_one(Level::Primary);
//             (command_pool, command_buffer)
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
//         let renderpass = {
//             use gfx_hal::image::Layout;
//             use gfx_hal::pass::{
//                 Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc,
//             };
//             // Create a color attachment (basically a slot for an image to occupy)
//             let color_attachment = Attachment {
//                 format: Some(surface_color_format),
//                 samples: 1,
//                 ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
//                 stencil_ops: AttachmentOps::DONT_CARE,
//                 layouts: Layout::Undefined..Layout::Present,
//             };
//             // Subpasses define a subset of our defined attachments to use during the renderpass
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
//     }
// }
