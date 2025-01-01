use std::{iter, ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use wgpu::{
    AdapterInfo, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, Limits, MaintainBase, PowerPreference, Queue, RequestAdapterOptions,
};

#[derive(Clone)]
pub struct Gpu {
    inner: Arc<GpuInner>,
}

pub struct GpuInner {
    pub(crate) device: Device,
    pub(crate) queue: Queue,

    pub(crate) info: AdapterInfo,
}

impl Gpu {
    // todo: nicer way to change limits
    pub fn init() -> Result<Self> {
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .context("Error requesting adapter")?;
        let info = adapter.get_info();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                required_limits: Limits::default(),
                ..Default::default()
            },
            None,
        ))?;

        Ok(Self {
            inner: Arc::new(GpuInner {
                device,
                queue,
                info,
            }),
        })
    }

    /// Returns information on the selected adapter
    pub fn info(&self) -> &AdapterInfo {
        &self.info
    }

    /// Processes any resource cleanups and mapping callbacks
    pub fn poll(&self) {
        self.device.poll(MaintainBase::Poll);
    }

    /// Waits for all resource cleanups and mapping callbacks to complete
    pub fn wait(&self) {
        while !self.device.poll(MaintainBase::Wait).is_queue_empty() {}
    }

    pub(crate) fn dispatch(&self, proc: impl FnOnce(&mut CommandEncoder)) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        proc(&mut encoder);
        self.queue.submit(iter::once(encoder.finish()));
    }
}

impl Deref for Gpu {
    type Target = GpuInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        self.wait();
    }
}
