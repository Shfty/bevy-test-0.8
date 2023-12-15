use bevy::prelude::Bundle;

use crate::prelude::{BoardMaterial, MeshInstanceColor};

use bevy_instancing::prelude::MeshInstanceBundle;

#[derive(Default, Bundle)]
pub struct BoardInstanceBundle {
    #[bundle]
    pub instance_bundle: MeshInstanceBundle<BoardMaterial>,
    pub mesh_instance_color: MeshInstanceColor,
}
