//! Error handling macros for CloudShuttle services

/// Macro to create a service error
#[macro_export]
macro_rules! service_error {
    ($code:expr, $status:expr, $message:expr) => {
        $crate::service_error::StandardServiceError::new($code, $status, $message)
    };
    ($code:expr, $status:expr, $message:expr, internal: $internal:expr) => {
        $crate::service_error::StandardServiceError::new($code, $status, $message)
            .with_internal_message($internal)
    };
    ($code:expr, $status:expr, $message:expr, details: $details:expr) => {
        $crate::service_error::StandardServiceError::new($code, $status, $message)
            .with_details($details)
    };
}

/// Macro to convert an error into a CloudShuttleError
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
}

/// Macro to create and return an error
#[macro_export]
macro_rules! bail_with {
    ($variant:ident, $($args:tt)*) => {
        return Err($crate::error::CloudShuttleError::$variant($($args)*));
    };
}

/// Macro to ensure a condition is true, otherwise return an error
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $err:expr) => {
        if !($condition) {
            bail!($err);
        }
    };
}

/// Macro to ensure a condition is true, otherwise return an error with a variant
#[macro_export]
macro_rules! ensure_with {
    ($condition:expr, $variant:ident, $($args:tt)*) => {
        if !($condition) {
            bail_with!($variant, $($args)*);
        }
    };
}

/// Macro to wrap database operations and convert errors
#[macro_export]
macro_rules! db_op {
    ($operation:expr) => {
        match $operation {
            Ok(result) => Ok(result),
            Err(err) => Err($crate::database_error::DatabaseError::from(err).into()),
        }
    };
}

/// Macro to handle optional database results
#[macro_export]
macro_rules! db_find {
    ($operation:expr, $resource:expr) => {
        match $operation {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err($crate::database_error::DatabaseError::not_found($resource).into()),
            Err(err) => Err($crate::database_error::DatabaseError::from(err).into()),
        }
    };
}

/// Macro to create API errors
#[macro_export]
macro_rules! api_error {
    (bad_request, $message:expr) => {
        $crate::api_error::ApiError::bad_request($message)
    };
    (unauthorized, $message:expr) => {
        $crate::api_error::ApiError::unauthorized($message)
    };
    (forbidden, $message:expr) => {
        $crate::api_error::ApiError::forbidden($message)
    };
    (not_found, $resource:expr) => {
        $crate::api_error::ApiError::not_found($resource)
    };
    (conflict, $message:expr) => {
        $crate::api_error::ApiError::conflict($message)
    };
    (unprocessable, $message:expr) => {
        $crate::api_error::ApiError::unprocessable_entity($message)
    };
    (internal, $message:expr) => {
        $crate::api_error::ApiError::internal_server_error($message)
    };
    (unavailable, $message:expr) => {
        $crate::api_error::ApiError::service_unavailable($message)
    };
}

/// Macro to validate required fields
#[macro_export]
macro_rules! validate_required {
    ($field:expr, $field_name:expr) => {
        if $field.trim().is_empty() {
            bail_with!(Validation, validator::ValidationErrors::new());
        }
    };
}

/// Macro to validate field length
#[macro_export]
macro_rules! validate_length {
    ($field:expr, $field_name:expr, min: $min:expr) => {
        if $field.len() < $min {
            bail_with!(Validation, validator::ValidationErrors::new());
        }
    };
    ($field:expr, $field_name:expr, max: $max:expr) => {
        if $field.len() > $max {
            bail_with!(Validation, validator::ValidationErrors::new());
        }
    };
    ($field:expr, $field_name:expr, min: $min:expr, max: $max:expr) => {
        if $field.len() < $min || $field.len() > $max {
            bail_with!(Validation, validator::ValidationErrors::new());
        }
    };
}
