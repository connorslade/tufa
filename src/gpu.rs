use std::{collections::HashMap, iter, ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use parking_lot::RwLock;
use wgpu::{
    AdapterInfo, Buffer, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Features, Instance, InstanceDescriptor, Limits, MaintainBase, PowerPreference, Queue,
    RequestAdapterOptions,
};

use crate::{
    buffer::{BindableResource, IndexBuffer, VertexBuffer},
    misc::{
        default_buffer::DefaultBuffers,
        ids::{BufferId, PipelineId},
    },
    pipeline::{render::Vertex, PipelineStatus},
};

#[derive(Clone)]
pub struct Gpu {
    inner: Arc<GpuInner>,
}

pub struct GpuInner {
    #[cfg(feature = "interactive")]
    pub(crate) instance: Instance,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) info: AdapterInfo,

    pub(crate) pipelines: RwLock<HashMap<PipelineId, PipelineStatus>>,
    pub(crate) buffers: RwLock<HashMap<BufferId, Buffer>>,

    default_buffers: DefaultBuffers,
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
                required_features: Features::default() | Features::VERTEX_WRITABLE_STORAGE,
                ..Default::default()
            },
            None,
        ))?;

        Ok(Self {
            inner: Arc::new(GpuInner {
                #[cfg(feature = "interactive")]
                instance,
                device,
                queue,
                info,

                default_buffers: DefaultBuffers::empty(),
                pipelines: RwLock::new(HashMap::new()),
                buffers: RwLock::new(HashMap::new()),
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

    pub(crate) fn default_buffers(&self) -> &(VertexBuffer<Vertex>, IndexBuffer) {
        self.default_buffers.get(self)
    }

    pub(crate) fn mark_resource_dirty(&self, resource: &BindableResource) {
        let mut pipelines = self.pipelines.write();
        let pipeline = pipelines
            .iter_mut()
            .find(|(_, PipelineStatus { resources, .. })| resources.contains(resource))
            .unwrap();
        pipeline.1.dirty = true;
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
