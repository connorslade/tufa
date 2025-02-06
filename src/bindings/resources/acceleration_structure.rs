use std::iter;

use encase::{ShaderSize, ShaderType};
use nalgebra::{Matrix4, Vector2, Vector3};
use wgpu::{
    AccelerationStructureFlags, AccelerationStructureGeometryFlags,
    AccelerationStructureUpdateMode, BindingType, BlasBuildEntry, BlasGeometries,
    BlasGeometrySizeDescriptors, BlasTriangleGeometry, BlasTriangleGeometrySizeDescriptor,
    CreateBlasDescriptor, CreateTlasDescriptor, TlasInstance, TlasPackage, VertexFormat,
};

use crate::{
    bindings::{Bindable, BindableResource},
    gpu::Gpu,
    misc::ids::AccelerationStructureId,
};

use super::VertexBuffer;

pub struct AccelerationStructure {
    id: AccelerationStructureId,
}

pub struct Geometry {
    transformation: Matrix4<f32>,
    primitives: Vec<GeometryPrimitive>,
}

pub struct GeometryPrimitive {
    first_vertex: u32,
    vertex_count: u32,
}

#[derive(ShaderType)]
pub struct RayTracingVertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
    uv: Vector2<f32>,
}

impl Gpu {
    pub fn create_acceleration_structure(
        &self,
        vertices: VertexBuffer<RayTracingVertex>,
        geometry: &[Geometry],
    ) -> AccelerationStructure {
        let tlas = self.device.create_tlas(&CreateTlasDescriptor {
            label: None,
            max_instances: 0,
            flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: AccelerationStructureUpdateMode::Build,
        });

        let mut package = TlasPackage::new(tlas);

        let vertex_buffer = vertices.get();
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
                        index_format: None,
                        index_count: None,
                        flags: AccelerationStructureGeometryFlags::OPAQUE,
                    })
                    .collect::<Vec<_>>();

                let blas = self.device.create_blas(
                    &CreateBlasDescriptor {
                        label: None,
                        flags: AccelerationStructureFlags::PREFER_FAST_TRACE,
                        update_mode: AccelerationStructureUpdateMode::Build,
                    },
                    BlasGeometrySizeDescriptors::Triangles {
                        descriptors: size.clone(),
                    },
                );
                package[i] = Some(TlasInstance::new(
                    &blas,
                    geometry.transformation.as_slice().try_into().unwrap(),
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
                            vertex_stride: RayTracingVertex::SHADER_SIZE.get(),
                            index_buffer: None,
                            first_index: None,
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
