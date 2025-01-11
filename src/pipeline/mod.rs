use crate::buffer::BindableResource;

pub mod compute;
pub mod render;

pub(crate) struct PipelineStatus {
    pub resources: Vec<BindableResource>,
    pub dirty: bool,
}
