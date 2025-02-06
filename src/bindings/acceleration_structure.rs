use std::iter;

use encase::{internal::WriteInto, ShaderSize, ShaderType};
use nalgebra::Matrix4;
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BindingType, BlasBuildEntry, BlasGeometries,
    BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor,
    CreateBlasDescriptor, CreateTlasDescriptor, IndexFormat, TlasInstance, TlasPackage,
    VertexFormat,
};

use crate::{
    bindings::{Bindable, BindableResource},
    gpu::Gpu,
    misc::ids::AccelerationStructureId,
};

use super::BlasBuffer;

pub struct AccelerationStructure {
    id: AccelerationStructureId,
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
}

impl Gpu {
    pub fn create_acceleration_structure<Vertex>(
        &self,
        vertices: BlasBuffer<Vertex>,
        indices: BlasBuffer<u32>,
        geometry: &[Geometry],
    ) -> AccelerationStructure
    where
        Vertex: ShaderType + ShaderSize + WriteInto,
    {
        let tlas = self.device.create_tlas(&CreateTlasDescriptor {
            label: None,
            max_instances: geometry.len() as u32,
            flags: AccelerationStructureFlags::PREFER_FAST_BUILD,
            update_mode: AccelerationStructureUpdateMode::Build,
        });

        let mut package = TlasPackage::new(tlas);

        let vertex_buffer = vertices.get();
        let index_buffer = indices.get();

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
                        flags: AccelerationStructureFlags::PREFER_FAST_BUILD,
                        update_mode: AccelerationStructureUpdateMode::Build,
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

        let entries = geometry
            .iter()
            .zip(blas.iter())
            .map(|(geometry, (blas, size))| {
                let geometries = geometry
                    .primitives
                    .iter()
                    .flat_map(|primitive| {
                        size.iter().map(|size| BlasTriangleGeometry {
                            size,
                            vertex_buffer: &vertex_buffer,
                            first_vertex: primitive.first_vertex,
                            vertex_stride: Vertex::SHADER_SIZE.get(),
                            index_buffer: Some(&index_buffer),
                            first_index: Some(primitive.first_index),
                            transform_buffer: None,
                            transform_buffer_offset: None,
                        })
                    })
                    .collect::<Vec<_>>();

                BlasBuildEntry {
                    blas: &blas,
                    geometry: BlasGeometries::TriangleGeometries(geometries),
                }
            })
            .collect::<Vec<_>>();

        self.immediate_dispatch(|endoder| {
            endoder.build_acceleration_structures(entries.iter(), iter::once(&package));
        });

        let id = AccelerationStructureId::new();
        self.binding_manager
            .add_acceleration_structures(id, package);

        AccelerationStructure { id }
    }
}

impl Bindable for AccelerationStructure {
    fn resource(&self) -> BindableResource {
        BindableResource::AccelerationStructure(self.id)
    }

    fn binding_type(&self) -> BindingType {
        BindingType::AccelerationStructure
    }
}
