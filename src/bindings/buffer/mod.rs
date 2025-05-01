use crate::misc::ids::BufferId;

pub mod blas;
pub mod index;
pub mod mutability;
pub mod storage;
pub mod uniform;
pub mod vertex;

pub trait BufferBinding {
    fn get_id(&self) -> BufferId;
}
