#![doc(hidden)]
#![allow(non_upper_case_globals)]

use std::ffi::CStr;

use crate::sys;

pub const RUST_ecs_id_FLAGS_MASK: u64 = 0xFF << 60;
pub const RUST_ECS_COMPONENT_MASK: u64 = !RUST_ecs_id_FLAGS_MASK;

pub type WorldT = sys::ecs_world_t;
pub type WorldInfoT = sys::ecs_world_info_t;
pub type IdT = sys::ecs_id_t;
pub type EntityT = sys::ecs_entity_t;
pub type TypeT = sys::ecs_type_t;
pub type TableT = sys::ecs_table_t;
pub type ObserverT = sys::ecs_observer_t;
pub type QueryT = sys::ecs_query_t;
pub type QueryGroupInfoT = sys::ecs_query_group_info_t;
pub type RefT = sys::ecs_ref_t;
pub type IterT = sys::ecs_iter_t;
pub type TypeInfoT = sys::ecs_type_info_t;
pub type TypeHooksT = sys::ecs_type_hooks_t;
pub type TypeKindT = sys::ecs_type_kind_t;
pub type Flags32T = sys::ecs_flags32_t;
pub type TermRefT = sys::ecs_term_ref_t;
pub type TermT = sys::ecs_term_t;
pub type PrimitiveKindT = sys::ecs_primitive_kind_t;
pub type FTimeT = f32;

pub static SEPARATOR: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"::\0") };

/// Specify read/write access for term
/// Specifies the access pattern of a system to a component term.
///
/// This enum is used to indicate how a system interacts with a component term during its execution,
/// differentiating between read-only access, write-only access, both, or neither.
///
/// Variants:
///
/// - `Default`: Default behavior, which is `InOut` for regular terms and `In` for shared terms.
/// - `None`: Indicates the term is neither read nor written by the system.
/// - `Filter`: Same as `None` + prevents term from triggering observers
/// - `InOut`: The term is both read and written, implying a mutable access to the component data.
/// - `In`: The term is only read, implying an immutable access to the component data.
/// - `Out`: The term is only written, providing exclusive access to modify the component data.
#[allow(clippy::unnecessary_cast)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InOutKind {
    Default = sys::ecs_inout_kind_t_EcsInOutDefault as u32,
    None = sys::ecs_inout_kind_t_EcsInOutNone as u32,
    Filter = sys::ecs_inout_kind_t_EcsInOutFilter as u32,
    InOut = sys::ecs_inout_kind_t_EcsInOut as u32,
    In = sys::ecs_inout_kind_t_EcsIn as u32,
    Out = sys::ecs_inout_kind_t_EcsOut as u32,
}

impl InOutKind {
    pub fn is_read_only(&self) -> bool {
        matches!(self, Self::In)
    }
}

impl From<sys::ecs_inout_kind_t> for InOutKind {
    fn from(value: sys::ecs_inout_kind_t) -> Self {
        match value {
            sys::ecs_inout_kind_t_EcsInOutDefault => InOutKind::Default,
            sys::ecs_inout_kind_t_EcsInOutNone => InOutKind::None,
            sys::ecs_inout_kind_t_EcsInOutFilter => InOutKind::Filter,
            sys::ecs_inout_kind_t_EcsInOut => InOutKind::InOut,
            sys::ecs_inout_kind_t_EcsIn => InOutKind::In,
            sys::ecs_inout_kind_t_EcsOut => InOutKind::Out,
            _ => InOutKind::Default,
        }
    }
}

const EcsInOutDefault: i16 = sys::ecs_inout_kind_t_EcsInOutDefault as i16;
const EcsInOutNone: i16 = sys::ecs_inout_kind_t_EcsInOutNone as i16;
const EcsInOut: i16 = sys::ecs_inout_kind_t_EcsInOut as i16;
const EcsIn: i16 = sys::ecs_inout_kind_t_EcsIn as i16;
const EcsOut: i16 = sys::ecs_inout_kind_t_EcsOut as i16;
const EcsInOutFilter: i16 = sys::ecs_inout_kind_t_EcsInOutFilter as i16;

impl From<i16> for InOutKind {
    fn from(value: i16) -> Self {
        match value {
            EcsInOutDefault => InOutKind::Default,
            EcsInOutNone => InOutKind::None,
            EcsInOutFilter => InOutKind::Filter,
            EcsInOut => InOutKind::InOut,
            EcsIn => InOutKind::In,
            EcsOut => InOutKind::Out,
            _ => InOutKind::Default,
        }
    }
}

