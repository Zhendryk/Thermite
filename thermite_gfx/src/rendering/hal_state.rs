use crate::rendering::mesh::Mesh;
use crate::shaders::shader::{PushConstants, Shader};
use backend::{
    self as ThermiteGfx, Backend as ThermiteBackend, Device as ThermiteDevice,
    Instance as ThermiteInstance,
};
use gfx_hal::{
    self,
    adapter::Adapter,
    device::{Device, OomOrDeviceLost, OutOfMemory},
    format::Format,
    pso::{Rect, ShaderStageFlags, Viewport},
    queue::{family::QueueFamily, QueueGroup},
    window::{
        AcquireError, CreationError, Extent2D, PresentationSurface, Surface, SwapchainConfig,
    },
    Backend, Instance,
};
use raw_window_handle::HasRawWindowHandle;
use std::mem::ManuallyDrop;
use thermite_core::resources;

// TODO: Simplify these horrendous <backend::Backend as Backend>::* types...
type ThermiteRenderPass = <ThermiteBackend as Backend>::RenderPass;
type ThermitePipelineLayout = <ThermiteBackend as Backend>::PipelineLayout;
type ThermiteGraphicsPipeline = <ThermiteBackend as Backend>::GraphicsPipeline;
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
    vertex_buffer_memory: B::Memory,
    vertex_buffer: B::Buffer,
    mesh: Mesh,
}

impl HALResources<ThermiteBackend> {
    pub fn recreate_swapchain(&mut self, extent: Extent2D) -> Result<Extent2D, CreationError> {
        let capabilities = self.surface.capabilities(&self.adapter.physical_device);
        let mut swapchain_config = SwapchainConfig::from_caps(&capabilities, self.format, extent);
        // This seems to fix some fullscreen slowdown on macOS.
        if capabilities.image_count.contains(&3) {
            swapchain_config.image_count = 3;
        }

        let extent = swapchain_config.extent;

        unsafe {
            self.surface
                .configure_swapchain(&self.logical_device, swapchain_config)?;
        };
        Ok(extent)
    }

    pub unsafe fn reset_command_pool(
        &mut self,
        render_timeout_ns: u64,
    ) -> Result<(), OomOrDeviceLost> {
        use gfx_hal::pool::CommandPool;
        self.logical_device
            .wait_for_fence(&self.submission_complete_fence, render_timeout_ns)?;
        self.logical_device
            .reset_fence(&self.submission_complete_fence)?;
        self.command_pool.reset(false);
        Ok(())
    }

    pub unsafe fn acquire_image(
        &mut self,
        acquire_timeout_ns: u64,
    ) -> Result<ThermiteSwapchainImage, AcquireError> {
        // Map the result tuple to just the swapchain image, because that's what we want
        self.surface.acquire_image(acquire_timeout_ns).map(|v| v.0)
    }

