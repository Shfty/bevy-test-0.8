use std::collections::BTreeMap;

use bevy::{
    gltf::Gltf,
    prelude::{
        debug, default, error, info, AssetEvent, Assets, CoreStage, Entity, EventReader, Name,
        ParallelSystemDescriptorCoercion, Plugin, ReflectComponent, ReflectDefault, Res, ResMut,
        World,
    },
    reflect::{
        serde::ReflectDeserializer, DynamicList, DynamicMap, DynamicStruct, DynamicTuple,
        DynamicTupleStruct, Map as ReflectMap, Reflect, TypeInfo, TypeRegistry,
        TypeRegistryInternal,
    },
    scene::Scene,
};
use serde::de::DeserializeSeed;
use serde_json::{Map, Value};

use crate::{
    gltf_json::gltf_json_remove,
    integration::ReflectEnum,
    prelude::{
        gltf_json_insert, reflect_bundle::ReflectBundle, GltfExtrasJson, ReflectBoolean,
        ReflectOption,
    },
};

pub struct GltfEntityPlugin;

impl Plugin for GltfEntityPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_to_stage(
            CoreStage::Last,
            deserialize_gltf_components
                .after(gltf_json_insert)
                .before(gltf_json_remove),
        );
    }
}

fn deserialize_gltf_components(
    mut gltf_events: EventReader<AssetEvent<Gltf>>,
    gltf_assets: ResMut<Assets<Gltf>>,
    mut scene_assets: ResMut<Assets<Scene>>,
    type_registry: Res<TypeRegistry>,
) {
    // Iterate over changed handles
    for handle in gltf_events.iter().filter_map(|event| match event {
        AssetEvent::Created { handle } | AssetEvent::Modified { handle } => Some(handle),
        _ => None,
    }) {
        // Fetch the gltf asset if it is valid
        let gltf = if let Some(gltf) = gltf_assets.get(handle) {
            gltf
        } else {
            continue;
        };

        debug!("GLTF modified: {gltf:#?}");

        // Iterate over each scene in the gltf asset
        for handle in gltf.scenes.iter() {
            let scene = scene_assets.get_mut(handle).unwrap();
            debug!("Scene: {scene:#?}");
            deserialize_gltf_scene(&type_registry, scene);
        }
    }
}

fn deserialize_gltf_scene(type_registry: &TypeRegistry, scene: &mut Scene) {
    // Iterate over scene and collect JSON extra entity entries
    let mut query = scene.world.query::<(Entity, &mut GltfExtrasJson)>();
    let extra_entries = query
        .iter_mut(&mut scene.world)
        .flat_map(|(entity, mut extras)| match &mut extras.0 {
            Value::Object(object) => {
                if let Some(entity_object) = object.remove_entry("entity") {
                    Some((entity, entity_object))
                } else {
                    None
                }
            }
            _ => panic!("Extras entry is not an object"),
        })
        .collect::<BTreeMap<_, _>>();

    // Iterate over entity entries
    for (entity, (_, mut value)) in extra_entries {
        let object = if let Value::Object(ref mut object) = value {
            object
        } else {
            panic!("Entity entry is not an object");
        };

        deserialize_gltf_entity(entity, scene, type_registry, object);
    }
}

fn deserialize_gltf_entity(
    entity: Entity,
    scene: &mut Scene,
    type_registry: &TypeRegistry,
    object: &mut Map<String, Value>,
) {
    // Iterate over entity components
    for (_, component) in object.iter_mut() {
        deserialize_gltf_component(scene, entity, type_registry, component);
    }
}

