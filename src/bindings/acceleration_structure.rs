//! Used for accelerating ray-triangle intersection tests for ray tracing.
//!
//! Just see [@connorslade/ray-tracing](https://github.com/connorslade/ray-tracing/blob/5b50604c880f0dda8721d2b613221b3a0f9670c8/src/scene.rs#L50) until I get around to documenting this module...

use std::iter;

use encase::{internal::WriteInto, ShaderSize, ShaderType};
use nalgebra::{Matrix4, Matrix4x3};
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BindingType, Blas, BlasBuildEntry, BlasGeometries,
    BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor,
    CreateBlasDescriptor, CreateTlasDescriptor, IndexFormat, TlasInstance, TlasPackage,
    VertexFormat,
};

use crate::{
    bindings::{Bindable, BindableResourceId},
    gpu::Gpu,
    misc::ids::AccelerationStructureId,
};

use super::buffer::BlasBuffer;

pub struct AccelerationStructure<Vertex> {
    gpu: Gpu,
    id: AccelerationStructureId,

    blas: Vec<(Blas, Vec<BlasTriangleGeometrySizeDescriptor>)>,
    geometry: Vec<Geometry>,

    vertices: BlasBuffer<Vertex>,
    indices: BlasBuffer<u32>,
    transformation: BlasBuffer<Matrix4x3<f32>>,
}

pub struct Geometry {
    pub transformation: Matrix4<f32>,
    pub primitives: Vec<GeometryPrimitive>,
}

pub struct GeometryPrimitive {
    pub first_vertex: u32,
    pub vertex_count: u32,

    pub first_index: u32,
    pub index_count: u32,

    pub transformation_offset: u64,
}

impl<Vertex> AccelerationStructure<Vertex>
where
    Vertex: ShaderType + ShaderSize + WriteInto,
{
    pub fn update(&self) {
        let vertex_buffer = self.vertices.get();
        let index_buffer = self.indices.get();
        let transformation_buffer = self.transformation.get();

        let entries = self
            .blas
            .iter()
            .zip(self.geometry.iter())
            .map(|((blas, size), geometry)| {
                let geometries = geometry
                    .primitives
                    .iter()
                    .zip(size.iter())
                    .map(|(primitive, size)| BlasTriangleGeometry {
                        size,
                        vertex_buffer: &vertex_buffer,
                        first_vertex: primitive.first_vertex,
                        vertex_stride: Vertex::SHADER_SIZE.get(),
                        index_buffer: Some(&index_buffer),
                        first_index: Some(primitive.first_index),
                        transform_buffer: Some(&transformation_buffer),
                        transform_buffer_offset: Some(primitive.transformation_offset * 48),
                    })
                    .collect::<Vec<_>>();

                BlasBuildEntry {
                    blas,
                    geometry: BlasGeometries::TriangleGeometries(geometries),
                }
            })
            .collect::<Vec<_>>();

        let binding_manager = &self.gpu.binding_manager;
        binding_manager.mark_resource_dirty(&BindableResourceId::AccelerationStructure(self.id));

        let resource = binding_manager.get_resource(self.id);
        let package = resource.expect_tlas_package();

        self.gpu.immediate_dispatch(|encoder| {
            encoder.build_acceleration_structures(entries.iter(), iter::once(package));
        });
    }
}

impl Gpu {
    /// Make sure you enabled raytracing when initializing the Gpu
    pub fn create_acceleration_structure<Vertex>(
        &self,
        vertices: BlasBuffer<Vertex>,
        indices: BlasBuffer<u32>,
        transformation: BlasBuffer<Matrix4x3<f32>>,
        geometry: Vec<Geometry>,
    ) -> AccelerationStructure<Vertex>
    where
        Vertex: ShaderType + ShaderSize + WriteInto,
    {
        let tlas = self.device.create_tlas(&CreateTlasDescriptor {
            label: None,
            max_instances: geometry.len() as u32,
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::PreferUpdate,
        });

        let mut package = TlasPackage::new(tlas);

        let blas = geometry
            .iter()
            .enumerate()
            .map(|(i, geometry)| {
                let size = geometry
                    .primitives
                    .iter()
                    .map(|primitive| BlasTriangleGeometrySizeDescriptor {
                        vertex_format: VertexFormat::Float32x3,
                        vertex_count: primitive.vertex_count,
                        index_format: Some(IndexFormat::Uint32),
                        index_count: Some(primitive.index_count),
                        flags: AccelerationStructureGeometryFlags::OPAQUE,
                    })
                    .collect::<Vec<_>>();

                let blas = self.device.create_blas(
                    &CreateBlasDescriptor {
                        label: None,
                        flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                        update_mode: AccelerationStructureUpdateMode::PreferUpdate,
                    },
                    BlasGeometrySizeDescriptors::Triangles {
                        descriptors: size.clone(),
                    },
                );
                package[i] = Some(TlasInstance::new(
                    &blas,
                    geometry.transformation.as_slice()[..12].try_into().unwrap(),
                    i as u32,
                    0xff,
                ));

                (blas, size)
            })
            .collect::<Vec<_>>();

        let id = AccelerationStructureId::new();
        self.binding_manager.add_resource(id, package);

        let this = AccelerationStructure {
            gpu: self.clone(),
            id,

            blas,
            geometry,

            vertices,
            indices,
            transformation,
        };

        this.update();
        this
    }
}

impl<Vertex> Bindable for AccelerationStructure<Vertex> {
    fn resource_id(&self) -> BindableResourceId {
        BindableResourceId::AccelerationStructure(self.id)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::AccelerationStructure
    }
}

impl<Vertex> Drop for AccelerationStructure<Vertex> {
    fn drop(&mut self) {
        self.gpu.binding_manager.remove_resource(self.id);
    }
}
