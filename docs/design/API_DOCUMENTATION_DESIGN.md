# API Documentation Design Document

## Overview

This document outlines the design for API documentation generation and management in the CloudShuttle Gateway, providing comprehensive endpoint documentation for developers and automated API discovery.

## Current Implementation

### Basic Documentation Structure

```rust
pub fn create_api_docs(&self) -> Json<ApiResponse<serde_json::Value>> {
    let docs = serde_json::json!({
        "title": "CloudShuttle Proxy API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Management and monitoring API for CloudShuttle proxy",
        "endpoints": [
            {
                "path": "/health",
                "method": "GET",
                "description": "Health check endpoint"
            },
            {
                "path": "/metrics",
                "method": "GET",
                "description": "Metrics endpoint"
            },
            {
                "path": "/config",
                "method": "GET",
                "description": "Configuration endpoint"
            }
        ]
    });

    self.create_success_response(docs)
}
```

### Response Format

```json
{
  "success": true,
  "data": {
    "title": "CloudShuttle Proxy API",
    "version": "0.1.0",
    "description": "Management and monitoring API for CloudShuttle proxy",
    "endpoints": [
      {
        "path": "/health",
        "method": "GET",
        "description": "Health check endpoint"
      }
    ]
  },
  "error": null,
  "timestamp": "2024-01-01T12:00:00.000Z"
}
```

## Future Enhancement: Structured Documentation

When cloudshuttle-api provides documentation types:

```rust
pub fn create_api_docs(&self) -> Json<ApiResponse<ApiDocumentation>> {
    let docs = ApiDocumentation::new("CloudShuttle Proxy API")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_description("Management and monitoring API for CloudShuttle proxy")
        .with_base_url("/api/v1")
        .with_authentication(AuthScheme::Bearer)
        .with_server("https://api.cloudshuttle.dev", "Production")
        .with_server("http://localhost:8080", "Development")
        .with_endpoint(
            Endpoint::new("/health", Method::GET)
                .with_description("Health check endpoint")
                .with_summary("Check service health status")
                .with_tags(vec!["health", "monitoring"])
                .with_response(StatusCode::OK, "HealthStatus")
                .with_example_response(HealthStatus {
                    status: "healthy".to_string(),
                    uptime_seconds: 3600,
                    version: "1.0.0".to_string(),
                })
        )
        .with_endpoint(
            Endpoint::new("/upstreams", Method::GET)
                .with_description("List upstream servers")
                .with_auth_required(true)
                .with_parameters(vec![
                    Parameter::query("page", "Page number", Type::Integer)
                        .with_default(1)
                        .with_minimum(1),
                    Parameter::query("limit", "Items per page", Type::Integer)
                        .with_default(50)
                        .with_minimum(1)
                        .with_maximum(1000),
                ])
                .with_response(StatusCode::OK, "PaginatedResponse<UpstreamInfo>")
                .with_response(StatusCode::UNAUTHORIZED, "Error")
                .with_tags(vec!["upstreams", "management"])
        )
        .build();

    self.create_success_response(docs)
}
```

### Enhanced Documentation Types

```rust
#[derive(Debug, Serialize)]
pub struct ApiDocumentation {
    pub openapi: String,  // "3.0.3"
    pub info: ApiInfo,
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
    pub components: Components,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
pub struct ApiInfo {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
    pub contact: Option<Contact>,
    pub license: Option<License>,
}

#[derive(Debug, Serialize)]
pub struct Endpoint {
    pub path: String,
    pub method: HttpMethod,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub tags: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    pub security: Vec<SecurityRequirement>,
    pub deprecated: bool,
}

#[derive(Debug, Serialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,  // query, header, path, cookie
    pub description: Option<String>,
    pub required: bool,
    pub schema: Schema,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
    pub headers: Option<HashMap<String, Header>>,
}

#[derive(Debug, Serialize)]
pub struct Schema {
    pub type_: Option<String>,
    pub format: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub items: Option<Box<Schema>>,
    pub required: Vec<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub enum_: Option<Vec<serde_json::Value>>,
}
```

## Documentation Generation

### Automatic Endpoint Discovery

