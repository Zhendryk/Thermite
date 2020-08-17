use crate::hal::types::HALError;
use gfx_hal::{
    adapter::{Adapter, PhysicalDevice},
    device::Device,
    pool::CommandPoolCreateFlags,
    queue::{QueueFamily, QueueGroup},
    window::Surface,
    Backend, Instance,
};

// Represents a single "GPU" resource, which has a single logical handle managing one or more physical GPU devices
pub struct GPU<B: Backend> {
    handle: B::Device,
    physical_adapters: Vec<Adapter<B>>,
    command_queue_group: QueueGroup<B>,
}

impl<B: Backend> GPU<B> {
    pub fn new(instance: &B::Instance, surface: &B::Surface) -> Result<Self, HALError> {
        // TODO: Pass in bitfield for desired queue family support? Default to supporting graphics only
        // ?NOTE: Can create single logical device from multiple physical devices if those pds belong to the same device group
        // ?NOTE: Must be at least 1:1 logical->physical for each unique physical device (except in the above case?)
        let physical_adapter = instance
            .enumerate_adapters()
            .into_iter()
            .find(|adapter| {
                adapter.queue_families.iter().any(|family| {
                    family.queue_type().supports_graphics() && surface.supports_queue_family(family)
                })
            })
            .ok_or(HALError::AdapterError {
                message: String::from("Couldn't find a suitable graphical adapter!"),
                inner: None,
            })?;
        let (logical_handle, command_queue_group) = {
            // Find the queue family which our window surface supports and supports graphics
            let queue_family = physical_adapter
                .queue_families
                .iter()
                .find(|family| {
                    surface.supports_queue_family(family) && family.queue_type().supports_graphics()
                })
                .ok_or(HALError::AdapterError {
                    message: String::from("No compatible queue family found"),
                    inner: None,
                })?;
            // TODO: Look into additional features
            // "Open" our GPU using the queue families we've selected and with the provided features
            let mut gpu = unsafe {
                physical_adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
                    .map_err(|e| HALError::AdapterError {
                        message: String::from("Failed to open physical device"),
                        inner: Option::from(e),
                    })?
            };
            (
                gpu.device, // Logical device handle
                gpu.queue_groups.pop().ok_or(HALError::AdapterError {
                    message: String::from("Couldn't get queue group from gpu"),
                    inner: None,
                })?, // Just grab the first command queue group we can find // TODO: Elegantly handle this selection process
            )
        };
        Ok(GPU {
            handle: logical_handle,
            physical_adapters: vec![physical_adapter],
            command_queue_group: command_queue_group,
        })
    }

    pub fn logical(&self) -> &B::Device {
        &self.handle
    }

    pub fn adapter(&self) -> Result<&Adapter<B>, HALError> {
        match self.physical_adapters.get(0) {
            Some(pa_ref) => Ok(pa_ref),
            None => Err(HALError::GPUNotFound),
        }
    }
    pub fn adapter_at(&self, index: usize) -> Result<&Adapter<B>, HALError> {
        match self.physical_adapters.get(index) {
            Some(pa_ref) => Ok(pa_ref),
            None => Err(HALError::GPUNotFound),
        }
    }

    pub fn queue_group(&mut self) -> &mut QueueGroup<B> {
        &mut self.command_queue_group
    }

    pub unsafe fn create_command_pool(
        &mut self,
        create_flags: CommandPoolCreateFlags,
    ) -> Result<B::CommandPool, HALError> {
        Ok(self
            .handle
            .create_command_pool(self.command_queue_group.family, create_flags)?)
    }

    // TODO: Enumerate available feature(s)

    // TODO: Enable certain feature(s)

    // TODO: Enumerate available queue group(s)

    // TODO: Select specific queue group(s)
}

pub struct GPUPool<B: Backend> {
    gpus: std::collections::HashMap<String, GPU<B>>,
}

impl<B: Backend> GPUPool<B> {
    // TODO: Make this configurable with what kind of GPU we want
    pub fn add(
        &mut self,
        name: &str,
        instance: &B::Instance,
        surface: &B::Surface,
    ) -> Result<(), HALError> {
        if self.gpus.contains_key(name) {
            Err(HALError::CannotAddGPU)
        } else {
            self.gpus
                .insert(name.to_string(), GPU::new(instance, surface)?);
            Ok(())
        }
    }

    pub fn select(&mut self, name: &str) -> Result<GPU<B>, HALError> {
        if self.gpus.contains_key(name) {
            match self.gpus.remove(name) {
                Some(gpu) => Ok(gpu),
                None => Err(HALError::GPUNotFound),
            }
        } else {
            Err(HALError::GPUNotFound)
        }
    }
}

impl<B: Backend> Default for GPUPool<B> {
    fn default() -> Self {
        GPUPool {
            gpus: std::collections::HashMap::default(),
        }
    }
}
