//! Serialization-friendly version of bevy's reflection data model
//! Used to export type information to external editors

pub mod reflect_boolean;
pub mod reflect_bundle;
pub mod reflect_float;
pub mod reflect_signed_integer;
pub mod reflect_string;
pub mod reflect_unsigned_integer;

use bevy::{
    ecs::{entity::MapEntities, reflect::ReflectMapEntities},
    prelude::{info, App, Bundle, Color, Plugin, ReflectComponent, ReflectDefault},
    reflect::{
        impl_reflect_value, serde::ReflectSerializer, ArrayInfo, FromType, ListInfo, MapInfo,
        NamedField, TypeRegistry, UnnamedField,
    },
    utils::HashMap,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use bevy::{
    prelude::{default, Component, Entity, ReflectDeserialize, ReflectSerialize},
    reflect::{FromReflect, Reflect},
};

use crate::{
    integration::{
        reflect_boolean::ReflectBoolean, reflect_float::ReflectFloat, reflect_string::ReflectString,
    },
    prelude::reflect_bundle::ReflectBundle,
    shmup::collision_group::CollisionGroup,
    util::default_entity,
};

use self::{
    reflect_signed_integer::ReflectSignedInteger, reflect_unsigned_integer::ReflectUnsignedInteger,
};

pub struct ExportIntegrationPlugin {
    pub path: &'static str,
}

impl Default for ExportIntegrationPlugin {
    fn default() -> Self {
        Self { path: "types.json" }
    }
}

impl Plugin for ExportIntegrationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        info!("Exporting Integration...");
        let type_registry = &mut *app.world.resource::<TypeRegistry>().write();

        type_registry.register_type_data::<bool, ReflectDefault>();
        type_registry.register_type_data::<bool, ReflectBoolean>();

        type_registry.register_type_data::<u8, ReflectDefault>();
        type_registry.register_type_data::<u8, ReflectUnsignedInteger>();

        type_registry.register_type_data::<u16, ReflectDefault>();
        type_registry.register_type_data::<u16, ReflectUnsignedInteger>();

        type_registry.register_type_data::<u32, ReflectDefault>();
        type_registry.register_type_data::<u32, ReflectUnsignedInteger>();

        type_registry.register_type_data::<u64, ReflectDefault>();
        type_registry.register_type_data::<u64, ReflectUnsignedInteger>();

        type_registry.register_type_data::<u128, ReflectIntegrationBlacklist>();

        type_registry.register_type_data::<usize, ReflectDefault>();
        type_registry.register_type_data::<usize, ReflectUnsignedInteger>();

        type_registry.register_type_data::<i8, ReflectDefault>();
        type_registry.register_type_data::<i8, ReflectSignedInteger>();

        type_registry.register_type_data::<i16, ReflectDefault>();
        type_registry.register_type_data::<i16, ReflectSignedInteger>();

        type_registry.register_type_data::<i32, ReflectDefault>();
        type_registry.register_type_data::<i32, ReflectSignedInteger>();

        type_registry.register_type_data::<i64, ReflectDefault>();
        type_registry.register_type_data::<i64, ReflectSignedInteger>();

        type_registry.register_type_data::<i128, ReflectIntegrationBlacklist>();

        type_registry.register_type_data::<isize, ReflectDefault>();
        type_registry.register_type_data::<isize, ReflectSignedInteger>();

        type_registry.register_type_data::<f32, ReflectDefault>();
        type_registry.register_type_data::<f32, ReflectFloat>();

        type_registry.register_type_data::<f64, ReflectDefault>();
        type_registry.register_type_data::<f64, ReflectFloat>();

        type_registry.register::<String>();
        type_registry.register_type_data::<String, ReflectDefault>();
        type_registry.register_type_data::<String, ReflectString>();

        type_registry.register::<Cow<'static, str>>();
        type_registry.register_type_data::<Cow<'static, str>, ReflectDefault>();
        type_registry.register_type_data::<Cow<'static, str>, ReflectSerialize>();
        type_registry.register_type_data::<Cow<'static, str>, ReflectDeserialize>();
        type_registry.register_type_data::<Cow<'static, str>, ReflectString>();

        type_registry.register::<Color>();
        type_registry.register_type_data::<Color, ReflectDefault>();

        type_registry.register::<Entity>();
        type_registry.register_type_data::<Entity, ReflectEntity>();

        type_registry.register::<Option<bool>>();
        type_registry.register_type_data::<Option<bool>, ReflectDefault>();
        type_registry.register_type_data::<Option<bool>, ReflectOption>();
        type_registry.register_type_data::<Option<bool>, ReflectSerialize>();
        type_registry.register_type_data::<Option<bool>, ReflectDeserialize>();

        type_registry.register::<Option<u8>>();
        type_registry.register_type_data::<Option<u8>, ReflectDefault>();
        type_registry.register_type_data::<Option<u8>, ReflectOption>();
        type_registry.register_type_data::<Option<u8>, ReflectSerialize>();
        type_registry.register_type_data::<Option<u8>, ReflectDeserialize>();

        type_registry.register::<Option<u16>>();
        type_registry.register_type_data::<Option<u16>, ReflectDefault>();
        type_registry.register_type_data::<Option<u16>, ReflectOption>();
        type_registry.register_type_data::<Option<u16>, ReflectSerialize>();
        type_registry.register_type_data::<Option<u16>, ReflectDeserialize>();

        type_registry.register::<Option<u32>>();
        type_registry.register_type_data::<Option<u32>, ReflectDefault>();
        type_registry.register_type_data::<Option<u32>, ReflectOption>();
        type_registry.register_type_data::<Option<u32>, ReflectSerialize>();
        type_registry.register_type_data::<Option<u32>, ReflectDeserialize>();

        type_registry.register::<Option<u64>>();
        type_registry.register_type_data::<Option<u64>, ReflectDefault>();
        type_registry.register_type_data::<Option<u64>, ReflectOption>();
        type_registry.register_type_data::<Option<u64>, ReflectSerialize>();
        type_registry.register_type_data::<Option<u64>, ReflectDeserialize>();

        type_registry.register::<Option<usize>>();
        type_registry.register_type_data::<Option<usize>, ReflectDefault>();
        type_registry.register_type_data::<Option<usize>, ReflectOption>();
        type_registry.register_type_data::<Option<usize>, ReflectSerialize>();
        type_registry.register_type_data::<Option<usize>, ReflectDeserialize>();

        type_registry.register::<Option<i8>>();
        type_registry.register_type_data::<Option<i8>, ReflectDefault>();
        type_registry.register_type_data::<Option<i8>, ReflectOption>();
        type_registry.register_type_data::<Option<i8>, ReflectSerialize>();
        type_registry.register_type_data::<Option<i8>, ReflectDeserialize>();

        type_registry.register::<Option<i16>>();
        type_registry.register_type_data::<Option<i16>, ReflectDefault>();
        type_registry.register_type_data::<Option<i16>, ReflectOption>();
        type_registry.register_type_data::<Option<i16>, ReflectSerialize>();
        type_registry.register_type_data::<Option<i16>, ReflectDeserialize>();

        type_registry.register::<Option<i32>>();
        type_registry.register_type_data::<Option<i32>, ReflectDefault>();
        type_registry.register_type_data::<Option<i32>, ReflectOption>();
        type_registry.register_type_data::<Option<i32>, ReflectSerialize>();
        type_registry.register_type_data::<Option<i32>, ReflectDeserialize>();

        type_registry.register::<Option<i64>>();
        type_registry.register_type_data::<Option<i64>, ReflectDefault>();
        type_registry.register_type_data::<Option<i64>, ReflectOption>();
        type_registry.register_type_data::<Option<i64>, ReflectSerialize>();
        type_registry.register_type_data::<Option<i64>, ReflectDeserialize>();

        type_registry.register::<Option<isize>>();
        type_registry.register_type_data::<Option<isize>, ReflectDefault>();
        type_registry.register_type_data::<Option<isize>, ReflectOption>();
        type_registry.register_type_data::<Option<isize>, ReflectSerialize>();
        type_registry.register_type_data::<Option<isize>, ReflectDeserialize>();

        type_registry.register::<Option<f32>>();
        type_registry.register_type_data::<Option<f32>, ReflectDefault>();
        type_registry.register_type_data::<Option<f32>, ReflectOption>();
        type_registry.register_type_data::<Option<f32>, ReflectSerialize>();
        type_registry.register_type_data::<Option<f32>, ReflectDeserialize>();

        type_registry.register::<Option<f64>>();
        type_registry.register_type_data::<Option<f64>, ReflectDefault>();
        type_registry.register_type_data::<Option<f64>, ReflectOption>();
        type_registry.register_type_data::<Option<f64>, ReflectSerialize>();
        type_registry.register_type_data::<Option<f64>, ReflectDeserialize>();

        type_registry.register::<Option<String>>();
        type_registry.register_type_data::<Option<String>, ReflectDefault>();
        type_registry.register_type_data::<Option<String>, ReflectOption>();
        type_registry.register_type_data::<Option<String>, ReflectSerialize>();
        type_registry.register_type_data::<Option<String>, ReflectDeserialize>();

        type_registry.register::<Vec<String>>();
        type_registry.register_type_data::<Vec<String>, ReflectDefault>();
        type_registry.register_type_data::<Vec<String>, ReflectSerialize>();
        type_registry.register_type_data::<Vec<String>, ReflectDeserialize>();

        info!("Registered primitive types");
        let mut metadata = IntegrationMetadata::default();

        for ty in type_registry.iter() {
            if ty.data::<ReflectIntegrationBlacklist>().is_some() {
                continue;
            }

            let type_name = ty.type_name();
            let short_name = ty.short_name();

            if metadata.short_name.contains_key(short_name) {
                panic!("Conflicting short name {short_name:}");
            }

            metadata
                .short_name
                .insert(type_name.to_string(), ty.short_name().to_string());

            match ty.type_info() {
                bevy::reflect::TypeInfo::Struct(info) => {
                    metadata.r#struct.insert(
                        type_name.to_string(),
                        info.iter()
                            .map(|field| (field.name().to_string(), field.type_name().to_string()))
                            .collect(),
                    );
                    metadata
                        .type_variant
                        .insert(type_name.to_string(), "struct".to_string());
                }
                bevy::reflect::TypeInfo::TupleStruct(info) => {
                    metadata.tuple_struct.insert(
                        type_name.to_string(),
                        info.iter()
                            .map(|field| field.type_name().to_string())
                            .collect(),
                    );
                    metadata
                        .type_variant
                        .insert(type_name.to_string(), "tuple_struct".to_string());
                }
                bevy::reflect::TypeInfo::Tuple(info) => {
                    metadata.tuple.insert(
                        type_name.to_string(),
                        info.iter()
                            .map(|field| field.type_name().to_string())
                            .collect(),
                    );
                    metadata
                        .type_variant
                        .insert(type_name.to_string(), "tuple".to_string());
                }
                bevy::reflect::TypeInfo::List(info) => {
                    metadata.list.insert(
                        type_name.to_string(),
                        IntegrationListInfo {
                            item_type: info.item_type_name().to_string(),
                        },
                    );
                    metadata
                        .type_variant
                        .insert(type_name.to_string(), "list".to_string());
                }
                bevy::reflect::TypeInfo::Array(info) => {
                    metadata.array.insert(
                        type_name.to_string(),
                        IntegrationArrayInfo {
                            item_type: info.item_type_name().to_string(),
                            capacity: info.capacity(),
                        },
                    );
                    metadata
                        .type_variant
                        .insert(type_name.to_string(), "array".to_string());
                }
                bevy::reflect::TypeInfo::Map(info) => {
                    metadata.map.insert(
                        type_name.to_string(),
                        IntegrationMapInfo {
                            key_type: info.key_type_name().to_string(),
                            value_type: info.value_type_name().to_string(),
                        },
                    );
                    metadata
                        .type_variant
                        .insert(type_name.to_string(), "map".to_string());
                }
                _ => (),
            }

            metadata.type_variant.insert(
                type_name.to_string(),
                match ty.type_info() {
                    bevy::reflect::TypeInfo::Struct(_) => "struct",
                    bevy::reflect::TypeInfo::TupleStruct(_) => "tuple_struct",
                    bevy::reflect::TypeInfo::Tuple(_) => "tuple",
                    bevy::reflect::TypeInfo::List(_) => "list",
                    bevy::reflect::TypeInfo::Array(_) => "array",
                    bevy::reflect::TypeInfo::Map(_) => "map",
                    bevy::reflect::TypeInfo::Value(_) => "value",
                    bevy::reflect::TypeInfo::Dynamic(_) => "dynamic",
                }
                .to_string(),
            );

            if let Some(default) = ty.data::<ReflectDefault>() {
                let default = default.default();

                let default = serde_json::to_value(ReflectSerializer::new(
                    default.as_reflect(),
                    type_registry,
                ));

                if ty.short_name() == "u128" || ty.short_name() == "i128" {
                    default.as_ref().unwrap();
                }

                let default = default.ok();

                if let Some(default) = default {
                    metadata.default.insert(type_name.to_string(), default);
                }
            }

            if ty.data::<ReflectComponent>().is_some() {
                metadata.component.insert(type_name.to_string());
            }

            if ty.data::<ReflectBundle>().is_some() {
                metadata.bundle.insert(type_name.to_string());
            }

            if let Some(_) = ty.data::<ReflectBoolean>() {
                metadata.boolean.insert(type_name.to_string());
            }

            if let Some(unsigned) = ty.data::<ReflectUnsignedInteger>() {
                metadata
                    .integer_unsigned
                    .insert(type_name.to_string(), *unsigned);
            }

            if let Some(signed) = ty.data::<ReflectSignedInteger>() {
                metadata
                    .integer_signed
                    .insert(type_name.to_string(), *signed);
            }

            if let Some(_) = ty.data::<ReflectFloat>() {
                metadata.float.insert(type_name.to_string());
            }

            if let Some(_) = ty.data::<ReflectString>() {
                metadata.string.insert(type_name.to_string());
            }

            if ty.data::<ReflectEntity>().is_some() {
                metadata.entity.insert(type_name.to_string());
            }

            if let Some(option) = ty.data::<ReflectOption>() {
                metadata
                    .option
                    .insert(type_name.to_string(), option.clone());
            }

            if let Some(enumeration) = ty.data::<ReflectEnum>() {
                metadata.enumeration.insert(
                    type_name.to_string(),
                    enumeration
                        .variants()
                        .map(|variant| variant.to_string())
                        .collect(),
                );
            }

            if let Some(bitflags) = ty.data::<ReflectBitflags>() {
                metadata.bitflags.insert(
                    type_name.to_string(),
                    bitflags
                        .flags()
                        .map(|(key, value)| (value, key.to_string()))
                        .collect(),
                );
            }
        }

        let out = serde_json::to_string_pretty(&metadata).unwrap();

        println!("Writing to {}", self.path);
        std::fs::write(self.path, out).expect("Failed to write type manifest");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationNamedField {
    pub r#type: String,
}

