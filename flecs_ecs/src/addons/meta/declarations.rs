use flecs_ecs_derive::Component;

use crate::core::*;
use crate::sys;

// Primitive type aliases
pub type BoolT = sys::ecs_bool_t;
pub type CharT = sys::ecs_char_t;
pub type U8T = sys::ecs_u8_t;
pub type U16T = sys::ecs_u16_t;
pub type U32T = sys::ecs_u32_t;
pub type U64T = sys::ecs_u64_t;
pub type UptrT = sys::ecs_uptr_t;
pub type I8T = sys::ecs_i8_t;
pub type I16T = sys::ecs_i16_t;
pub type I32T = sys::ecs_i32_t;
pub type I64T = sys::ecs_i64_t;
pub type IptrT = sys::ecs_iptr_t;
pub type F32T = sys::ecs_f32_t;
pub type F64T = sys::ecs_f64_t;

// Embedded type aliases
pub type MemberT = sys::ecs_member_t;
pub type EnumConstantT = sys::ecs_enum_constant_t;
pub type BitmaskConstantT = sys::ecs_bitmask_constant_t;

// Components
pub type Type = sys::EcsType;
pub type TypeSerializer = sys::EcsTypeSerializer;
pub type Primitive = sys::EcsPrimitive;
pub type Enum = sys::EcsEnum;
pub type Bitmask = sys::EcsBitmask;
pub type Member = sys::EcsMember;
pub type MemberRanges = sys::EcsMemberRanges;
pub type Struct = sys::EcsStruct;
pub type Array = sys::EcsArray;
pub type Vector = sys::EcsVector;
pub type Unit = sys::EcsUnit;

// Base type for bitmasks
pub struct EcsBitmask {
    value: u32,
}

pub const BOOL: EntityT = ECS_BOOL_T;
pub const CHAR: EntityT = ECS_CHAR_T;
pub const BYTE: EntityT = ECS_BYTE_T;
pub const U8: EntityT = ECS_U8_T;
pub const U16: EntityT = ECS_U16_T;
pub const U32: EntityT = ECS_U32_T;
pub const U64: EntityT = ECS_U64_T;
pub const U_PTR: EntityT = ECS_UPTR_T;
pub const I8: EntityT = ECS_I8_T;
pub const I16: EntityT = ECS_I16_T;
pub const I32: EntityT = ECS_I32_T;
pub const I64: EntityT = ECS_I64_T;
pub const I_PTR: EntityT = ECS_IPTR_T;
pub const F32: EntityT = ECS_F32_T;
pub const F64: EntityT = ECS_F64_T;
pub const STRING: EntityT = ECS_STRING_T;
pub const ENTITY: EntityT = ECS_ENTITY_T;
pub const CONSTANT: EntityT = ECS_CONSTANT;
pub const QUANTITY: EntityT = ECS_QUANTITY;

#[allow(clippy::unnecessary_cast)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub enum EcsTypeKind {
    PrimitiveType = sys::ecs_type_kind_t_EcsPrimitiveType as u32,
    BitmaskType = sys::ecs_type_kind_t_EcsBitmaskType as u32,
    EnumType = sys::ecs_type_kind_t_EcsEnumType as u32,
    StructType = sys::ecs_type_kind_t_EcsStructType as u32,
    ArrayType = sys::ecs_type_kind_t_EcsArrayType as u32,
    VectorType = sys::ecs_type_kind_t_EcsVectorType as u32,
    OpaqueType = sys::ecs_type_kind_t_EcsOpaqueType as u32,
}

pub(crate) const PRIMITIVE_TYPE: EcsTypeKind = EcsTypeKind::PrimitiveType;
pub(crate) const BITMASK_TYPE: EcsTypeKind = EcsTypeKind::BitmaskType;
pub(crate) const ENUM_TYPE: EcsTypeKind = EcsTypeKind::EnumType;
pub(crate) const STRUCT_TYPE: EcsTypeKind = EcsTypeKind::StructType;
pub(crate) const ARRAY_TYPE: EcsTypeKind = EcsTypeKind::ArrayType;
pub(crate) const VECTOR_TYPE: EcsTypeKind = EcsTypeKind::VectorType;
pub(crate) const OPAQUE_TYPE: EcsTypeKind = EcsTypeKind::OpaqueType;

impl EcsTypeKind {
    pub fn last_type_kind() -> EcsTypeKind {
        EcsTypeKind::OpaqueType
    }
}

/// Component that is automatically added to every type with the right kind.
#[derive(Debug, Copy, Clone, Component)]
#[repr(C)]
pub struct EcsMetaType {
    kind: EcsTypeKind,
    existing: bool, // Indicates if the type exists or is populated from reflection
    partial: bool,  // Indicates if the reflection data is a partial type description
}

#[derive(Debug, PartialEq, Eq, Component)]
#[repr(C)]
pub enum EcsPrimitiveKind {
    Bool = 1,
    Char,
    Byte,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    UPtr,
    IPtr,
    String,
    Entity,
    Id,
}

impl EcsPrimitiveKind {
    pub fn last_primitive_kind() -> EcsPrimitiveKind {
        EcsPrimitiveKind::Id
    }
}
