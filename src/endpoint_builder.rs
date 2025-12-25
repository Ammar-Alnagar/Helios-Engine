//! # Endpoint Builder Module
//!
//! This module provides a simple and ergonomic way to create custom endpoints
//! for the Helios Engine HTTP server.

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A custom endpoint handler function type.
/// Takes optional request data and returns a response.
pub type EndpointHandler = Arc<dyn Fn(Option<EndpointRequest>) -> EndpointResponse + Send + Sync>;

/// Request data passed to endpoint handlers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRequest {
    /// Query parameters from the URL.
    #[serde(default)]
    pub query: HashMap<String, String>,

    /// Path parameters (e.g., /users/:id -> {"id": "123"}).
    #[serde(default)]
    pub params: HashMap<String, String>,

    /// Request body as JSON (for POST, PUT, PATCH).
    pub body: Option<Value>,
}

/// Response from an endpoint handler.
#[derive(Debug, Clone)]
pub struct EndpointResponse {
    /// HTTP status code.
    pub status: StatusCode,

    /// Response body as JSON.
    pub body: Value,
}

impl EndpointResponse {
    /// Creates a new successful response (200 OK).
    pub fn ok(body: Value) -> Self {
        Self {
            status: StatusCode::OK,
            body,
        }
    }

    /// Creates a new created response (201 CREATED).
    pub fn created(body: Value) -> Self {
        Self {
            status: StatusCode::CREATED,
            body,
        }
    }

    /// Creates a new accepted response (202 ACCEPTED).
    pub fn accepted(body: Value) -> Self {
        Self {
            status: StatusCode::ACCEPTED,
            body,
        }
    }

    /// Creates a new bad request response (400 BAD REQUEST).
    pub fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: serde_json::json!({"error": message}),
        }
    }

    /// Creates a new not found response (404 NOT FOUND).
    pub fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: serde_json::json!({"error": message}),
        }
    }

    /// Creates a new internal server error response (500 INTERNAL SERVER ERROR).
    pub fn internal_error(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: serde_json::json!({"error": message}),
        }
    }

    /// Creates a custom response with a specific status code.
    pub fn with_status(status: StatusCode, body: Value) -> Self {
        Self { status, body }
    }
}

impl IntoResponse for EndpointResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Builder for creating custom endpoints with an ergonomic API.
pub struct EndpointBuilder {
    path: String,
    method: HttpMethod,
    handler: Option<EndpointHandler>,
    description: Option<String>,
}

/// HTTP method for the endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl EndpointBuilder {
    /// Creates a new GET endpoint builder.
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: HttpMethod::Get,
            handler: None,
            description: None,
        }
    }

    /// Creates a new POST endpoint builder.
    pub fn post(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: HttpMethod::Post,
            handler: None,
            description: None,
        }
    }

    /// Creates a new PUT endpoint builder.
    pub fn put(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: HttpMethod::Put,
            handler: None,
            description: None,
        }
    }

    /// Creates a new DELETE endpoint builder.
    pub fn delete(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: HttpMethod::Delete,
            handler: None,
            description: None,
        }
    }

    /// Creates a new PATCH endpoint builder.
    pub fn patch(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: HttpMethod::Patch,
            handler: None,
            description: None,
        }
    }

    /// Sets a static JSON response for the endpoint.
    /// This is the simplest way to create an endpoint that returns fixed data.
    pub fn json(mut self, response: Value) -> Self {
        self.handler = Some(Arc::new(move |_| EndpointResponse::ok(response.clone())));
        self
    }

    /// Sets a handler function that receives request data and returns a response.
    /// This allows for dynamic responses based on query params, path params, and body.
    pub fn handle<F>(mut self, handler: F) -> Self
    where
        F: Fn(Option<EndpointRequest>) -> EndpointResponse + Send + Sync + 'static,
    {
        self.handler = Some(Arc::new(handler));
        self
    }

    /// Sets a description for the endpoint (for documentation purposes).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builds the endpoint.
    pub fn build(self) -> CustomEndpoint {
        CustomEndpoint {
            path: self.path,
            method: self.method,
            handler: self
                .handler
                .expect("Handler must be set with .json() or .handle()"),
            description: self.description,
        }
    }
}

/// A custom endpoint with a handler function.
#[derive(Clone)]
pub struct CustomEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub handler: EndpointHandler,
    pub description: Option<String>,
}

/// Helper function to create a simple GET endpoint with a static JSON response.
///
/// # Example
/// ```
/// use helios_engine::get;
///
/// let endpoint = get("/api/status", serde_json::json!({
///     "status": "ok"
/// }));
/// ```
pub fn get(path: impl Into<String>, response: Value) -> CustomEndpoint {
    EndpointBuilder::get(path).json(response).build()
}

/// Helper function to create a simple POST endpoint with a static JSON response.
pub fn post(path: impl Into<String>, response: Value) -> CustomEndpoint {
    EndpointBuilder::post(path).json(response).build()
}

/// Helper function to create a simple PUT endpoint with a static JSON response.
pub fn put(path: impl Into<String>, response: Value) -> CustomEndpoint {
    EndpointBuilder::put(path).json(response).build()
}

/// Helper function to create a simple DELETE endpoint with a static JSON response.
pub fn delete(path: impl Into<String>, response: Value) -> CustomEndpoint {
    EndpointBuilder::delete(path).json(response).build()
}

/// Helper function to create a simple PATCH endpoint with a static JSON response.
pub fn patch(path: impl Into<String>, response: Value) -> CustomEndpoint {
    EndpointBuilder::patch(path).json(response).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_builder_get() {
        let endpoint = EndpointBuilder::get("/api/test")
            .json(serde_json::json!({"test": "data"}))
            .build();

        assert_eq!(endpoint.path, "/api/test");
        assert_eq!(endpoint.method, HttpMethod::Get);
    }

    #[test]
    fn test_endpoint_response_helpers() {
        let ok_response = EndpointResponse::ok(serde_json::json!({"status": "ok"}));
        assert_eq!(ok_response.status, StatusCode::OK);

        let created_response = EndpointResponse::created(serde_json::json!({"id": 1}));
        assert_eq!(created_response.status, StatusCode::CREATED);

        let not_found = EndpointResponse::not_found("Resource not found");
        assert_eq!(not_found.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_helper_functions() {
        let endpoint = get("/api/version", serde_json::json!({"version": "1.0.0"}));
        assert_eq!(endpoint.path, "/api/version");
        assert_eq!(endpoint.method, HttpMethod::Get);
    }
}