fn deserialize_gltf_component(
    scene: &mut Scene,
    entity: Entity,
    type_registry: &TypeRegistry,
    component: &mut Value,
) {
    let type_name = component
        .get("type")
        .expect("Component {short_name:} has no type entry")
        .as_str()
        .expect("Component type entry is not a string")
        .to_string();

    info!("{type_name:}");

    // Perform fixups to ensure JSON data is compliant with serde format
    walk_json(component, &mut |value| {
        fixup_bool(type_registry, value);
        fixup_enum(type_registry, value);
        fixup_option(type_registry, value);
        fixup_tuple_types(type_registry, value);
        fixup_map(type_registry, value);
        fixup_entity(&mut scene.world, value);
    });

    info!("{:}", serde_json::to_string_pretty(component).unwrap());

    // Take a read lock on the type registry
    let type_registry = type_registry.read();

    let result = deserialize_gltf_field(&type_registry, component)
        .unwrap_or_else(|| panic!("Failed to desreialize component {type_name:}"));

    let registration = type_registry
        .get_with_name(&type_name)
        .unwrap_or_else(|| panic!("No type registration for {type_name:}"));

    if let Some(reflect_component) = registration.data::<ReflectComponent>() {
        debug!("Reflect component OK");
        reflect_component.apply_or_insert(&mut scene.world, entity, &*result);
        debug!("Insert component OK");
    } else if let Some(reflect_bundle) = registration.data::<ReflectBundle>() {
        debug!("Reflect bundle OK");
        reflect_bundle.insert(&mut scene.world, entity, &*result);
        debug!("Insert bundle OK");
    } else {
        error!("Unrecognized GLTF entity datatype {type_name:}")
    }
}

fn deserialize_gltf_field(
    type_registry: &TypeRegistryInternal,
    value: &mut Value,
) -> Option<Box<dyn Reflect>> {
    let type_name = value
        .get("type")
        .expect("Field {short_name:} has no type entry")
        .as_str()
        .expect("Field type entry is not a string")
        .to_string();

    info!("{type_name:}");

    let registration = type_registry
        .get_with_name(&type_name)
        .unwrap_or_else(|| panic!("No type registration for {type_name:}"));

    // Try to deserialize
    let deserializer = ReflectDeserializer::new(&type_registry);
    let result = deserializer.deserialize(value.clone());

    info!("Deserialize result: {result:#?}");

    match result {
        Ok(reflect) => {
            info!("Deserialized: {reflect:#?}");
            Some(reflect)
        }
        Err(e) => {
            info!("Deserialization failed: {e:#?}");

            let type_name = registration.type_name().to_string();

            let reflect_default =
                if let Some(reflect_default) = registration.data::<ReflectDefault>() {
                    reflect_default
                } else {
                    return None;
                };

            let mut default = reflect_default.default();

            let component = if let Value::Object(object) = value {
                object
            } else {
                panic!("Component is not an object");
            };

            match registration.type_info() {
                TypeInfo::Struct(_) => {
                    let mut dynamic = DynamicStruct::default();
                    dynamic.set_name(type_name);

                    let object = if let Some(Value::Object(object)) = component.remove("struct") {
                        object
                    } else {
                        panic!("struct field is not an object");
                    };

                    for (key, mut value) in object.into_iter() {
                        if let Some(field) = deserialize_gltf_field(&type_registry, &mut value) {
                            dynamic.insert_boxed(&key, field);
                        }
                    }

                    default.apply(dynamic.as_reflect());
                }
                TypeInfo::TupleStruct(_) => {
                    let mut dynamic = DynamicTupleStruct::default();
                    dynamic.set_name(type_name);

                    let object =
                        if let Some(Value::Object(object)) = component.remove("tuple_struct") {
                            object
                        } else {
                            panic!("tuple_struct field is not an object");
                        };

                    for (_, mut value) in object.into_iter() {
                        if let Some(field) = deserialize_gltf_field(&type_registry, &mut value) {
                            dynamic.insert_boxed(field);
                        }
                    }

                    default.apply(dynamic.as_reflect());
                }
                TypeInfo::Tuple(_) => {
                    let mut dynamic = DynamicTuple::default();
                    dynamic.set_name(type_name);

                    let object = if let Some(Value::Object(object)) = component.remove("tuple") {
                        object
                    } else {
                        panic!("tuple field is not an object");
                    };

                    for (_, mut value) in object.into_iter() {
                        if let Some(field) = deserialize_gltf_field(&type_registry, &mut value) {
                            dynamic.insert_boxed(field);
                        }
                    }

                    default.apply(dynamic.as_reflect());
                }
                TypeInfo::List(_) => {
                    let mut dynamic = DynamicList::default();
                    dynamic.set_name(type_name);

                    let object = if let Some(Value::Object(object)) = component.remove("list") {
                        object
                    } else {
                        panic!("list field is not an object");
                    };

                    for (_, mut value) in object.into_iter() {
                        if let Some(field) = deserialize_gltf_field(&type_registry, &mut value) {
                            dynamic.push_box(field);
                        }
                    }

                    default.apply(dynamic.as_reflect());
                }
                TypeInfo::Map(_) => {
                    let mut dynamic = DynamicMap::default();
                    dynamic.set_name(type_name);

                    let object = if let Some(Value::Object(object)) = component.remove("map") {
                        object
                    } else {
                        panic!("map field is not an object");
                    };

                    for (key, mut value) in object.into_iter() {
                        if let Some(field) = deserialize_gltf_field(&type_registry, &mut value) {
                            dynamic.insert_boxed(Box::new(key), field);
                        }
                    }

                    default.apply(dynamic.as_reflect());
                }
                // For unhandled variants, defer to the parent default implementation
                _ => (),
            };

            Some(default)
        }
    }
}

