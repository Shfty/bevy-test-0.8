use bevy::{
    prelude::{debug, World},
    render::{
        render_graph,
        render_resource::{ComputePassDescriptor, PipelineCache},
        renderer::RenderContext,
    },
};

use super::{BoardComputePipeline, BoardComputeQueue};

const WORKGROUP_SIZE: u32 = 64;

#[derive(Default)]
pub struct BoardComputeNode;

impl render_graph::Node for BoardComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<BoardComputePipeline>();
        debug!("Board compute node run");

        if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipelines.pipeline) {
            debug!("Pipeline valid");
            let bind_groups = &world.resource::<BoardComputeQueue>().0;
            for compute_job in bind_groups {
                let mut pass = render_context
                    .command_encoder
                    .begin_compute_pass(&ComputePassDescriptor::default());

                let instance_workgroups = (compute_job.cell_count / WORKGROUP_SIZE).max(1);

                debug!("Dispatching {instance_workgroups:?} board compute workgroups");
                pass.set_bind_group(0, &compute_job.bind_group_uniform, &[]);
                pass.set_bind_group(1, &compute_job.bind_group_storage, &[]);
                pass.set_pipeline(pipeline);
                pass.dispatch(instance_workgroups, 1, 1);
            }
        }

        Ok(())
    }
}
