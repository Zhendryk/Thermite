use crate::hal::{gpu_pool::GPU, types::HALError};
use crate::primitives::buffer::VertexBuffer;
use crate::shaders::shader::PushConstants;
use gfx_hal::{
    command::{ClearValue, CommandBuffer, CommandBufferFlags, Level, SubpassContents},
    device::Device,
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::{ShaderStageFlags, Viewport},
    queue::{CommandQueue, Submission},
    window::PresentationSurface,
    Backend,
};
use std::borrow::Borrow;

pub struct CmdPool<B: Backend> {
    command_pool: B::CommandPool,
    command_buffers: Vec<B::CommandBuffer>,
}

impl<B: Backend> CmdPool<B> {
    pub fn new(gpu: &mut GPU<B>, create_flags: CommandPoolCreateFlags) -> Result<Self, HALError> {
        Ok(CmdPool {
            command_pool: unsafe { gpu.create_command_pool(create_flags)? },
            command_buffers: vec![],
        })
    }

    /// Allocates a single `CommandBuffer` of the given level (`Primary` or `Secondary`) for this `CmdPool`
    pub unsafe fn allocate_one_buffer(&mut self, level: Level) {
        self.command_buffers
            .push(self.command_pool.allocate_one(level));
    }

    pub unsafe fn destroy(self, device: &B::Device) {
        device.destroy_command_pool(self.command_pool);
    }

    /// Waits for the command pool to finish submission via fences, and resets it
    pub unsafe fn reset(
        &mut self,
        gpu: &GPU<B>,
        submission_complete_fence: &B::Fence,
        render_timeout_ns: u64,
    ) -> Result<(), HALError> {
        gpu.logical()
            .wait_for_fence(submission_complete_fence, render_timeout_ns)?;
        gpu.logical().reset_fence(submission_complete_fence)?;
        self.command_pool.reset(false);
        Ok(())
    }

    /// Records commands to be flushed from the command buffer to the GPU
    pub unsafe fn record<C>(
        &mut self,
        render_pass: &B::RenderPass,
        contains_subpasses: bool,
        framebuffer: &B::Framebuffer,
        viewport: &Viewport,
        pipeline: &B::GraphicsPipeline,
        pipeline_layout: &B::PipelineLayout,
        clear_values: C,
        vertex_buffers: &[VertexBuffer<B>],
        teapots: &[PushConstants],
    ) where
        C: IntoIterator,
        C::Item: Borrow<ClearValue>,
    {
        let primary_buffer = self.command_buffers.get_mut(0).expect("");
        primary_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);
        primary_buffer.set_viewports(0, &[viewport.clone()]);
        primary_buffer.set_scissors(0, &[viewport.rect]);
        let vb: Vec<(&B::Buffer, gfx_hal::buffer::SubRange)> = vertex_buffers
            .iter()
            .map(|buf| buf.subrange(gfx_hal::buffer::SubRange::WHOLE)) // TODO: Make this extensible externally
            .collect();
        primary_buffer.bind_vertex_buffers(0, vb);
        primary_buffer.begin_render_pass(
            render_pass,
            framebuffer,
            viewport.rect,
            clear_values,
            if contains_subpasses {
                SubpassContents::SecondaryBuffers
            } else {
                SubpassContents::Inline
            },
        );
        primary_buffer.bind_graphics_pipeline(pipeline);
        for (idx, teapot) in teapots.iter().enumerate() {
            primary_buffer.push_graphics_constants(
                pipeline_layout,
                ShaderStageFlags::VERTEX,
                0,
                push_constant_bytes(teapot),
            );
            primary_buffer.draw(0..vertex_buffers[idx].count as u32, 0..1);
        }
        primary_buffer.end_render_pass();
        primary_buffer.finish()
    }

    /// Submits all commands in the command buffers to the queue for execution
    pub unsafe fn submit(
        &self,
        gpu: &mut GPU<B>,
        submission_complete_fence: &B::Fence,
        rendering_complete_semaphore: &B::Semaphore,
    ) {
        let submission = Submission {
            command_buffers: &self.command_buffers,
            wait_semaphores: None,
            signal_semaphores: vec![rendering_complete_semaphore],
        };
        gpu.queue_group().queues[0].submit(submission, Some(submission_complete_fence));
    }

    // Presents the surface, and returns whether or not the operation was successful
    pub unsafe fn present(
        &self,
        gpu: &mut GPU<B>,
        surface: &mut B::Surface,
        surface_image: <B::Surface as PresentationSurface<B>>::SwapchainImage,
        rendering_complete_semaphore: &B::Semaphore,
    ) -> bool {
        let result = gpu.queue_group().queues[0].present_surface(
            surface,
            surface_image,
            Some(rendering_complete_semaphore),
        );
        result.is_err()
    }
}

/// Returns a view of a struct (normally `PushConstants`) as a slice of `u32`s
unsafe fn push_constant_bytes<T>(push_constants: &T) -> &[u32] {
    let size_in_bytes = std::mem::size_of::<T>();
    let size_in_u32s = size_in_bytes / std::mem::size_of::<u32>();
    let start_ptr = push_constants as *const T as *const u32;
    std::slice::from_raw_parts(start_ptr, size_in_u32s)
}
