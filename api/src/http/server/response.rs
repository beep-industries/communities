use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
};
use communities_core::domain::common::TotalPaginatedElements;
use serde::Serialize;
use utoipa::ToSchema;

/// Generic response wrapper for consistent API responses
#[derive(Debug, Clone)]
pub struct Response<T> {
    data: T,
    status_code: StatusCode,
}

impl<T> Response<T>
where
    T: Serialize,
{
    /// Create a 200 OK response
    pub fn ok(data: T) -> Self {
        Self {
            data,
            status_code: StatusCode::OK,
        }
    }

    /// Create a 201 CREATED response
    pub fn created(data: T) -> Self {
        Self {
            data,
            status_code: StatusCode::CREATED,
        }
    }

    /// Create a 200 OK response
    pub fn deleted(data: T) -> Self {
        Self {
            data,
            status_code: StatusCode::OK,
        }
    }

    /// Create a response with a custom status code
    #[allow(dead_code)]
    pub fn with_status(data: T, status_code: StatusCode) -> Self {
        Self { data, status_code }
    }
}

impl<T> IntoResponse for Response<T>
where
    T: Serialize,
{
    fn into_response(self) -> AxumResponse {
        (self.status_code, Json(self.data)).into_response()
    }
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: TotalPaginatedElements,
    pub page: u32,
}

impl<T: Serialize> Into<Response<PaginatedResponse<T>>> for PaginatedResponse<T> {
    fn into(self) -> Response<PaginatedResponse<T>> {
        Response::ok(self)
    }
}
