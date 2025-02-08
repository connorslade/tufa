use crate::bindings::BindableResourceId;

pub mod compute;
pub mod render;

pub(crate) struct PipelineStatus {
    pub resources: Vec<BindableResourceId>,
    pub dirty: bool,
}