impl From<&NamedField> for IntegrationNamedField {
    fn from(field: &NamedField) -> Self {
        IntegrationNamedField {
            r#type: field.type_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationUnnamedField {
    pub r#type: String,
}

impl From<&UnnamedField> for IntegrationUnnamedField {
    fn from(field: &UnnamedField) -> Self {
        IntegrationUnnamedField {
            r#type: field.type_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationListInfo {
    pub item_type: String,
}

impl From<&ListInfo> for IntegrationListInfo {
    fn from(info: &ListInfo) -> Self {
        IntegrationListInfo {
            item_type: info.item_type_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationArrayInfo {
    pub item_type: String,
    pub capacity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationMapInfo {
    pub key_type: String,
    pub value_type: String,
}

impl From<&MapInfo> for IntegrationMapInfo {
    fn from(info: &MapInfo) -> Self {
        IntegrationMapInfo {
            key_type: info.key_type_name().to_string(),
            value_type: info.value_type_name().to_string(),
        }
    }
}

impl From<&ArrayInfo> for IntegrationArrayInfo {
    fn from(info: &ArrayInfo) -> Self {
        IntegrationArrayInfo {
            item_type: info.item_type_name().to_string(),
            capacity: info.capacity(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationTypeInfo {
    #[serde(rename = "struct")]
    Struct(indexmap::IndexMap<String, IntegrationNamedField>),
    #[serde(rename = "tuple_struct")]
    TupleStruct(Vec<IntegrationUnnamedField>),
    #[serde(rename = "tuple")]
    Tuple(Vec<IntegrationUnnamedField>),
    #[serde(rename = "list")]
    List(IntegrationListInfo),
    #[serde(rename = "array")]
    Array(IntegrationArrayInfo),
    #[serde(rename = "map")]
    Map(IntegrationMapInfo),
    #[serde(rename = "value")]
    Value(()),
    #[serde(rename = "dynamic")]
    Dynamic(()),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UIntMetadata {
    min: u128,
    max: u128,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntMetadata {
    min: i128,
    max: i128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    default: Entity,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntegrationMetadata {
    types: BTreeMap<String, IntegrationTypeInfo>,
    type_variant: BTreeMap<String, String>,
    short_name: BTreeMap<String, String>,

    boolean: BTreeSet<String>,
    integer_unsigned: BTreeMap<String, ReflectUnsignedInteger>,
    integer_signed: BTreeMap<String, ReflectSignedInteger>,
    float: BTreeSet<String>,
    string: BTreeSet<String>,
    option: BTreeMap<String, ReflectOption>,
    enumeration: BTreeMap<String, Vec<String>>,
    bitflags: BTreeMap<String, BTreeMap<usize, String>>,
    list: BTreeMap<String, IntegrationListInfo>,
    array: BTreeMap<String, IntegrationArrayInfo>,
    map: BTreeMap<String, IntegrationMapInfo>,
    r#struct: BTreeMap<String, indexmap::IndexMap<String, String>>,
    tuple_struct: BTreeMap<String, Vec<String>>,
    tuple: BTreeMap<String, Vec<String>>,
    entity: BTreeSet<String>,

    default: BTreeMap<String, serde_json::Value>,
    component: BTreeSet<String>,
    bundle: BTreeSet<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReflectOption {
    pub inner_type: String,
}

impl<T> FromType<Option<T>> for ReflectOption {
    fn from_type() -> Self {
        ReflectOption {
            inner_type: std::any::type_name::<T>().to_string(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReflectEnum {
    pub variants: Vec<String>,
}

impl ReflectEnum {
    pub fn variants(&self) -> impl Iterator<Item = &str> {
        self.variants.iter().map(|variant| variant.as_str())
    }
}

impl<T> FromType<T> for ReflectEnum
where
    T: Enum,
{
    fn from_type() -> Self {
        ReflectEnum {
            variants: T::variants().into_iter().map(ToString::to_string).collect(),
        }
    }
}

pub trait Enum {
    fn variants() -> &'static [&'static str];
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Default, Enum, Component, Serialize, Deserialize)]
pub enum TestEnum {
    Foo,
    Bar,
    Baz,
    Decafisbad,
}

impl Default for TestEnum {
    fn default() -> Self {
        TestEnum::Foo
    }
}

impl Enum for TestEnum {
    fn variants() -> &'static [&'static str] {
        &["Foo", "Bar", "Baz", "Decafisbad"]
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReflectBitflags {
    pub flags: BTreeMap<String, usize>,
}

impl ReflectBitflags {
    pub fn flag(&self, name: &str) -> Option<usize> {
        self.flags.get(name).copied()
    }

    pub fn flags(&self) -> impl Iterator<Item = (&str, usize)> {
        self.flags.iter().map(|(key, value)| (key.as_str(), *value))
    }
}

impl<T> FromType<T> for ReflectBitflags
where
    T: Bitflags,
{
    fn from_type() -> Self {
        ReflectBitflags { flags: T::flags() }
    }
}

pub trait Bitflags {
    fn flags() -> BTreeMap<String, usize>;

    fn flag(flag: &str) -> Option<usize> {
        Self::flags().get(flag).copied()
    }
}

impl Bitflags for CollisionGroup {
    fn flags() -> BTreeMap<String, usize> {
        [
            ("STATIC", CollisionGroup::STATIC),
            ("SHIP_BODY", CollisionGroup::SHIP_BODY),
            ("SHIP_HURTBOX", CollisionGroup::SHIP_HURTBOX),
            ("SHIP_HITBOX", CollisionGroup::SHIP_HITBOX),
            ("SHIP_SENSOR", CollisionGroup::SHIP_SENSOR),
            ("FORCE_BODY", CollisionGroup::FORCE_BODY),
            ("FORCE_HITBOX", CollisionGroup::FORCE_HITBOX),
            ("FORCE_SENSOR", CollisionGroup::FORCE_SENSOR),
            ("ENEMY_BODY", CollisionGroup::ENEMY_BODY),
            ("ENEMY_HURTBOX", CollisionGroup::ENEMY_HURTBOX),
            ("ENEMY_HITBOX", CollisionGroup::ENEMY_HITBOX),
        ]
        .map(|(key, value)| (key.to_string(), value.bits() as usize))
        .into_iter()
        .collect()
    }
}

/// Prevents a type from being exported to integration format
#[derive(Debug, Default, Copy, Clone)]
pub struct ReflectIntegrationBlacklist;

impl<T> FromType<T> for ReflectIntegrationBlacklist {
    fn from_type() -> Self {
        default()
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ReflectEntity;

impl FromType<Entity> for ReflectEntity {
    fn from_type() -> Self {
        default()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Component, Reflect, FromReflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct TestTupleStruct(pub Cow<'static, str>, pub Option<usize>);

pub type TestTuple = (Entity, isize);

#[derive(Debug, Clone, Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, MapEntities)]
pub struct TestStruct {
    pub foo: f32,
    pub bar: TestTuple,
    pub baz: Vec<String>,
}

impl Default for TestStruct {
    fn default() -> Self {
        Self {
            foo: default(),
            bar: (default_entity(), default()),
            baz: default(),
        }
    }
}

impl MapEntities for TestStruct {
    fn map_entities(
        &mut self,
        entity_map: &bevy::ecs::entity::EntityMap,
    ) -> Result<(), bevy::ecs::entity::MapEntitiesError> {
        self.bar.0 = entity_map.get(self.bar.0)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Bundle, Reflect)]
#[reflect(Default, Bundle)]
pub struct TestBundle {
    pub r#struct: TestStruct,
    pub tuple_struct: TestTupleStruct,
    pub list: TestList,
    pub array: TestArray,
    pub map: TestMap,
    pub enumeration: TestEnum,
    pub struct_private: TestStructPrivate,
    pub option_private: TestOptionPrivate,
    pub list_private: TestListPrivate,
    pub array_private: TestArrayPrivate,
    pub map_private: TestMapPrivate,
}

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Default, Component)]
pub struct TestOption(Option<String>);

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Default, Component)]
pub struct TestList(Vec<String>);

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Default, Component)]
pub struct TestArray([String; 4]);

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Default, Component)]
pub struct TestMap(HashMap<String, String>);

#[derive(Debug, Clone, Default, Component, Serialize, Deserialize)]
pub struct TestStructPrivate(TestStruct);

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct TestOptionPrivate(Option<String>);

impl Default for TestOptionPrivate {
    fn default() -> Self {
        Self(Some("Hello World".to_string()))
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct TestListPrivate(Vec<String>);

impl Default for TestListPrivate {
    fn default() -> Self {
        Self(vec![
            "Foo".to_string(),
            "Bar".to_string(),
            "Baz".to_string(),
        ])
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct TestArrayPrivate([String; 4]);

impl Default for TestArrayPrivate {
    fn default() -> Self {
        Self([
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
            "Four".to_string(),
        ])
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct TestMapPrivate(HashMap<String, String>);

impl Default for TestMapPrivate {
    fn default() -> Self {
        Self(
            [
                ("One".to_string(), "Two".to_string()),
                ("Three".to_string(), "Four".to_string()),
            ]
            .into_iter()
            .collect(),
        )
    }
}

impl_reflect_value!(TestStructPrivate);
impl_reflect_value!(TestOptionPrivate);
impl_reflect_value!(TestListPrivate);
impl_reflect_value!(TestArrayPrivate);
impl_reflect_value!(TestMapPrivate);

pub fn register_test_types(app: &mut App) {
    app.register_type::<TestEnum>();
    app.register_type::<TestOption>();
    app.register_type::<TestList>();
    app.register_type::<TestArray>();
    app.register_type::<TestMap>();
    app.register_type::<TestStruct>();
    app.register_type::<TestTupleStruct>();
    app.register_type::<TestTuple>();
    app.register_type::<TestBundle>();

    app.register_type::<TestOptionPrivate>();
    app.register_type_data::<TestOptionPrivate, ReflectDefault>();
    app.register_type_data::<TestOptionPrivate, ReflectComponent>();
    app.register_type_data::<TestOptionPrivate, ReflectSerialize>();
    app.register_type_data::<TestOptionPrivate, ReflectDeserialize>();

    app.register_type::<TestListPrivate>();
    app.register_type_data::<TestListPrivate, ReflectDefault>();
    app.register_type_data::<TestListPrivate, ReflectComponent>();
    app.register_type_data::<TestListPrivate, ReflectSerialize>();
    app.register_type_data::<TestListPrivate, ReflectDeserialize>();

    app.register_type::<TestArrayPrivate>();
    app.register_type_data::<TestArrayPrivate, ReflectDefault>();
    app.register_type_data::<TestArrayPrivate, ReflectComponent>();
    app.register_type_data::<TestArrayPrivate, ReflectSerialize>();
    app.register_type_data::<TestArrayPrivate, ReflectDeserialize>();

    app.register_type::<TestMapPrivate>();
    app.register_type_data::<TestMapPrivate, ReflectDefault>();
    app.register_type_data::<TestMapPrivate, ReflectComponent>();
    app.register_type_data::<TestMapPrivate, ReflectSerialize>();
    app.register_type_data::<TestMapPrivate, ReflectDeserialize>();

    app.register_type::<TestStructPrivate>();
    app.register_type_data::<TestStructPrivate, ReflectDefault>();
    app.register_type_data::<TestStructPrivate, ReflectComponent>();
    app.register_type_data::<TestStructPrivate, ReflectSerialize>();
    app.register_type_data::<TestStructPrivate, ReflectDeserialize>();

    app.register_type::<[String; 4]>();

    app.register_type::<HashMap<String, String>>();
    app.register_type_data::<HashMap<String, String>, ReflectSerialize>();
    app.register_type_data::<HashMap<String, String>, ReflectDeserialize>();
}

#[test]
pub fn test_export() {
    let mut app = App::new();
    register_test_types(&mut app);
    app.add_plugin(ExportIntegrationPlugin {
        path: "assets/blender/types_test.json",
    });
    app.run();
}
