//! Main way of interacting with tufa.

use std::{mem, ops::Deref, sync::Arc};

use anyhow::{Context, Result};
use parking_lot::Mutex;
use wgpu::{
    AdapterInfo, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Features, Instance, InstanceDescriptor, Limits, MaintainBase, PowerPreference, Queue,
    RequestAdapterOptions,
};

use crate::{
    bindings::{
        buffer::{IndexBuffer, VertexBuffer},
        manager::BindingManager,
    },
    misc::default_buffer::DefaultBuffers,
    pipeline::render::Vertex,
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

    pub(crate) binding_manager: BindingManager,
    default_buffers: DefaultBuffers,
    dispatch_queue: Mutex<DispatchQueue>,
}

pub struct GpuBuilder {
    limits: Limits,
    features: Features,
    power_preference: PowerPreference,
}

#[derive(Default)]
struct DispatchQueue {
    command_buffers: Vec<CommandBuffer>,
    callbacks: Vec<Box<dyn FnOnce() + Send>>,
}

impl GpuBuilder {
    pub fn with_features(self, features: Features) -> Self {
        Self {
            features: self.features | features,
            ..self
        }
    }

    pub fn with_limits(self, limits: Limits) -> Self {
        Self { limits, ..self }
    }

    pub fn power_preference(self, power_preference: PowerPreference) -> Self {
        Self {
            power_preference,
            ..self
        }
    }

    pub fn with_raytracing(self) -> Self {
        self.with_features(
            Features::EXPERIMENTAL_RAY_TRACING_ACCELERATION_STRUCTURE
                | Features::EXPERIMENTAL_RAY_QUERY,
        )
    }

    pub fn build(self) -> Result<Gpu> {
        let instance = Instance::new(&InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: self.power_preference,
            ..Default::default()
        }))
        .context("Error requesting adapter")?;
        let info = adapter.get_info();

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            required_limits: self.limits,
            required_features: self.features,
            ..Default::default()
        }))?;

        Ok(Gpu {
            inner: Arc::new(GpuInner {
                #[cfg(feature = "interactive")]
                instance,
                device,
                queue,
                info,

                binding_manager: BindingManager::new(),
                default_buffers: DefaultBuffers::empty(),
                dispatch_queue: Mutex::new(DispatchQueue::default()),
            }),
        })
    }
}

impl Gpu {
    pub fn builder() -> GpuBuilder {
        GpuBuilder {
            limits: Limits::default(),
            features: Features::VERTEX_WRITABLE_STORAGE,
            power_preference: PowerPreference::None,
        }
    }

    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Returns information on the selected adapter
    pub fn info(&self) -> &AdapterInfo {
        &self.info
    }

    /// Processes any resource cleanups and mapping callbacks
    pub fn poll(&self) {
        let _ = self.device.poll(MaintainBase::Poll).unwrap();
    }

    /// Waits for all resource cleanups and mapping callbacks to complete
    pub fn wait(&self) {
        while !self
            .device
            .poll(MaintainBase::Wait)
            .unwrap()
            .is_queue_empty()
        {}
    }
}

impl Gpu {
    pub(crate) fn default_buffers(&self) -> &(VertexBuffer<Vertex>, IndexBuffer) {
        self.default_buffers.get(self)
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
        drop(queue);

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
