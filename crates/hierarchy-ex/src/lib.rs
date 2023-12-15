use bevy::prelude::{Entity, Parent, Query};

pub fn walk_up<T>(
    from: Entity,
    query_parent: &Query<&Parent>,
    mut f: impl FnMut(Entity) -> Option<T>,
) -> Option<T> {
    if let Some(t) = f(from) {
        Some(t)
    } else if let Ok(parent) = query_parent.get(from) {
        walk_up(**parent, query_parent, f)
    } else {
        None
    }
}
