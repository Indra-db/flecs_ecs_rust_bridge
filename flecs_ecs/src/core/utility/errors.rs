#![doc(hidden)]
use std::fmt::{Display, Formatter};

#[doc(hidden)]
/// Enum representing the error codes that can be used by `ecs_asserts` and `ecs_abort`
pub enum FlecsErrorCode {
    InvalidOperation,
    InvalidParameter,
    ConstraintViolated,
    OutOfMemory,
    OutOfRange,
    Unsupported,
    InternalError,
    AlreadyDefined,
    MissingOsApi,
    OperationFailed,
    InvalidConversion,
    IdInUse,
    CycleDetected,
    LeakDetected,
    InconsistentName,
    NameInUse,
    NotAComponent,
    InvalidComponentSize,
    InvalidComponentAlignment,
    ComponentNotRegistered,
    InconsistentComponentId,
    InconsistentComponentAction,
    ModuleUndefined,
    MissingSymbol,
    AlreadyInUse,
    AccessViolation,
    ColumnIndexOutOfRange,
    ColumnIsNotShared,
    ColumnIsShared,
    ColumnTypeMismatch,
    InvalidWhileReadonly,
    LockedStorage,
    InvalidFromWorker,
    UnwrapFailed,
}

impl Display for FlecsErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FlecsErrorCode::InvalidOperation => "ECS_INVALID_OPERATION",
                FlecsErrorCode::InvalidParameter => "ECS_INVALID_PARAMETER",
                FlecsErrorCode::ConstraintViolated => "ECS_CONSTRAINT_VIOLATED",
                FlecsErrorCode::OutOfMemory => "ECS_OUT_OF_MEMORY",
                FlecsErrorCode::OutOfRange => "ECS_OUT_OF_RANGE",
                FlecsErrorCode::Unsupported => "ECS_UNSUPPORTED",
                FlecsErrorCode::InternalError => "ECS_INTERNAL_ERROR",
                FlecsErrorCode::AlreadyDefined => "ECS_ALREADY_DEFINED",
                FlecsErrorCode::MissingOsApi => "ECS_MISSING_OS_API",
                FlecsErrorCode::OperationFailed => "ECS_OPERATION_FAILED",
                FlecsErrorCode::InvalidConversion => "ECS_INVALID_CONVERSION",
                FlecsErrorCode::IdInUse => "ecs_id_IN_USE",
                FlecsErrorCode::CycleDetected => "ECS_CYCLE_DETECTED",
                FlecsErrorCode::LeakDetected => "ECS_LEAK_DETECTED",
                FlecsErrorCode::InconsistentName => "ECS_INCONSISTENT_NAME",
                FlecsErrorCode::NameInUse => "ECS_NAME_IN_USE",
                FlecsErrorCode::NotAComponent => "ECS_NOT_A_COMPONENT",
                FlecsErrorCode::InvalidComponentSize => "ECS_INVALID_COMPONENT_SIZE",
                FlecsErrorCode::InvalidComponentAlignment => "ECS_INVALID_COMPONENT_ALIGNMENT",
                FlecsErrorCode::ComponentNotRegistered => "ECS_COMPONENT_NOT_REGISTERED",
                FlecsErrorCode::InconsistentComponentId => "ECS_INCONSISTENT_COMPONENT_ID",
                FlecsErrorCode::InconsistentComponentAction => "ECS_INCONSISTENT_COMPONENT_ACTION",
                FlecsErrorCode::ModuleUndefined => "ECS_MODULE_UNDEFINED",
                FlecsErrorCode::MissingSymbol => "ECS_MISSING_SYMBOL",
                FlecsErrorCode::AlreadyInUse => "ECS_ALREADY_IN_USE",
                FlecsErrorCode::AccessViolation => "ECS_ACCESS_VIOLATION",
                FlecsErrorCode::ColumnIndexOutOfRange => "ECS_COLUMN_INDEX_OUT_OF_RANGE",
                FlecsErrorCode::ColumnIsNotShared => "ECS_COLUMN_IS_NOT_SHARED",
                FlecsErrorCode::ColumnIsShared => "ECS_COLUMN_IS_SHARED",
                FlecsErrorCode::ColumnTypeMismatch => "ECS_COLUMN_TYPE_MISMATCH",
                FlecsErrorCode::InvalidWhileReadonly => "ECS_INVALID_WHILE_READONLY",
                FlecsErrorCode::LockedStorage => "ECS_LOCKED_STORAGE",
                FlecsErrorCode::InvalidFromWorker => "ECS_INVALID_FROM_WORKER",
                FlecsErrorCode::UnwrapFailed => "ECS_UNWRAP_FAILED",
            }
        )
    }
}