impl From<InOutKind> for i16 {
    fn from(value: InOutKind) -> Self {
        match value {
            InOutKind::Default => sys::ecs_inout_kind_t_EcsInOutDefault as i16,
            InOutKind::None => sys::ecs_inout_kind_t_EcsInOutNone as i16,
            InOutKind::Filter => sys::ecs_inout_kind_t_EcsInOutFilter as i16,
            InOutKind::InOut => sys::ecs_inout_kind_t_EcsInOut as i16,
            InOutKind::In => sys::ecs_inout_kind_t_EcsIn as i16,
            InOutKind::Out => sys::ecs_inout_kind_t_EcsOut as i16,
        }
    }
}

/// Specify operator for term
/// Represents the logical operation applied to a term within a query.
///
/// This enum defines how a term within a query is matched against entities in the ECS,
/// supporting complex query compositions through logical operations.
///
/// Variants:
///
/// - `And`: The term must be present for an entity to match.
/// - `Or`: At least one of the terms in an `Or` chain must be present for an entity to match.
/// - `Not`: The term must not be present for an entity to match.
/// - `Optional`: The presence or absence of the term does not affect matching.
/// - `AndFrom`: All components identified by the term's ID must be present for an entity to match.
/// - `OrFrom`: At least one component identified by the term's ID must be present for an entity to match.
/// - `NotFrom`: None of the components identified by the term's ID should be present for an entity to match.
///
/// These operations allow for flexible and powerful queries within an ECS framework, enabling
/// systems to precisely specify the conditions under which entities are selected for processing.
#[allow(clippy::unnecessary_cast)]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OperKind {
    And = sys::ecs_oper_kind_t_EcsAnd as u32,
    Or = sys::ecs_oper_kind_t_EcsOr as u32,
    Not = sys::ecs_oper_kind_t_EcsNot as u32,
    Optional = sys::ecs_oper_kind_t_EcsOptional as u32,
    AndFrom = sys::ecs_oper_kind_t_EcsAndFrom as u32,
    OrFrom = sys::ecs_oper_kind_t_EcsOrFrom as u32,
    NotFrom = sys::ecs_oper_kind_t_EcsNotFrom as u32,
}

impl OperKind {
    pub fn is_negation(&self) -> bool {
        matches!(self, Self::Not | Self::NotFrom)
    }
}

impl From<sys::ecs_oper_kind_t> for OperKind {
    fn from(value: sys::ecs_oper_kind_t) -> Self {
        match value {
            sys::ecs_oper_kind_t_EcsAnd => OperKind::And,
            sys::ecs_oper_kind_t_EcsOr => OperKind::Or,
            sys::ecs_oper_kind_t_EcsNot => OperKind::Not,
            sys::ecs_oper_kind_t_EcsOptional => OperKind::Optional,
            sys::ecs_oper_kind_t_EcsAndFrom => OperKind::AndFrom,
            sys::ecs_oper_kind_t_EcsOrFrom => OperKind::OrFrom,
            sys::ecs_oper_kind_t_EcsNotFrom => OperKind::NotFrom,
            _ => OperKind::And,
        }
    }
}

const EcsAnd: i16 = sys::ecs_oper_kind_t_EcsAnd as i16;
const EcsOr: i16 = sys::ecs_oper_kind_t_EcsOr as i16;
const EcsNot: i16 = sys::ecs_oper_kind_t_EcsNot as i16;
const EcsOptional: i16 = sys::ecs_oper_kind_t_EcsOptional as i16;
const EcsAndFrom: i16 = sys::ecs_oper_kind_t_EcsAndFrom as i16;
const EcsOrFrom: i16 = sys::ecs_oper_kind_t_EcsOrFrom as i16;
const EcsNotFrom: i16 = sys::ecs_oper_kind_t_EcsNotFrom as i16;

impl From<i16> for OperKind {
    fn from(value: i16) -> Self {
        match value {
            EcsAnd => OperKind::And,
            EcsOr => OperKind::Or,
            EcsNot => OperKind::Not,
            EcsOptional => OperKind::Optional,
            EcsAndFrom => OperKind::AndFrom,
            EcsOrFrom => OperKind::OrFrom,
            EcsNotFrom => OperKind::NotFrom,
            _ => OperKind::And,
        }
    }
}

