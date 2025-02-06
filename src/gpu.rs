use std::{collections::HashMap, mem, ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use parking_lot::{Mutex, RwLock};
use wgpu::{
    AdapterInfo, Buffer, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, MaintainBase,
    PowerPreference, Queue, RequestAdapterOptions,
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

    dispatch_queue: Mutex<DispatchQueue>,
    default_buffers: DefaultBuffers,
}

#[derive(Default)]
struct DispatchQueue {
    command_buffers: Vec<CommandBuffer>,
    callbacks: Vec<Box<dyn FnOnce() + Send>>,
}

impl Gpu {
    // todo: nicer way to change limits
    pub fn init() -> Result<Self> {
        let instance = Instance::new(&InstanceDescriptor::default());
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

                pipelines: RwLock::new(HashMap::new()),
                buffers: RwLock::new(HashMap::new()),

                dispatch_queue: Mutex::new(DispatchQueue::default()),
                default_buffers: DefaultBuffers::empty(),
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
}

impl Gpu {
    pub(crate) fn default_buffers(&self) -> &(VertexBuffer<Vertex>, IndexBuffer) {
        self.default_buffers.get(self)
    }

    pub(crate) fn mark_resource_dirty(&self, resource: &BindableResource) {
        let mut pipelines = self.pipelines.write();
        for (_id, PipelineStatus { resources, dirty }) in pipelines.iter_mut() {
            *dirty |= resources.contains(resource);
        }
    }
}

impl Gpu {
    pub fn flush_dispatch_queue(&self) {
        let queue = mem::take(&mut *self.dispatch_queue.lock());

        self.queue.submit(queue.command_buffers);

        for callback in queue.callbacks.into_iter() {
            self.queue.on_submitted_work_done(callback);
        }
    }

    pub(crate) fn queue_dispatch(&self, proc: impl FnOnce(&mut CommandEncoder)) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        proc(&mut encoder);

        let mut queue = self.dispatch_queue.lock();
        queue.command_buffers.push(encoder.finish());
    }

    pub(crate) fn queue_dispatch_callback(
        &self,
        proc: impl FnOnce(&mut CommandEncoder),
        callback: impl FnOnce() + Send + 'static,
    ) {
        self.queue_dispatch(proc);

        let mut queue = self.dispatch_queue.lock();
        queue.callbacks.push(Box::new(callback));
    }

    pub(crate) fn immediate_dispatch(&self, proc: impl FnOnce(&mut CommandEncoder)) {
        self.queue_dispatch(proc);
        self.flush_dispatch_queue();
    }

    pub(crate) fn immediate_dispatch_callback(
        &self,
        proc: impl FnOnce(&mut CommandEncoder),
        callback: impl FnOnce() + Send + 'static,
    ) {
        self.queue_dispatch(proc);
        let mut queue = self.dispatch_queue.lock();
        queue.callbacks.push(Box::new(callback));

        self.flush_dispatch_queue();
    }

    pub(crate) fn dispach(&self, proc: impl FnOnce(&mut CommandEncoder), immediate: bool) {
        if immediate {
            self.immediate_dispatch(proc);
        } else {
            self.queue_dispatch(proc);
        }
    }

    pub(crate) fn dispach_callback(
        &self,
        proc: impl FnOnce(&mut CommandEncoder),
        callback: impl FnOnce() + Send + 'static,
        immediate: bool,
    ) {
        if immediate {
            self.immediate_dispatch_callback(proc, callback);
        } else {
            self.queue_dispatch_callback(proc, callback);
        }
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