/// Recursively traverse a JSON Value, invoking the supplied callback at each field
fn walk_json<F>(value: &mut Value, callback: &mut F)
where
    F: FnMut(&mut Value),
{
    callback(value);

    match value {
        Value::Array(array) => {
            for value in array {
                walk_json(value, callback);
            }
        }
        Value::Object(object) => {
            for (_, value) in object {
                walk_json(value, callback)
            }
        }
        _ => (),
    }
}

fn fixup_bool(type_registry: &TypeRegistry, value: &mut Value) {
    let object = if let Value::Object(object) = value {
        object
    } else {
        return;
    };

    let type_name = if let Some(Value::String(type_name)) = object.get("type") {
        type_name
    } else {
        return;
    };

    let type_registry = type_registry.read();
    let registration = if let Some(registration) = type_registry.get_with_name(type_name) {
        registration
    } else {
        return;
    };

    if registration.data::<ReflectBoolean>().is_none() {
        return;
    }

    if let Some(Value::Number(number)) = object.remove("value") {
        object.insert(
            "value".to_string(),
            Value::Bool(number.as_u64().expect("Bool representation is non-integer") > 0),
        );
    }
}

fn fixup_entity(world: &mut World, value: &mut Value) {
    let object = if let Value::Object(object) = value {
        object
    } else {
        return;
    };

    if let Some(Value::String(type_name)) = object.get("type") {
        if type_name != std::any::type_name::<Entity>() {
            return;
        }
    }

    let mut entity_value = if let Some(Value::Object(object)) = object.remove("value") {
        object
    } else {
        return;
    };

    let entity_name = if let Some(Value::String(name)) = entity_value.remove("name") {
        name
    } else {
        return;
    };

    info!("Fixing up entity for object {object:#?}");
    info!("Entity value: {entity_value:#?}");
    info!("Entity Name: {entity_name:#?}");

    let mut query = world.query::<(Entity, &Name)>();
    let entity = query
        .iter(world)
        .find_map(|(entity, name)| {
            if **name == entity_name {
                Some(entity)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("No entity with name {entity_name:}"));

    let serialized = serde_json::to_value(&entity).unwrap();
    object.insert("value".to_string(), serialized);
}

fn fixup_enum(type_registry: &TypeRegistry, value: &mut Value) {
    let object = if let Value::Object(object) = value {
        object
    } else {
        return;
    };

    let type_name = if let Some(Value::String(type_name)) = object.get("type") {
        type_name
    } else {
        return;
    };

    let type_registry = type_registry.read();
    let registration = if let Some(registration) = type_registry.get_with_name(type_name) {
        registration
    } else {
        return;
    };

    let enumeration = if let Some(enumeration) = registration.data::<ReflectEnum>() {
        enumeration
    } else {
        return;
    };

    if let Some((key, Value::Number(number))) = object.remove_entry("value") {
        object.insert(
            key,
            Value::String(
                enumeration
                    .variants()
                    .nth(number.as_u64().unwrap() as usize)
                    .unwrap()
                    .to_string(),
            ),
        );
    }
}

fn fixup_option(type_registry: &TypeRegistry, value: &mut Value) {
    let object = if let Value::Object(object) = value {
        object
    } else {
        return;
    };

    let type_name = if let Some(Value::String(type_name)) = object.get("type") {
        type_name
    } else {
        return;
    };

    let type_registry = type_registry.read();
    let registration = if let Some(registration) = type_registry.get_with_name(type_name) {
        registration
    } else {
        return;
    };

    if registration.data::<ReflectOption>().is_none() {
        return;
    }

    if let Some(Value::Object(mut obj)) = object.remove("value") {
        let val = obj.remove("value").unwrap();
        object.insert("value".to_string(), val);
    }
}

fn fixup_tuple_types(type_registry: &TypeRegistry, value: &mut Value) {
    let object = if let Value::Object(object) = value {
        object
    } else {
        return;
    };

    let type_name = if let Some(Value::String(type_name)) = object.get("type") {
        type_name
    } else {
        return;
    };

    let type_registry = type_registry.read();
    let registration = if let Some(registration) = type_registry.get_with_name(type_name) {
        registration
    } else {
        return;
    };

    if let TypeInfo::Tuple(_) = registration.type_info() {
        let mut tuple = object.remove("tuple").unwrap();
        fixup_tuple(&mut tuple);
        object.insert("tuple".to_string(), tuple);
    }

    if let TypeInfo::TupleStruct(_) = registration.type_info() {
        let mut tuple_struct = object.remove("tuple_struct").unwrap();
        fixup_tuple(&mut tuple_struct);
        object.insert("tuple_struct".to_string(), tuple_struct);
    }
}

fn fixup_tuple(tuple: &mut Value) {
    println!("Tuple value: {tuple:#?}");
    let object = if let Value::Object(object) = tuple {
        object
    } else {
        panic!("tuple_struct field is not an object");
    };

    *tuple = Value::Array(
        object
            .iter()
            .map(|(key, value)| {
                let i: usize = key
                    .parse()
                    .unwrap_or_else(|_| panic!("tuple_struct key {key:?} is not numeric"));
                let value = value.clone();
                (i, value)
            })
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect::<Vec<_>>(),
    )
}

fn fixup_map(type_registry: &TypeRegistry, value: &mut Value) {
    let object = if let Value::Object(object) = value {
        object
    } else {
        return;
    };

    let type_name = if let Some(Value::String(type_name)) = object.get("type") {
        type_name
    } else {
        return;
    };

    let type_registry = type_registry.read();
    let registration = if let Some(registration) = type_registry.get_with_name(type_name) {
        registration
    } else {
        return;
    };

    // Fixup maps
    let info = if let TypeInfo::Map(info) = registration.type_info() {
        info
    } else {
        return;
    };

    match (object.remove("keys"), object.remove("values")) {
        (Some(Value::Array(keys)), Some(Value::Array(values))) => {
            object.insert(
                "map".to_string(),
                Value::Object(
                    keys.into_iter()
                        .zip(values.into_iter())
                        .map(|(key, value)| {
                            let (key, value) = match (key, value) {
                                (Value::Object(mut key), Value::Object(mut value)) => {
                                    (key.remove("value").unwrap(), value.remove("value").unwrap())
                                }
                                _ => panic!("Unexpected map format"),
                            };

                            (
                                key.as_str().expect("Key must be String").to_string(),
                                Value::Object(
                                    [
                                        (
                                            "type".to_string(),
                                            Value::String(info.value_type_name().to_string()),
                                        ),
                                        ("value".to_string(), value),
                                    ]
                                    .into_iter()
                                    .collect(),
                                ),
                            )
                        })
                        .collect(),
                ),
            );
        }
        _ => {
            object.insert("value".to_string(), Value::Object(default()));
        }
    }
}