impl From<OperKind> for i16 {
    fn from(value: OperKind) -> Self {
        match value {
            OperKind::And => sys::ecs_oper_kind_t_EcsAnd as i16,
            OperKind::Or => sys::ecs_oper_kind_t_EcsOr as i16,
            OperKind::Not => sys::ecs_oper_kind_t_EcsNot as i16,
            OperKind::Optional => sys::ecs_oper_kind_t_EcsOptional as i16,
            OperKind::AndFrom => sys::ecs_oper_kind_t_EcsAndFrom as i16,
            OperKind::OrFrom => sys::ecs_oper_kind_t_EcsOrFrom as i16,
            OperKind::NotFrom => sys::ecs_oper_kind_t_EcsNotFrom as i16,
        }
    }
}

/// Specify cache policy for query
///
/// This enum specifies the caching policy for a query, which determines how the query results are stored
///
/// Variants:
///
/// - `Default`: Behavior determined by query creation context
/// - `Auto`: Cache query terms that are cacheable
/// - `All`: Require that all query terms can be cached
/// - `None`: No caching
#[allow(clippy::unnecessary_cast)]
#[repr(u32)]
pub enum QueryCacheKind {
    Default = sys::ecs_query_cache_kind_t_EcsQueryCacheDefault as u32,
    Auto = sys::ecs_query_cache_kind_t_EcsQueryCacheAuto as u32,
    All = sys::ecs_query_cache_kind_t_EcsQueryCacheAll as u32,
    None = sys::ecs_query_cache_kind_t_EcsQueryCacheNone as u32,
}

impl QueryCacheKind {
    pub fn is_auto(&self) -> bool {
        matches!(self, Self::Auto)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }
}

impl From<sys::ecs_query_cache_kind_t> for QueryCacheKind {
    fn from(value: sys::ecs_query_cache_kind_t) -> Self {
        match value {
            sys::ecs_query_cache_kind_t_EcsQueryCacheDefault => QueryCacheKind::Default,
            sys::ecs_query_cache_kind_t_EcsQueryCacheAuto => QueryCacheKind::Auto,
            sys::ecs_query_cache_kind_t_EcsQueryCacheAll => QueryCacheKind::All,
            sys::ecs_query_cache_kind_t_EcsQueryCacheNone => QueryCacheKind::None,
            _ => QueryCacheKind::Default,
        }
    }
}

// Id flags

/// Indicates that the id is a pair.
pub(crate) const ECS_PAIR: u64 = 1 << 63;
/// Automatically override component when it is inherited
pub(crate) const ECS_AUTO_OVERRIDE: u64 = 1 << 62;
/// Adds bitset to storage which allows component to be enabled/disabled
pub(crate) const ECS_TOGGLE: u64 = 1 << 61;
/// Include all components from entity to which AND is applied
pub(crate) const ECS_AND: u64 = 1 << 60;

// Builtin component ids
pub(crate) const ECS_COMPONENT: u64 = 1;
pub(crate) const ECS_IDENTIFIER: u64 = 2;
pub(crate) const ECS_ITERABLE: u64 = 3;
pub(crate) const ECS_POLY: u64 = 4;

// Poly target components
pub(crate) const ECS_QUERY: u64 = 5;
pub(crate) const ECS_OBSERVER: u64 = 6;
pub(crate) const ECS_SYSTEM: u64 = 7;

///Term id flags

///  The base ID, equivalent to the C #define
pub(crate) const FLECS_HI_COMPONENT_ID: u64 = 256;

/// Match on self
/// Can be combined with other term flags on the `ecs_term_t::flags` field
pub(crate) const ECS_SELF: u64 = 1 << 63;

/// Match by traversing upwards
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_UP: u64 = 1 << 62;

/// Match by traversing downwards (derived, cannot be set)
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_TRAV: u64 = 1 << 61;

/// Sort results breadth first
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_CASCADE: u64 = 1 << 60;

/// Iterate groups in descending order (used for ordering)
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_DESC: u64 = 1 << 59;

/// Term id is a variable
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_IS_VARIABLE: u64 = 1 << 58;

/// Term id is an entity
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_IS_ENTITY: u64 = 1 << 57;