impl FlecsErrorCode {
    pub fn to_int(&self) -> i32 {
        match self {
            FlecsErrorCode::InvalidOperation => 1,
            FlecsErrorCode::InvalidParameter => 2,
            FlecsErrorCode::ConstraintViolated => 3,
            FlecsErrorCode::OutOfMemory => 4,
            FlecsErrorCode::OutOfRange => 5,
            FlecsErrorCode::Unsupported => 6,
            FlecsErrorCode::InternalError => 7,
            FlecsErrorCode::AlreadyDefined => 8,
            FlecsErrorCode::MissingOsApi => 9,
            FlecsErrorCode::OperationFailed => 10,
            FlecsErrorCode::InvalidConversion => 11,
            FlecsErrorCode::IdInUse => 12,
            FlecsErrorCode::CycleDetected => 13,
            FlecsErrorCode::LeakDetected => 14,
            FlecsErrorCode::InconsistentName => 20,
            FlecsErrorCode::NameInUse => 21,
            FlecsErrorCode::NotAComponent => 22,
            FlecsErrorCode::InvalidComponentSize => 23,
            FlecsErrorCode::InvalidComponentAlignment => 24,
            FlecsErrorCode::ComponentNotRegistered => 25,
            FlecsErrorCode::InconsistentComponentId => 26,
            FlecsErrorCode::InconsistentComponentAction => 27,
            FlecsErrorCode::ModuleUndefined => 28,
            FlecsErrorCode::MissingSymbol => 29,
            FlecsErrorCode::AlreadyInUse => 30,
            FlecsErrorCode::AccessViolation => 40,
            FlecsErrorCode::ColumnIndexOutOfRange => 41,
            FlecsErrorCode::ColumnIsNotShared => 42,
            FlecsErrorCode::ColumnIsShared => 43,
            FlecsErrorCode::ColumnTypeMismatch => 45,
            FlecsErrorCode::InvalidWhileReadonly => 70,
            FlecsErrorCode::LockedStorage => 71,
            FlecsErrorCode::InvalidFromWorker => 72,
            FlecsErrorCode::UnwrapFailed => 73,
        }
    }
}

/// Macro to assert a condition.
/// In release mode, the condition is not checked.
/// Can be turned off by disabling the `flecs_ecs_asserts` feature
#[cfg(any(debug_assertions, feature = "flecs_force_enable_ecs_asserts"))]
macro_rules! ecs_assert {
    ($condition:expr $(,)?, $error_code:expr $(,)?) => {
        assert!($condition, "{}", $error_code);
    };
    ($condition:expr $(,)?, $error_code:expr, $msg:expr $(,)?) => {
        assert!($condition, "{}: {}", $error_code, $msg);
    };
    ($condition:expr $(,)?, $error_code:expr, $arg:ident: *const c_char $(,)?) => {
        assert!($condition, "{}: {}", $error_code,
            if $arg.is_null() {
                "<null>"
            } else {
                unsafe { std::ffi::CStr::from_ptr($arg).to_str().unwrap_or("<invalid>") }
            }
        );
    };
    ($condition:expr $(,)?, $error_code:expr, $fmt:expr, $($arg:tt)+) => {
        assert!($condition, "{}: {}", $error_code, format!($fmt, $($arg)+));
    };
}

#[cfg(all(not(debug_assertions), not(feature = "flecs_force_enable_ecs_asserts")))]
macro_rules! ecs_assert {
    ($($args:tt)*) => {};
}

/// Macro to abort the application when an error occurs.
#[allow(unused_macros)]
macro_rules! ecs_abort {
    ($error_code:expr $(,)?) => {
        let file = file!();
        let line = line!();

        eprintln!("{}:{}: {}", file, line, $error_code);

        //unsafe {
        //    if let Some(abort_func) = ecs_os_api.abort_ {
        //        abort_func();
        //    }
        //};

        std::process::abort();
    };
    ($error_code:expr, $msg:expr $(,)?) => {
        eprintln!("{}: {}", $error_code, $msg);
        //unsafe {
        //    if let Some(abort_func) = ecs_os_api.abort_ {
        //        abort_func();
        //    }
        //};
        std::process::abort();
    };
    ($error_code:expr, $arg:ident: *const c_char $(,)?) => {
        eprintln!("{}: {}",
            $error_code,
            if $arg.is_null() {
                "<null>"
            } else {
                unsafe { CStr::from_ptr($arg).to_str().unwrap_or("<invalid>") }
            }
        );
        //unsafe {
        //    if let Some(abort_func) = ecs_os_api.abort_ {
        //        abort_func();
        //    }
        //};
        std::process::abort();
    };
    ($error_code:expr, $fmt:expr, $($arg:tt)+) => {
        eprintln!("{}: {}", $error_code, format!($fmt, $($arg)+));
        //unsafe {
        //    if let Some(abort_func) = ecs_os_api.abort_ {
        //        abort_func();
        //    }
        //};
        std::process::abort();
    };
}

#[allow(unused_imports)]
pub(crate) use ecs_abort;
pub(crate) use ecs_assert;