```rust
pub struct ApiDocumentationBuilder {
    endpoints: Vec<Endpoint>,
    schemas: HashMap<String, Schema>,
    base_url: String,
}

impl ApiDocumentationBuilder {
    pub fn discover_endpoints(&mut self) {
        // Scan route definitions and extract endpoint information
        // This could be done at compile time with procedural macros
        // or at runtime by inspecting the router

        self.add_endpoint(Endpoint {
            path: "/health",
            method: Method::GET,
            summary: Some("Health Check".to_string()),
            description: Some("Check the health status of the service".to_string()),
            tags: vec!["health".to_string()],
            responses: self.build_health_responses(),
            ..Default::default()
        });
    }

    pub fn generate_openapi_spec(&self) -> serde_json::Value {
        json!({
            "openapi": "3.0.3",
            "info": {
                "title": "CloudShuttle Proxy API",
                "version": env!("CARGO_PKG_VERSION"),
                "description": "Management and monitoring API"
            },
            "paths": self.build_paths_object(),
            "components": {
                "schemas": self.schemas,
                "securitySchemes": self.build_security_schemes()
            }
        })
    }
}
```

### Schema Generation

```rust
pub trait ApiSchema {
    fn to_schema() -> Schema;
}

impl ApiSchema for HealthStatus {
    fn to_schema() -> Schema {
        Schema {
            type_: Some("object".to_string()),
            properties: Some(HashMap::from([
                ("status".to_string(), Schema {
                    type_: Some("string".to_string()),
                    enum_: Some(vec![
                        json!("healthy"),
                        json!("degraded"),
                        json!("unhealthy")
                    ]),
                    ..Default::default()
                }),
                ("uptime_seconds".to_string(), Schema {
                    type_: Some("integer".to_string()),
                    minimum: Some(0.0),
                    ..Default::default()
                }),
                ("version".to_string(), Schema {
                    type_: Some("string".to_string()),
                    ..Default::default()
                }),
            ])),
            required: vec!["status".to_string(), "uptime_seconds".to_string()],
            ..Default::default()
        }
    }
}

impl ApiSchema for UpstreamInfo {
    fn to_schema() -> Schema {
        Schema {
            type_: Some("object".to_string()),
            properties: Some(HashMap::from([
                ("id".to_string(), Schema {
                    type_: Some("string".to_string()),
                    ..Default::default()
                }),
                ("name".to_string(), Schema {
                    type_: Some("string".to_string()),
                    ..Default::default()
                }),
                ("address".to_string(), Schema {
                    type_: Some("string".to_string()),
                    ..Default::default()
                }),
                ("weight".to_string(), Schema {
                    type_: Some("integer".to_string()),
                    minimum: Some(0.0),
                    ..Default::default()
                }),
            ])),
            required: vec!["id".to_string(), "name".to_string(), "address".to_string()],
            ..Default::default()
        }
    }
}
```

### Compile-Time Documentation

```rust
// Procedural macro for automatic documentation generation
#[derive(ApiDocument)]
#[api(path = "/health", method = "GET")]
struct HealthCheck;

#[derive(ApiDocument)]
#[api(
    path = "/upstreams",
    method = "GET",
    auth_required = true,
    tags = ["upstreams", "management"]
)]
struct ListUpstreams {
    #[api_param(query, name = "page", type = "integer", minimum = 1)]
    page: Option<usize>,

    #[api_param(query, name = "limit", type = "integer", minimum = 1, maximum = 1000)]
    limit: Option<usize>,
}

// Generated code would include:
impl ListUpstreams {
    fn documentation() -> Endpoint {
        Endpoint {
            path: "/upstreams".to_string(),
            method: Method::GET,
            tags: vec!["upstreams".to_string(), "management".to_string()],
            parameters: vec![
                Parameter {
                    name: "page".to_string(),
                    location: ParameterLocation::Query,
                    schema: Schema {
                        type_: Some("integer".to_string()),
                        minimum: Some(1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            ],
            security: vec![SecurityRequirement::bearer_auth()],
            ..Default::default()
        }
    }
}
```

## Documentation Serving

### Multiple Formats