/// Term id is a name (don't attempt to lookup as entity)
/// Can be combined with other term flags on the `ecs_term_ref_t::id` field.
pub(crate) const ECS_IS_NAME: u64 = 1 << 56;

/// all term traversal flags
pub(crate) const ECS_TRAVERSE_FLAGS: u64 = ECS_SELF | ECS_UP | ECS_TRAV | ECS_CASCADE | ECS_DESC;

/// all term reference kind flags
pub(crate) const ECS_TERM_REF_FLAGS: u64 =
    ECS_TRAVERSE_FLAGS | ECS_IS_VARIABLE | ECS_IS_ENTITY | ECS_IS_NAME;

/// Term flags
/// Term flags discovered & set during query creation.
/// Mostly used internally to store information relevant to queries.
pub(crate) const MATCH_ANY: u64 = 1 << 0;
pub(crate) const MATCH_ANY_SRC: u64 = 1 << 1;
pub(crate) const TRANSITIVE: u64 = 1 << 2;
pub(crate) const REFLEXIVE: u64 = 1 << 3;
pub(crate) const ID_INHERITED: u64 = 1 << 4;
pub(crate) const IS_TRIVIAL: u64 = 1 << 5;
pub(crate) const NO_DATA: u64 = 1 << 6;
pub(crate) const IS_CACHEABLE: u64 = 1 << 7;
pub(crate) const IS_SCOPE: u64 = 1 << 8;
pub(crate) const IS_MEMBER: u64 = 1 << 9;
pub(crate) const IS_TOGGLE: u64 = 1 << 10;

/// Query flags
/// Query flags discovered & set during query creation.

/// Query must match prefabs.
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_MATCH_PREFAB: u64 = 1 << 1;

/// Query must match disabled entities.
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_MATCH_DISABLED: u64 = 1 << 2;

/// Query must match empty tables.
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_MATCH_EMPTY_TABLES: u64 = 1 << 3;

/// Query won't provide component data.
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_NO_DATA: u64 = 1 << 4;

/// Query iteration is always instanced.
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_IS_INSTANCED: u64 = 1 << 5;

/// Query may have unresolved entity identifiers.
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_ALLOW_UNRESOLVED_BY_NAME: u64 = 1 << 6;

/// Query only returns whole tables (ignores toggle/member fields).
/// Can be combined with other query flags on the `ecs_query_desc_t::flags` field.
pub(crate) const ECS_QUERY_TABLE_ONLY: u64 = 1 << 7;

// Core scopes & entities
pub(crate) const ECS_WORLD: u64 = FLECS_HI_COMPONENT_ID;
pub(crate) const ECS_FLECS: u64 = FLECS_HI_COMPONENT_ID + 1;
pub(crate) const ECS_FLECS_CORE: u64 = FLECS_HI_COMPONENT_ID + 2;
pub(crate) const ECS_FLECS_INTERNALS: u64 = FLECS_HI_COMPONENT_ID + 3;
pub(crate) const ECS_MODULE: u64 = FLECS_HI_COMPONENT_ID + 4;
pub(crate) const ECS_PRIVATE: u64 = FLECS_HI_COMPONENT_ID + 5;
pub(crate) const ECS_PREFAB: u64 = FLECS_HI_COMPONENT_ID + 6;
pub(crate) const ECS_DISABLED: u64 = FLECS_HI_COMPONENT_ID + 7;
pub(crate) const ECS_NOT_QUERYABLE: u64 = FLECS_HI_COMPONENT_ID + 8;
pub(crate) const ECS_SLOT_OF: u64 = FLECS_HI_COMPONENT_ID + 9;
pub(crate) const ECS_FLAG: u64 = FLECS_HI_COMPONENT_ID + 10;

// Marker entities for query encoding

pub(crate) const ECS_WILDCARD: u64 = FLECS_HI_COMPONENT_ID + 11;
pub(crate) const ECS_ANY: u64 = FLECS_HI_COMPONENT_ID + 12;
pub(crate) const ECS_THIS: u64 = FLECS_HI_COMPONENT_ID + 13;
pub(crate) const ECS_VARIABLE: u64 = FLECS_HI_COMPONENT_ID + 14;

