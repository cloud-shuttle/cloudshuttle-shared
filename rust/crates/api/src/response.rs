//! Standardized API response formatting

use serde::{Deserialize, Serialize};
use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};
use uuid::Uuid;

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
    pub errors: Option<Vec<String>>,
    pub request_id: Option<String>,
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
            errors: None,
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a success response with message
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(message.into()),
            data: Some(data),
            errors: None,
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse::<()> {
            success: false,
            message: Some(message.into()),
            data: None,
            errors: None,
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an error response with details
    pub fn error_with_details(message: impl Into<String>, errors: Vec<String>) -> ApiResponse<()> {
        ApiResponse::<()> {
            success: false,
            message: Some(message.into()),
            data: None,
            errors: Some(errors),
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the data if successful
    pub fn into_data(self) -> Option<T> {
        self.data
    }
}

/// Type alias for common API result
pub type ApiResult<T> = Result<ApiResponse<T>, ApiResponse<()>>;

/// Convert ApiResponse to Axum response
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };

        (status, Json(self)).into_response()
    }
}

/// Empty response for operations that don't return data
pub type EmptyResponse = ApiResponse<()>;

impl EmptyResponse {
    pub fn ok() -> Self {
        Self::success(())
    }

    pub fn created() -> Self {
        Self::success_with_message((), "Resource created successfully")
    }

    pub fn updated() -> Self {
        Self::success_with_message((), "Resource updated successfully")
    }

    pub fn deleted() -> Self {
        Self::success_with_message((), "Resource deleted successfully")
    }
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub request_id: Option<String>,
    pub processing_time_ms: Option<u64>,
    pub api_version: Option<String>,
}

impl ResponseMeta {
    pub fn new() -> Self {
        Self {
            request_id: None,
            processing_time_ms: None,
            api_version: None,
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_processing_time(mut self, time_ms: u64) -> Self {
        self.processing_time_ms = Some(time_ms);
        self
    }

    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }
}

/// Enhanced response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedResponse<T> {
    #[serde(flatten)]
    pub response: ApiResponse<T>,
    pub meta: ResponseMeta,
}

impl<T> EnhancedResponse<T> {
    pub fn success(data: T, meta: ResponseMeta) -> Self {
        Self {
            response: ApiResponse::success(data),
            meta,
        }
    }

    pub fn error(message: impl Into<String>, meta: ResponseMeta) -> EnhancedResponse<()> {
        EnhancedResponse::<()> {
            response: ApiResponse::error(message),
            meta,
        }
    }
}

impl<T: Serialize> IntoResponse for EnhancedResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.response.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };

        (status, Json(self)).into_response()
    }
}

/// Response builder pattern
pub struct ResponseBuilder<T> {
    response: ApiResponse<T>,
}

impl<T> ResponseBuilder<T> {
    pub fn success(data: T) -> Self {
        Self {
            response: ApiResponse::success(data),
        }
    }

    pub fn error(message: impl Into<String>) -> ResponseBuilder<()> {
        ResponseBuilder::<()> {
            response: ApiResponse::error(message),
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.response.message = Some(message.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.response.request_id = Some(request_id.into());
        self
    }

    pub fn build(self) -> ApiResponse<T> {
        self.response
    }
}

/// HTTP status code helpers
pub struct StatusHelper;

impl StatusHelper {
    pub fn status_from_result<T>(result: &Result<T, impl std::error::Error>) -> StatusCode {
        match result {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn is_success_status(code: StatusCode) -> bool {
        code.is_success()
    }

    pub fn is_error_status(code: StatusCode) -> bool {
        code.is_client_error() || code.is_server_error()
    }
}