```rust
pub enum DocumentationFormat {
    Json,
    Yaml,
    Html,
    Postman,
    OpenApi,
}

pub async fn serve_documentation(
    format: DocumentationFormat,
) -> Result<Response<BoxBody>, ApiError> {
    let docs = generate_api_documentation().await?;

    match format {
        DocumentationFormat::Json => {
            let json = serde_json::to_string(&docs)?;
            Response::builder()
                .header("content-type", "application/json")
                .body(json.into())
        }
        DocumentationFormat::Yaml => {
            let yaml = serde_yaml::to_string(&docs)?;
            Response::builder()
                .header("content-type", "application/yaml")
                .body(yaml.into())
        }
        DocumentationFormat::Html => {
            let html = generate_html_docs(&docs).await?;
            Response::builder()
                .header("content-type", "text/html")
                .body(html.into())
        }
        DocumentationFormat::OpenApi => {
            let spec = docs.to_openapi_spec();
            Response::builder()
                .header("content-type", "application/yaml")
                .body(spec.into())
        }
    }
}
```

### Interactive Documentation

```rust
pub async fn serve_swagger_ui() -> Result<Response<BoxBody>, ApiError> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>CloudShuttle API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.7.2/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.7.2/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: '/api/v1/docs/openapi.yaml',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.presets.standalone
                ]
            });
        };
    </script>
</body>
</html>
    "#;

    Response::builder()
        .header("content-type", "text/html")
        .body(html.into())
}
```

## Documentation Testing

### Contract Testing

```rust
#[cfg(test)]
mod contract_tests {
    use super::*;
    use pact_consumer::*;

    #[tokio::test]
    async fn test_api_contract_compliance() {
        let pact_builder = PactBuilder::new("CloudShuttle Gateway", "API Consumer");

        pact_builder
            .interaction("get health status", |interaction| {
                interaction
                    .given("service is healthy")
                    .upon_receiving("a health check request")
                    .with_request("GET", "/health")
                    .will_respond_with(200)
                    .with_header("content-type", "application/json")
                    .with_body(json!({
                        "success": true,
                        "data": {
                            "status": "healthy",
                            "uptime_seconds": 3600,
                            "version": "1.0.0"
                        }
                    }))
            })
            .await
            .verify()
            .await;
    }
}
```

### Documentation Validation

```rust
#[cfg(test)]
mod documentation_tests {
    use super::*;

    #[tokio::test]
    async fn test_openapi_spec_validity() {
        let docs = generate_api_documentation().await.unwrap();
        let spec = docs.to_openapi_spec();

        // Validate against OpenAPI schema
        let schema: serde_json::Value = serde_json::from_str(include_str!("openapi-schema.json"))?;
        let validator = jsonschema::validator_for(&schema)?;

        let result = validator.validate(&spec);
        assert!(result.is_ok(), "OpenAPI spec is invalid");
    }

    #[test]
    fn test_endpoint_documentation_completeness() {
        let docs = generate_api_documentation().await.unwrap();

        // Check that all registered routes have documentation
        let routes = get_registered_routes();
        for route in routes {
            assert!(
                docs.paths.contains_key(&route.path),
                "Route {} is missing documentation",
                route.path
            );
        }
    }

    #[test]
    fn test_schema_completeness() {
        let docs = generate_api_documentation().await.unwrap();

        // Check that all response types have schemas
        for endpoint in docs.endpoints() {
            for response in &endpoint.responses {
                if let Some(schema_ref) = &response.schema_ref {
                    assert!(
                        docs.components.schemas.contains_key(schema_ref),
                        "Schema {} is missing for endpoint {}",
                        schema_ref,
                        endpoint.path
                    );
                }
            }
        }
    }
}
```

## Version Management

### API Versioning

```rust
#[derive(Debug, Clone)]
pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    pub fn base_path(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "/api/v1",
            ApiVersion::V2 => "/api/v2",
        }
    }

    pub fn is_deprecated(&self) -> bool {
        matches!(self, ApiVersion::V1)
    }
}
```

### Version-Specific Documentation