// query traits
pub(crate) const ECS_TRANSITIVE: u64 = FLECS_HI_COMPONENT_ID + 15;
pub(crate) const ECS_REFLEXIVE: u64 = FLECS_HI_COMPONENT_ID + 16;
pub(crate) const ECS_SYMMETRIC: u64 = FLECS_HI_COMPONENT_ID + 17;
pub(crate) const ECS_FINAL: u64 = FLECS_HI_COMPONENT_ID + 18;
pub(crate) const ECS_ON_INSTANTIATE: u64 = FLECS_HI_COMPONENT_ID + 19;
pub(crate) const ECS_OVERRIDE: u64 = FLECS_HI_COMPONENT_ID + 20;
pub(crate) const ECS_INHERIT: u64 = FLECS_HI_COMPONENT_ID + 21;
pub(crate) const ECS_DONT_INHERIT: u64 = FLECS_HI_COMPONENT_ID + 22;
pub(crate) const ECS_PAIR_IS_TAG: u64 = FLECS_HI_COMPONENT_ID + 23;
pub(crate) const ECS_EXCLUSIVE: u64 = FLECS_HI_COMPONENT_ID + 24;
pub(crate) const ECS_ACYCLIC: u64 = FLECS_HI_COMPONENT_ID + 25;
pub(crate) const ECS_TRAVERSABLE: u64 = FLECS_HI_COMPONENT_ID + 26;
pub(crate) const ECS_WITH: u64 = FLECS_HI_COMPONENT_ID + 27;
pub(crate) const ECS_ONE_OF: u64 = FLECS_HI_COMPONENT_ID + 28;
pub(crate) const ECS_CAN_TOGGLE: u64 = FLECS_HI_COMPONENT_ID + 29;
pub(crate) const ECS_TRAIT: u64 = FLECS_HI_COMPONENT_ID + 30;
pub(crate) const ECS_RELATIONSHIP: u64 = FLECS_HI_COMPONENT_ID + 31;
pub(crate) const ECS_TARGET: u64 = FLECS_HI_COMPONENT_ID + 32;

// Builtin relationships
pub(crate) const ECS_CHILD_OF: u64 = FLECS_HI_COMPONENT_ID + 33;
pub(crate) const ECS_IS_A: u64 = FLECS_HI_COMPONENT_ID + 34;
pub(crate) const ECS_DEPENDS_ON: u64 = FLECS_HI_COMPONENT_ID + 35;

// Identifier tags
pub(crate) const ECS_NAME: u64 = FLECS_HI_COMPONENT_ID + 36;
pub(crate) const ECS_SYMBOL: u64 = FLECS_HI_COMPONENT_ID + 37;
pub(crate) const ECS_ALIAS: u64 = FLECS_HI_COMPONENT_ID + 38;

// Events
pub(crate) const ECS_ON_ADD: u64 = FLECS_HI_COMPONENT_ID + 39;
pub(crate) const ECS_ON_REMOVE: u64 = FLECS_HI_COMPONENT_ID + 40;
pub(crate) const ECS_ON_SET: u64 = FLECS_HI_COMPONENT_ID + 41;
pub(crate) const ECS_UNSET: u64 = FLECS_HI_COMPONENT_ID + 42;
pub(crate) const ECS_ON_DELETE: u64 = FLECS_HI_COMPONENT_ID + 43;
pub(crate) const ECS_ON_DELETE_TARGET: u64 = FLECS_HI_COMPONENT_ID + 44;
pub(crate) const ECS_ON_TABLE_CREATE: u64 = FLECS_HI_COMPONENT_ID + 45;
pub(crate) const ECS_ON_TABLE_DELETE: u64 = FLECS_HI_COMPONENT_ID + 46;
pub(crate) const ECS_ON_TABLE_EMPTY: u64 = FLECS_HI_COMPONENT_ID + 47;
pub(crate) const ECS_ON_TABLE_FILL: u64 = FLECS_HI_COMPONENT_ID + 48;

// Timers
pub(crate) const ECS_TICK_SOURCE: u64 = FLECS_HI_COMPONENT_ID + 49;
pub(crate) const ECS_TIMER: u64 = FLECS_HI_COMPONENT_ID + 50;
pub(crate) const ECS_RATE_FILTER: u64 = FLECS_HI_COMPONENT_ID + 51;

// Actions
pub(crate) const ECS_REMOVE: u64 = FLECS_HI_COMPONENT_ID + 52;
pub(crate) const ECS_DELETE: u64 = FLECS_HI_COMPONENT_ID + 53;
pub(crate) const ECS_PANIC: u64 = FLECS_HI_COMPONENT_ID + 54;

