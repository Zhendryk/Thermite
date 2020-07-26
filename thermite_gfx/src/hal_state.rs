use crate::shader::Shader;
use backend;
use gfx_hal::{
    self,
    device::Device,
    queue::family::QueueFamily,
    window::{PresentationSurface, Surface},
    Instance,
};
use raw_window_handle::HasRawWindowHandle;
use std::mem::ManuallyDrop;
use thermite_core::resources;

pub struct HALResources<B: gfx_hal::Backend> {
    pub instance: B::Instance,
    pub surface: B::Surface,
    pub logical_device: B::Device,
    pub render_passes: Vec<B::RenderPass>,
    pub pipeline_layouts: Vec<B::PipelineLayout>,
    pub pipelines: Vec<B::GraphicsPipeline>,
    pub command_pool: B::CommandPool,
    pub command_buffer: B::CommandBuffer,
    pub submission_complete_fence: B::Fence,
    pub rendering_complete_semaphore: B::Semaphore,
}

pub struct HALState {
    pub resources: ManuallyDrop<HALResources<backend::Backend>>,
}

impl HALState {
    pub fn new(window: &impl HasRawWindowHandle) -> Result<Self, &'static str> {
        let (instance, surface, adapter) = {
            let instance =
                backend::Instance::create("Thermite GFX", 1).expect("Backend not supported");
            let surface = unsafe {
                instance
                    .create_surface(window)
                    .expect("Failed to create surface for window")
            };
            let adapter = instance
                .enumerate_adapters()
                .into_iter()
                .find(|a| {
                    a.queue_families.iter().any(|qf| {
                        qf.queue_type().supports_graphics() && surface.supports_queue_family(qf)
                    })
                })
                .ok_or("Couldn't find a graphical adapter!")?;
            (instance, surface, adapter)
        };
        let (logical_device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|family| {
                    surface.supports_queue_family(family) && family.queue_type().supports_graphics()
                })
                .expect("No compatible queue family found");
            let mut gpu = unsafe {
                use gfx_hal::adapter::PhysicalDevice;
                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
                    .expect("Failed to open physical device")
            };
            (
                gpu.device,
                gpu.queue_groups
                    .pop()
                    .expect("Couldn't get queue group from gpu"),
            )
        };
        let (command_pool, command_buffer) = {
            use gfx_hal::command::Level;
            use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
            let mut command_pool = unsafe {
                logical_device
                    .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                    .expect("Out of memory")
            };
            let command_buffer = unsafe { command_pool.allocate_one(Level::Primary) };
            (command_pool, command_buffer)
        };
        let surface_color_format = {
            use gfx_hal::format::{ChannelType, Format};
            let supported_formats = surface
                .supported_formats(&adapter.physical_device)
                .unwrap_or(vec![]);
            let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);
            supported_formats
                .into_iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .unwrap_or(default_format)
        };
        let render_pass = {
            use gfx_hal::image::Layout;
            use gfx_hal::pass::{
                Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc,
            };
            let color_attachment = Attachment {
                format: Some(surface_color_format),
                samples: 1,
                ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };
            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };
            unsafe {
                logical_device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .expect("Out of memory")
            }
        };
        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&[], &[])
                .expect("Out of memory")
        };
        let pipeline = unsafe {
            make_pipeline::<backend::Backend>(
                &logical_device,
                &render_pass,
                &pipeline_layout,
                "p1.vert",
                "p1.frag",
            )
        };
        let submission_complete_fence = logical_device.create_fence(true).expect("Out of memory");
        let rendering_complete_semaphore =
            logical_device.create_semaphore().expect("Out of memory");
        let hal_state = HALState {
            resources: ManuallyDrop::new(HALResources::<backend::Backend> {
                instance: instance,
                surface: surface,
                logical_device: logical_device,
                render_passes: vec![render_pass],
                pipeline_layouts: vec![pipeline_layout],
                pipelines: vec![pipeline],
                command_pool: command_pool,
                command_buffer: command_buffer,
                submission_complete_fence: submission_complete_fence,
                rendering_complete_semaphore: rendering_complete_semaphore,
            }),
        };
        Ok(hal_state)
    }
}

unsafe fn make_pipeline<B: gfx_hal::Backend>(
    logical_device: &B::Device,
    render_pass: &B::RenderPass,
    pipeline_layout: &B::PipelineLayout,
    vertex_shader: &str,
    fragment_shader: &str,
) -> B::GraphicsPipeline {
    use gfx_hal::pass::Subpass;
    use gfx_hal::pso::{
        BlendState, ColorBlendDesc, ColorMask, EntryPoint, Face, GraphicsPipelineDesc,
        GraphicsShaderSet, Primitive, Rasterizer, Specialization,
    };
    let shader_res = resources::Resource::new(std::path::Path::new("assets/shaders/spirv_out"))
        .expect("Couldn't open shader resource");
    let vs = Shader::new(&shader_res, vertex_shader).expect("Couldn't create vertex shader");
    let vertex_shader_module = logical_device
        .create_shader_module(&vs.data.expect("Couldn't get vertex shader data"))
        .expect("Couldn't load vertex shader module");
    let fs = Shader::new(&shader_res, fragment_shader).expect("Couldn't create fragment shader");
    let fragment_shader_module = logical_device
        .create_shader_module(&fs.data.expect("Couldn't get fragment shader data"))
        .expect("Couldn't load fragment shader module");
    let (vs_entry, fs_entry) = (
        EntryPoint {
            entry: "main",
            module: &vertex_shader_module,
            specialization: Specialization::default(),
        },
        EntryPoint {
            entry: "main",
            module: &fragment_shader_module,
            specialization: Specialization::default(),
        },
    );
    let shader_entries = GraphicsShaderSet {
        vertex: vs_entry,
        hull: None,
        domain: None,
        geometry: None,
        fragment: Some(fs_entry),
    };
    let mut pipeline_desc = GraphicsPipelineDesc::new(
        shader_entries,
        Primitive::TriangleList,
        Rasterizer {
            cull_face: Face::BACK,
            ..Rasterizer::FILL
        },
        pipeline_layout,
        Subpass {
            index: 0,
            main_pass: render_pass,
        },
    );
    pipeline_desc.blender.targets.push(ColorBlendDesc {
        mask: ColorMask::ALL,
        blend: Some(BlendState::ALPHA),
    });
    let pipeline = logical_device
        .create_graphics_pipeline(&pipeline_desc, None)
        .expect("Failed to create graphics pipeline!");
    logical_device.destroy_shader_module(vertex_shader_module);
    logical_device.destroy_shader_module(fragment_shader_module);

    pipeline
}

impl Drop for HALState {
    fn drop(&mut self) {
        unsafe {
            let HALResources {
                instance,
                mut surface,
                logical_device,
                command_pool,
                command_buffer,
                render_passes,
                pipeline_layouts,
                pipelines,
                submission_complete_fence,
                rendering_complete_semaphore,
            } = ManuallyDrop::take(&mut self.resources);
            logical_device.destroy_semaphore(rendering_complete_semaphore);
            logical_device.destroy_fence(submission_complete_fence);
            for pipeline in pipelines {
                logical_device.destroy_graphics_pipeline(pipeline);
            }
            for pipeline_layout in pipeline_layouts {
                logical_device.destroy_pipeline_layout(pipeline_layout);
            }
            for render_pass in render_passes {
                logical_device.destroy_render_pass(render_pass);
            }
            logical_device.destroy_command_pool(command_pool);
            surface.unconfigure_swapchain(&logical_device);
            instance.destroy_surface(surface);
        }
    }
}