```rust
pub async fn get_versioned_documentation(
    version: ApiVersion,
) -> Result<ApiDocumentation, ApiError> {
    let mut docs = ApiDocumentation::new("CloudShuttle Proxy API");

    match version {
        ApiVersion::V1 => {
            docs.with_version("1.0.0")
                .with_description("Legacy API v1")
                .add_v1_endpoints();
        }
        ApiVersion::V2 => {
            docs.with_version("2.0.0")
                .with_description("Current API v2")
                .add_v2_endpoints();
        }
    }

    Ok(docs)
}
```

## Security Considerations

### Documentation Access Control

```rust
pub async fn serve_documentation(
    user: Option<UserContext>,
) -> Result<Response<BoxBody>, ApiError> {
    // Check if user has permission to view documentation
    if let Some(user) = &user {
        if !user.permissions.contains(&"docs:read".to_string()) {
            return Err(ApiError::Forbidden);
        }
    }

    // Serve documentation
    let docs = generate_api_documentation().await?;
    Ok(Json(docs).into_response())
}
```

### Sensitive Data Filtering

```rust
impl ApiDocumentation {
    pub fn filter_sensitive_data(mut self) -> Self {
        // Remove sensitive endpoints from public documentation
        self.endpoints.retain(|endpoint| {
            !endpoint.tags.contains(&"internal".to_string()) &&
            !endpoint.path.contains("admin")
        });

        // Remove sensitive parameters
        for endpoint in &mut self.endpoints {
            endpoint.parameters.retain(|param| {
                !param.name.contains("secret") &&
                !param.name.contains("token")
            });
        }

        self
    }
}
```

## Performance Optimization

### Caching

```rust
pub struct DocumentationCache {
    docs: RwLock<Option<ApiDocumentation>>,
    last_updated: AtomicU64,
    ttl: Duration,
}

impl DocumentationCache {
    pub async fn get_or_refresh(&self) -> Result<ApiDocumentation, CacheError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        // Check if cache is still valid
        let last_updated = self.last_updated.load(Ordering::Relaxed);
        if now - last_updated < self.ttl.as_secs() {
            if let Some(docs) = self.docs.read().await.as_ref() {
                return Ok(docs.clone());
            }
        }

        // Refresh cache
        let new_docs = generate_api_documentation().await?;
        *self.docs.write().await = Some(new_docs.clone());
        self.last_updated.store(now, Ordering::Relaxed);

        Ok(new_docs)
    }
}
```

### Lazy Loading

```rust
pub struct LazyDocumentation {
    generator: Box<dyn Fn() -> ApiDocumentation + Send + Sync>,
    cache: OnceCell<ApiDocumentation>,
}

impl LazyDocumentation {
    pub fn new<F>(generator: F) -> Self
    where
        F: Fn() -> ApiDocumentation + Send + Sync + 'static,
    {
        Self {
            generator: Box::new(generator),
            cache: OnceCell::new(),
        }
    }

    pub fn get(&self) -> &ApiDocumentation {
        self.cache.get_or_init(|| (self.generator)())
    }
}
```

## Monitoring and Analytics

### Documentation Metrics

```rust
pub struct DocumentationMetrics {
    pub total_views: Counter,
    pub format_distribution: Histogram,
    pub response_time: Histogram,
    pub error_rate: Counter,
}

impl DocumentationMetrics {
    pub fn record_view(&self, format: DocumentationFormat, response_time: Duration) {
        self.total_views.inc();
        self.format_distribution.observe(format as u8 as f64);
        self.response_time.observe(response_time.as_secs_f64());
    }

    pub fn record_error(&self, error_type: &str) {
        self.error_rate.with_label_values(&[error_type]).inc();
    }
}
```

## Migration Strategy

### Phase 1: Basic Documentation (Current)
- ✅ JSON-based endpoint listing
- ✅ Basic API information
- ✅ Manual endpoint documentation

### Phase 2: Enhanced Documentation
- 🔄 Structured OpenAPI specification
- 🔄 Schema generation
- 🔄 Interactive documentation
- 🔄 Multiple output formats

### Phase 3: Advanced Features
- 🔄 Compile-time documentation generation
- 🔄 Contract testing integration
- 🔄 Version management
- 🔄 Access control

## Conclusion

The API documentation design provides comprehensive, accurate, and accessible documentation for CloudShuttle Gateway APIs. The structured approach ensures consistency, discoverability, and developer experience while supporting multiple formats and use cases.