    pub unsafe fn create_framebuffer(
        &self,
        surface_image: &ThermiteSwapchainImage,
        surface_extent: Extent2D,
    ) -> Result<ThermiteFramebuffer, OutOfMemory> {
        use gfx_hal::image::Extent;
        use std::borrow::Borrow;
        let render_pass = &self.render_passes[0];
        self.logical_device.create_framebuffer(
            render_pass,
            vec![surface_image.borrow()],
            Extent {
                width: surface_extent.width,
                height: surface_extent.height,
                depth: 1,
            },
        )
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
        teapots: &[PushConstants],
    ) {
        use gfx_hal::command::{
            ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents,
        };
        self.command_buffer
            .begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
        self.command_buffer.set_viewports(0, &[viewport.clone()]);
        self.command_buffer.set_scissors(0, &[viewport.rect]);
        self.command_buffer.bind_vertex_buffers(
            0,
            vec![(&self.vertex_buffer, gfx_hal::buffer::SubRange::WHOLE)],
        );
        self.command_buffer.begin_render_pass(
            &self.render_passes[0],
            framebuffer,
            viewport.rect,
            &[ClearValue {
                color: ClearColor {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }],
            SubpassContents::Inline,
        );
        self.command_buffer
            .bind_graphics_pipeline(&self.pipelines[0]);
        for teapot in teapots {
            self.command_buffer.push_graphics_constants(
                &self.pipeline_layouts[0],
                ShaderStageFlags::VERTEX,
                0,
                push_constant_bytes(teapot),
            );
            self.command_buffer
                .draw(0..self.mesh.vertex_count as u32, 0..1);
        }
        self.command_buffer.end_render_pass();
        self.command_buffer.finish()
    }

    pub unsafe fn submit_cmds(&mut self, surface_image: ThermiteSwapchainImage) -> bool {
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
        result.is_err()
    }

    pub unsafe fn destroy_framebuffer(&mut self, framebuffer: ThermiteFramebuffer) {
        self.logical_device.destroy_framebuffer(framebuffer)
    }
}

/// Returns a view of a struct (normally `PushConstants`) as a slice of `u32`s
unsafe fn push_constant_bytes<T>(push_constants: &T) -> &[u32] {
    let size_in_bytes = std::mem::size_of::<T>();
    let size_in_u32s = size_in_bytes / std::mem::size_of::<u32>();
    let start_ptr = push_constants as *const T as *const u32;
    std::slice::from_raw_parts(start_ptr, size_in_u32s)
}

// TODO (HALState): Error handling &| propagation, doc comments, general cleanup, maybe some function separation
pub struct HALState {
    pub resources: ManuallyDrop<HALResources<ThermiteBackend>>,
}

#[derive(Debug)]
pub enum HALError {
    UnsupportedBackend,
    SurfaceCreationError {
        inner: gfx_hal::window::InitError,
    },
    AdapterError {
        message: String,
        inner: Option<gfx_hal::device::CreationError>,
    },
    OutOfMemory {
        inner: gfx_hal::device::OutOfMemory,
    },
    ShaderError {
        message: String,
    },
    PipelineError {
        inner: gfx_hal::pso::CreationError,
    },
}

impl HALState {
    pub fn new(window: &impl HasRawWindowHandle) -> Result<Self, HALError> {
        let (instance, surface, adapter) = {
            let instance = ThermiteInstance::create("Thermite GFX", 1)
                .map_err(|_| HALError::UnsupportedBackend)?;
            let surface = unsafe {
                instance
                    .create_surface(window)
                    .map_err(|e| HALError::SurfaceCreationError { inner: e })?
            };
            let adapter = instance
                .enumerate_adapters()
                .into_iter()
                .find(|a| {
                    a.queue_families.iter().any(|qf| {
                        qf.queue_type().supports_graphics() && surface.supports_queue_family(qf)
                    })
                })
                .ok_or(HALError::AdapterError {
                    message: String::from("Couldn't find a graphical adapter!"),
                    inner: None,
                })?;
            (instance, surface, adapter)
        };
        let (logical_device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|family| {
                    surface.supports_queue_family(family) && family.queue_type().supports_graphics()
                })
                .ok_or(HALError::AdapterError {
                    message: String::from("No compatible queue family found"),
                    inner: None,
                })?;
            let mut gpu = unsafe {
                use gfx_hal::adapter::PhysicalDevice;
                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
                    .map_err(|e| HALError::AdapterError {
                        message: String::from("Failed to open physical device"),
                        inner: Option::from(e),
                    })?
            };
            (
                gpu.device,
                gpu.queue_groups.pop().ok_or(HALError::AdapterError {
                    message: String::from("Couldn't get queue group from gpu"),
                    inner: None,
                })?,
            )
        };
        let (command_pool, command_buffer) = unsafe {
            use gfx_hal::command::Level;
            use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
            let mut command_pool = logical_device
                .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                .map_err(|e| HALError::OutOfMemory { inner: e })?;
            let command_buffer = command_pool.allocate_one(Level::Primary);
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
                    .map_err(|e| HALError::OutOfMemory { inner: e })?
            }
        };
        let push_constant_bytes = std::mem::size_of::<PushConstants>() as u32;
        let pipeline_layout = unsafe {
            logical_device
                .create_pipeline_layout(&[], &[(ShaderStageFlags::VERTEX, 0..push_constant_bytes)])
                .map_err(|e| HALError::OutOfMemory { inner: e })?
        };
        let pipeline = unsafe {
            make_pipeline::<ThermiteBackend>(
                &logical_device,
                &render_pass,
                &pipeline_layout,
                "test.vert.spv",
                "test.frag.spv",
            )?
        };
        let submission_complete_fence = logical_device
            .create_fence(true)
            .map_err(|e| HALError::OutOfMemory { inner: e })?;
        let rendering_complete_semaphore = logical_device
            .create_semaphore()
            .map_err(|e| HALError::OutOfMemory { inner: e })?;
        use crate::rendering::mesh::Mesh;
        let mesh_res = resources::Resource::new(std::path::Path::new("assets/meshes/"))
            .expect("Couldn't get mesh resource");
        let teapot_mesh = Mesh::new(&mesh_res, "teapot_mesh.bin");
        let (vertex_buffer_memory, vertex_buffer) =
            teapot_mesh.vertex_buffer::<ThermiteBackend>(&logical_device, &adapter.physical_device);
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
                vertex_buffer_memory,
                vertex_buffer,
                mesh: teapot_mesh,
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
                vertex_buffer_memory,
                vertex_buffer,
                mesh,
            } = ManuallyDrop::take(&mut self.resources);
            logical_device.free_memory(vertex_buffer_memory);
            logical_device.destroy_buffer(vertex_buffer);
            logical_device.destroy_semaphore(rendering_complete_semaphore);
            logical_device.destroy_fence(submission_complete_fence);
            for pipeline in pipelines {
                // TODO: See why this results in an Access Violation upon closing the window...
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
unsafe fn make_pipeline<ThermiteBackend>(
    logical_device: &ThermiteDevice,
    render_pass: &ThermiteRenderPass,
    pipeline_layout: &ThermitePipelineLayout,
    vertex_shader: &str,
    fragment_shader: &str,
) -> Result<ThermiteGraphicsPipeline, HALError> {
    use gfx_hal::pass::Subpass;
    use gfx_hal::pso::{
        BlendState, ColorBlendDesc, ColorMask, EntryPoint, Face, GraphicsPipelineDesc,
        GraphicsShaderSet, PolygonMode, Primitive, Rasterizer, ShaderStageFlags, Specialization,
    };
    let shader_res = resources::Resource::new(std::path::Path::new("assets/shaders/spirv"))
        .map_err(|_| HALError::ShaderError {
            message: String::from("Couldn't open shader resource"),
        })?;
    let vs = Shader::new(&shader_res, vertex_shader, ShaderStageFlags::VERTEX, "main").map_err(
        |_| HALError::ShaderError {
            message: String::from("Couldn't create vertex shader"),
        },
    )?;
    let vertex_shader_module =
        vs.module::<ThermiteGfx::Backend>(&logical_device)
            .map_err(|_| HALError::ShaderError {
                message: String::from("Couldn't load vertex shader module"),
            })?;
    let fs = Shader::new(
        &shader_res,
        fragment_shader,
        ShaderStageFlags::FRAGMENT,
        "main",
    )
    .map_err(|_| HALError::ShaderError {
        message: String::from("Couldn't create fragment shader"),
    })?;
    let fragment_shader_module =
        fs.module::<ThermiteGfx::Backend>(&logical_device)
            .map_err(|_| HALError::ShaderError {
                message: String::from("Couldn't load fragment shader module"),
            })?;
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
            // polygon_mode: PolygonMode::Line, // Uncomment this for wireframe polygons
            cull_face: Face::NONE,
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
    // Vertex buffer stuff
    use crate::rendering::mesh::Vertex;
    use gfx_hal::pso::{AttributeDesc, Element, VertexBufferDesc, VertexInputRate};
    pipeline_desc.vertex_buffers.push(VertexBufferDesc {
        binding: 0,
        stride: std::mem::size_of::<Vertex>() as u32,
        rate: VertexInputRate::Vertex,
    });
    pipeline_desc.attributes.push(AttributeDesc {
        location: 0,
        binding: 0,
        element: Element {
            format: Format::Rgb32Sfloat,
            offset: 0,
        },
    });
    pipeline_desc.attributes.push(AttributeDesc {
        location: 1,
        binding: 0,
        element: Element {
            format: Format::Rgb32Sfloat,
            offset: 12,
        },
    });
    let pipeline = logical_device
        .create_graphics_pipeline(&pipeline_desc, None)
        .map_err(|e| HALError::PipelineError { inner: e })?;
    logical_device.destroy_shader_module(vertex_shader_module);
    logical_device.destroy_shader_module(fragment_shader_module);

    Ok(pipeline)
}
