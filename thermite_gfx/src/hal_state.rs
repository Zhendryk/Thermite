use crate::shader::Shader;
use backend;
use gfx_hal::{
    self,
    adapter::Adapter,
    device::Device,
    format::Format,
    pso::{Rect, Viewport},
    queue::{family::QueueFamily, QueueGroup},
    window::{Extent2D, PresentationSurface, Surface},
    Backend, Instance,
};
use raw_window_handle::HasRawWindowHandle;
use std::mem::ManuallyDrop;
use thermite_core::resources;

// TODO: Simplify these horrendous <backend::Backend as Backend>::* types...
type ThermiteBackend = backend::Backend;
type ThermiteInstance = backend::Instance;
type ThermiteSwapchainImage =
    <<ThermiteBackend as Backend>::Surface as PresentationSurface<ThermiteBackend>>::SwapchainImage;
type ThermiteFramebuffer = <ThermiteBackend as Backend>::Framebuffer;

// TODO (HALResources): Error handling &| propagation, doc comments, general cleanup
pub struct HALResources<B: Backend> {
    instance: B::Instance,
    surface: B::Surface,
    adapter: Adapter<B>,
    logical_device: B::Device,
    queue_group: QueueGroup<ThermiteBackend>,
    render_passes: Vec<B::RenderPass>,
    pipeline_layouts: Vec<B::PipelineLayout>,
    pipelines: Vec<B::GraphicsPipeline>,
    command_pool: B::CommandPool,
    command_buffer: B::CommandBuffer,
    format: Format,
    submission_complete_fence: B::Fence,
    rendering_complete_semaphore: B::Semaphore,
}

impl HALResources<ThermiteBackend> {
    pub fn recreate_swapchain(&mut self, extent: Extent2D) -> Extent2D {
        use gfx_hal::window::SwapchainConfig;
        let capabilities = self.surface.capabilities(&self.adapter.physical_device);
        let mut swapchain_config = SwapchainConfig::from_caps(&capabilities, self.format, extent);
        // This seems to fix some fullscreen slowdown on macOS.
        if capabilities.image_count.contains(&3) {
            swapchain_config.image_count = 3;
        }

        let extent = swapchain_config.extent;

        unsafe {
            self.surface
                .configure_swapchain(&self.logical_device, swapchain_config)
                .expect("Failed to configure swapchain");
        };
        extent
    }

    pub unsafe fn reset_command_pool(&mut self, render_timeout_ns: u64) {
        use gfx_hal::pool::CommandPool;
        self.logical_device
            .wait_for_fence(&self.submission_complete_fence, render_timeout_ns)
            .expect("Out of memory or device lost");
        self.logical_device
            .reset_fence(&self.submission_complete_fence)
            .expect("Out of memory");
        self.command_pool.reset(false);
    }

    pub unsafe fn acquire_image(
        &mut self,
        acquire_timeout_ns: u64,
    ) -> Result<ThermiteSwapchainImage, bool> {
        match self.surface.acquire_image(acquire_timeout_ns) {
            Ok((image, _)) => Ok(image),
            Err(_) => Err(true),
        }
    }

    pub unsafe fn create_framebuffer(
        &self,
        surface_image: &ThermiteSwapchainImage,
        surface_extent: Extent2D,
    ) -> Result<ThermiteFramebuffer, &'static str> {
        use gfx_hal::image::Extent;
        use std::borrow::Borrow;
        self.logical_device
            .create_framebuffer(
                &self.render_passes[0],
                vec![surface_image.borrow()],
                Extent {
                    width: surface_extent.width,
                    height: surface_extent.height,
                    depth: 1,
                },
            )
            .map_err(|_| "Out of memory")
    }

    pub fn viewport(&self, surface_extent: Extent2D) -> Viewport {
        Viewport {
            rect: Rect {
                x: 0,
                y: 0,
                w: surface_extent.width as i16,
                h: surface_extent.height as i16,
            },
            depth: 0.0..1.0,
        }
    }

    pub unsafe fn record_cmds_for_submission(
        &mut self,
        framebuffer: &ThermiteFramebuffer,
        viewport: &Viewport,
    ) {
        use gfx_hal::command::{
            ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents,
        };
        let render_pass = &self.render_passes[0];
        let pipeline = &self.pipelines[0];
        self.command_buffer
            .begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
        self.command_buffer.set_viewports(0, &[viewport.clone()]);
        self.command_buffer.set_scissors(0, &[viewport.rect]);
        self.command_buffer.begin_render_pass(
            render_pass,
            &framebuffer,
            viewport.rect,
            &[ClearValue {
                color: ClearColor {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }],
            SubpassContents::Inline,
        );
        self.command_buffer.bind_graphics_pipeline(pipeline);
        self.command_buffer.draw(0..3, 0..1);
        self.command_buffer.end_render_pass();
        self.command_buffer.finish()
    }

    pub unsafe fn submit_cmds(
        &mut self,
        surface_image: ThermiteSwapchainImage,
        framebuffer: ThermiteFramebuffer,
    ) -> bool {
        use gfx_hal::queue::{CommandQueue, Submission};
        let submission = Submission {
            command_buffers: vec![&self.command_buffer],
            wait_semaphores: None,
            signal_semaphores: vec![&self.rendering_complete_semaphore],
        };
        self.queue_group.queues[0].submit(submission, Some(&self.submission_complete_fence));
        let result = self.queue_group.queues[0].present_surface(
            &mut self.surface,
            surface_image,
            Some(&self.rendering_complete_semaphore),
        );
        self.logical_device.destroy_framebuffer(framebuffer);
        result.is_err()
    }
}

// TODO (HALState): Error handling &| propagation, doc comments, general cleanup, maybe some function separation
pub struct HALState {
    pub resources: ManuallyDrop<HALResources<ThermiteBackend>>,
}

impl HALState {
    pub fn new(window: &impl HasRawWindowHandle) -> Result<Self, &'static str> {
        let (instance, surface, adapter) = {
            let instance =
                ThermiteInstance::create("Thermite GFX", 1).expect("Backend not supported");
            let surface = unsafe {
                instance
                    .create_surface(window) // TODO: Check out why this gives UnsupportedWindowHandle for winit::window::Window?
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
            use gfx_hal::format::ChannelType;
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
            make_pipeline::<ThermiteBackend>(
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
            resources: ManuallyDrop::new(HALResources::<ThermiteBackend> {
                instance: instance,
                surface: surface,
                adapter: adapter,
                logical_device: logical_device,
                queue_group: queue_group,
                render_passes: vec![render_pass],
                pipeline_layouts: vec![pipeline_layout],
                pipelines: vec![pipeline],
                command_pool: command_pool,
                command_buffer: command_buffer,
                format: surface_color_format,
                submission_complete_fence: submission_complete_fence,
                rendering_complete_semaphore: rendering_complete_semaphore,
            }),
        };
        Ok(hal_state)
    }
}

// TODO: Ensure everything that needs to be dropped here is properly, and in the correct order
impl Drop for HALState {
    fn drop(&mut self) {
        unsafe {
            let HALResources {
                instance,
                mut surface,
                adapter: _,
                logical_device,
                queue_group: _,
                command_pool,
                command_buffer: _,
                format: _,
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

// TODO: Comments / docstrings
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
