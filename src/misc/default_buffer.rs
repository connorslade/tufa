use std::cell::OnceCell;

use crate::{
    buffer::{IndexBuffer, VertexBuffer},
    gpu::Gpu,
    pipeline::render::{
        consts::{QUAD_INDEX, QUAD_VERTEX},
        Vertex,
    },
};

pub(crate) struct DefaultBuffers {
    buffers: OnceCell<(VertexBuffer<Vertex>, IndexBuffer)>,
}

impl DefaultBuffers {
    pub fn empty() -> Self {
        Self {
            buffers: OnceCell::new(),
        }
    }

    pub fn get(&self, gpu: &Gpu) -> &(VertexBuffer<Vertex>, IndexBuffer) {
        self.buffers.get_or_init(|| {
            (
                gpu.create_vertex(QUAD_VERTEX).unwrap(),
                gpu.create_index(QUAD_INDEX).unwrap(),
            )
        })
    }
}