pub(crate) const ECS_SPARSE: u64 = FLECS_HI_COMPONENT_ID + 55;
pub(crate) const ECS_UNION: u64 = FLECS_HI_COMPONENT_ID + 56;

// Misc
pub(crate) const ECS_DEFAULT_CHILD_COMPONENT: u64 = FLECS_HI_COMPONENT_ID + 57;

// Builtin predicate ids (used by rule engine)
pub(crate) const ECS_PRED_EQ: u64 = FLECS_HI_COMPONENT_ID + 58;
pub(crate) const ECS_PRED_MATCH: u64 = FLECS_HI_COMPONENT_ID + 59;
pub(crate) const ECS_PRED_LOOKUP: u64 = FLECS_HI_COMPONENT_ID + 60;
pub(crate) const ECS_SCOPE_OPEN: u64 = FLECS_HI_COMPONENT_ID + 61;
pub(crate) const ECS_SCOPE_CLOSE: u64 = FLECS_HI_COMPONENT_ID + 62;

// Systems
pub(crate) const ECS_MONITOR: u64 = FLECS_HI_COMPONENT_ID + 63;
pub(crate) const ECS_EMPTY: u64 = FLECS_HI_COMPONENT_ID + 64;
pub(crate) const ECS_PIPELINE: u64 = FLECS_HI_COMPONENT_ID + 65;
pub(crate) const ECS_ON_START: u64 = FLECS_HI_COMPONENT_ID + 66;
pub(crate) const ECS_PRE_FRAME: u64 = FLECS_HI_COMPONENT_ID + 67;
pub(crate) const ECS_ON_LOAD: u64 = FLECS_HI_COMPONENT_ID + 68;
pub(crate) const ECS_POST_LOAD: u64 = FLECS_HI_COMPONENT_ID + 69;
pub(crate) const ECS_PRE_UPDATE: u64 = FLECS_HI_COMPONENT_ID + 70;
pub(crate) const ECS_ON_UPDATE: u64 = FLECS_HI_COMPONENT_ID + 71;
pub(crate) const ECS_ON_VALIDATE: u64 = FLECS_HI_COMPONENT_ID + 72;
pub(crate) const ECS_POST_UPDATE: u64 = FLECS_HI_COMPONENT_ID + 73;
pub(crate) const ECS_PRE_STORE: u64 = FLECS_HI_COMPONENT_ID + 74;
pub(crate) const ECS_ON_STORE: u64 = FLECS_HI_COMPONENT_ID + 75;
pub(crate) const ECS_POST_FRAME: u64 = FLECS_HI_COMPONENT_ID + 76;
pub(crate) const ECS_PHASE: u64 = FLECS_HI_COMPONENT_ID + 77;

// Meta primitive components (don't use low ids to save id space)
pub(crate) const ECS_BOOL_T: u64 = FLECS_HI_COMPONENT_ID + 80;
pub(crate) const ECS_CHAR_T: u64 = FLECS_HI_COMPONENT_ID + 81;
pub(crate) const ECS_BYTE_T: u64 = FLECS_HI_COMPONENT_ID + 82;
pub(crate) const ECS_U8_T: u64 = FLECS_HI_COMPONENT_ID + 83;
pub(crate) const ECS_U16_T: u64 = FLECS_HI_COMPONENT_ID + 84;
pub(crate) const ECS_U32_T: u64 = FLECS_HI_COMPONENT_ID + 85;
pub(crate) const ECS_U64_T: u64 = FLECS_HI_COMPONENT_ID + 86;
pub(crate) const ECS_UPTR_T: u64 = FLECS_HI_COMPONENT_ID + 87;
pub(crate) const ECS_I8_T: u64 = FLECS_HI_COMPONENT_ID + 88;
pub(crate) const ECS_I16_T: u64 = FLECS_HI_COMPONENT_ID + 89;
pub(crate) const ECS_I32_T: u64 = FLECS_HI_COMPONENT_ID + 90;
pub(crate) const ECS_I64_T: u64 = FLECS_HI_COMPONENT_ID + 91;
pub(crate) const ECS_IPTR_T: u64 = FLECS_HI_COMPONENT_ID + 92;
pub(crate) const ECS_F32_T: u64 = FLECS_HI_COMPONENT_ID + 93;
pub(crate) const ECS_F64_T: u64 = FLECS_HI_COMPONENT_ID + 94;
pub(crate) const ECS_STRING_T: u64 = FLECS_HI_COMPONENT_ID + 95;
pub(crate) const ECS_ENTITY_T: u64 = FLECS_HI_COMPONENT_ID + 96;
pub(crate) const ECS_ID_T: u64 = FLECS_HI_COMPONENT_ID + 97;

