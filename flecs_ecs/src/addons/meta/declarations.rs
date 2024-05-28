use flecs_ecs_derive::Component;

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

// Base type for bitmasks
pub struct EcsBitmask {
    value: u32,
}

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
