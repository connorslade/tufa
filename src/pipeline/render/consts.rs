use nalgebra::{Vector2, Vector4};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use super::Vertex;

pub const VERTEX_BUFFER_LAYOUT: VertexBufferLayout = VertexBufferLayout {
    array_stride: 32, // NOTE: WGSL alignment rules factor into this
    step_mode: VertexStepMode::Vertex,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 4 * 4,
            shader_location: 1,
        },
    ],
};

pub const QUAD_INDEX: &[u32] = &[0, 1, 2, 2, 3, 0];
pub const QUAD_VERTEX: &[Vertex] = &[
    Vertex::new(Vector4::new(-1.0, -1.0, 0.0, 1.0), Vector2::new(0.0, 0.0)),
    Vertex::new(Vector4::new(1.0, -1.0, 0.0, 1.0), Vector2::new(1.0, 0.0)),
    Vertex::new(Vector4::new(1.0, 1.0, 0.0, 1.0), Vector2::new(1.0, 1.0)),
    Vertex::new(Vector4::new(-1.0, 1.0, 0.0, 1.0), Vector2::new(0.0, 1.0)),
];
