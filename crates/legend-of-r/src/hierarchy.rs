use bevy::{
    prelude::{default, Bundle, VisibilityBundle},
    transform::TransformBundle,
};

#[derive(Bundle)]
pub struct HierarchyBundle {
    #[bundle]
    pub transform: TransformBundle,
    #[bundle]
    pub visibility: VisibilityBundle,
}

impl Clone for HierarchyBundle {
    fn clone(&self) -> Self {
        Self {
            transform: self.transform.clone(),
            visibility: VisibilityBundle {
                visibility: self.visibility.visibility.clone(),
                computed: self.visibility.computed.clone(),
            },
        }
    }
}

impl Default for HierarchyBundle {
    fn default() -> Self {
        Self {
            transform: default(),
            visibility: default(),
        }
    }
}