// Meta module component ids
pub(crate) const ECS_META_TYPE: u64 = FLECS_HI_COMPONENT_ID + 98;
pub(crate) const ECS_META_TYPE_SERIALIZED: u64 = FLECS_HI_COMPONENT_ID + 99;
pub(crate) const ECS_PRIMITIVE: u64 = FLECS_HI_COMPONENT_ID + 100;
pub(crate) const ECS_ENUM: u64 = FLECS_HI_COMPONENT_ID + 101;
pub(crate) const ECS_BITMASK: u64 = FLECS_HI_COMPONENT_ID + 102;
pub(crate) const ECS_MEMBER: u64 = FLECS_HI_COMPONENT_ID + 103;
pub(crate) const ECS_MEMBER_RANGES: u64 = FLECS_HI_COMPONENT_ID + 104;
pub(crate) const ECS_STRUCT: u64 = FLECS_HI_COMPONENT_ID + 105;
pub(crate) const ECS_ARRAY: u64 = FLECS_HI_COMPONENT_ID + 106;
pub(crate) const ECS_VECTOR: u64 = FLECS_HI_COMPONENT_ID + 107;
pub(crate) const ECS_OPAQUE: u64 = FLECS_HI_COMPONENT_ID + 108;
pub(crate) const ECS_UNIT: u64 = FLECS_HI_COMPONENT_ID + 109;
pub(crate) const ECS_UNIT_PREFIX: u64 = FLECS_HI_COMPONENT_ID + 110;
pub(crate) const ECS_CONSTANT: u64 = FLECS_HI_COMPONENT_ID + 111;
pub(crate) const ECS_QUANTITY: u64 = FLECS_HI_COMPONENT_ID + 112;

// Doc module components
pub(crate) const ECS_DOC_DESCRIPTION: u64 = FLECS_HI_COMPONENT_ID + 113;
pub(crate) const ECS_DOC_BRIEF: u64 = FLECS_HI_COMPONENT_ID + 114;
pub(crate) const ECS_DOC_DETAIL: u64 = FLECS_HI_COMPONENT_ID + 115;
pub(crate) const ECS_DOC_LINK: u64 = FLECS_HI_COMPONENT_ID + 116;
pub(crate) const ECS_DOC_COLOR: u64 = FLECS_HI_COMPONENT_ID + 117;

// REST module components
pub(crate) const ECS_REST: u64 = FLECS_HI_COMPONENT_ID + 118;

macro_rules! impl_component_traits_binding_type_w_id {
    ($name:ident, $id:ident) => {
        impl FlecsConstantId for $name {
            const ID: u64 = $id;
        }
        impl DataComponent for $name {}

        impl ComponentType<flecs_ecs::core::Struct> for $name {}

        impl ComponentInfo for $name {
            const IS_GENERIC: bool = false;
            const IS_ENUM: bool = false;
            const IS_TAG: bool = false;
            type TagType = FlecsFirstIsNotATag;
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
        }
        impl ComponentId for $name {
            type UnderlyingType = $name;
            type UnderlyingEnumType = NoneEnum;

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: std::sync::atomic::AtomicU32 =
                    std::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
            }
            fn __register_lifecycle_hooks(type_hooks: &mut TypeHooksT) {
                register_lifecycle_actions::<$name>(type_hooks);
            }
            fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

            fn __register_clone_hooks(type_hooks: &mut TypeHooksT) {
                register_copy_lifecycle_action::<$name>(type_hooks);
            }

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl IntoWorld<'a>,
            ) -> EntityT {
                $id
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl IntoWorld<'a>,
                _name: &str,
            ) -> EntityT {
                $id
            }

            fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
                true
            }

            fn id<'a>(_world: impl IntoWorld<'a>) -> IdT {
                $id
            }
        }
    };
}

pub(crate) use impl_component_traits_binding_type_w_id;
