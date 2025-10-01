# API Contracts & Contract Testing

**Status:** CRITICAL - Missing Implementation
**Timeline:** Implement within Phase 2 (Week 2)
**Standards:** OpenAPI 3.0, Pact Contract Testing

## 🚨 Current State: NO API CONTRACTS

### Critical Gaps:
- ❌ **No API specifications** defined
- ❌ **No contract test framework** implemented
- ❌ **No consumer-driven contracts**
- ❌ **No API versioning strategy**
- ❌ **No schema validation**

### Business Impact:
- Integration failures between services
- Breaking changes without detection
- Poor API documentation
- Difficult testing and debugging

---

## 🎯 Required Implementation

### 1. OpenAPI Specifications

#### File Structure:
```
docs/api-contracts/
├── database-api.yaml     # Database service API
├── auth-api.yaml        # Authentication service API
├── observability-api.yaml # Observability service API
├── api-gateway.yaml     # API Gateway contracts
└── shared-types.yaml    # Common data types
```

#### Example: Database API Contract
```yaml
openapi: 3.0.3
info:
  title: CloudShuttle Database API
  version: 0.2.0
  description: Database operations contract

servers:
  - url: https://api.cloudshuttle.com/database
    description: Production server

paths:
  /health:
    get:
      summary: Database health check
      responses:
        '200':
          description: Database is healthy
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DatabaseHealth'

  /transactions:
    post:
      summary: Execute database transaction
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/TransactionRequest'
      responses:
        '200':
          description: Transaction completed
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TransactionResult'

components:
  schemas:
    DatabaseHealth:
      type: object
      properties:
        status:
          type: string
          enum: [healthy, degraded, unhealthy]
        message:
          type: string
        timestamp:
          type: string
          format: date-time
```

### 2. Contract Testing Framework

#### Implementation Strategy:
```rust
// contracts/tests/database_contract.rs
use pact_consumer::prelude::*;
use cloudshuttle_database::*;

#[tokio::test]
async fn database_health_contract() {
    let pact_builder = PactBuilder::new("CloudShuttle API", "Database Service");

    pact_builder
        .interaction("Database health check", |mut i| {
            i.given("Database is running");
            i.upon_receiving("GET request to /health");
            i.with_request("GET", "/health");
            i.will_respond_with(200)
                .with_header("Content-Type", "application/json")
                .with_body("{\"status\":\"healthy\",\"message\":\"OK\"}");
            i
        })
        .await
        .start_mock_server()
        .await;
}
```

#### Contract Test Structure:
```
contracts/
├── tests/
│   ├── database_contracts.rs
│   ├── auth_contracts.rs
│   ├── observability_contracts.rs
│   └── integration_contracts.rs
├── pacts/
│   ├── cloudshuttle-api-database.json
│   ├── cloudshuttle-api-auth.json
│   └── cloudshuttle-api-observability.json
└── Cargo.toml
```

### 3. Consumer-Driven Contracts

#### Provider States:
```rust
#[pact_provider_state]
async fn database_is_running() -> bool {
    // Setup database with test data
    let db = setup_test_database().await;
    // Return true if setup successful
    true
}
```

#### Consumer Expectations:
```rust
#[tokio::test]
async fn consumer_expects_database_health() {
    let service = PactBuilder::new("API Consumer", "Database Service")
        .interaction("should return health status", |i| {
            i.given("database is running");
            i.upon_receiving("a health check request");
            i.with_request("GET", "/health");
            i.will_respond_with(200)
                .json_body(json!({
                    "status": "healthy",
                    "timestamp": like!("2025-09-20T10:00:00Z")
                }));
        })
        .build();
}
```

### 4. API Versioning Strategy

#### Semantic Versioning:
- **MAJOR.MINOR.PATCH** (e.g., 0.2.0)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

#### Version Headers:
```rust
// Request versioning
GET /api/v1/database/health
Accept: application/vnd.cloudshuttle.v1+json

// Response versioning
{
  "version": "1.0",
  "data": { ... }
}
```

---

## 🔧 Implementation Plan

### Phase 1: Contract Definition (Week 1)
1. [ ] Define OpenAPI specifications for all services
2. [ ] Create shared schema definitions
3. [ ] Document API versioning strategy

### Phase 2: Contract Testing Setup (Week 2)
1. [ ] Add Pact dependency for contract testing
2. [ ] Implement provider contract tests
3. [ ] Implement consumer contract tests

### Phase 3: Integration & Validation (Week 3)
1. [ ] Run contract tests in CI/CD
2. [ ] Validate API compliance
3. [ ] Generate API documentation

---

## 📋 Verification Checklist

### API Contract Completeness:
- [ ] All public APIs documented in OpenAPI
- [ ] Request/response schemas defined
- [ ] Error responses documented
- [ ] Authentication requirements specified

### Contract Testing Coverage:
- [ ] Happy path scenarios covered
- [ ] Error scenarios tested
- [ ] Edge cases documented
- [ ] Consumer expectations validated

### CI/CD Integration:
- [ ] Contract tests run automatically
- [ ] Pact broker integration (if applicable)
- [ ] Contract validation on deployment

---

## ✅ Success Criteria

### Technical Requirements:
- [ ] All APIs have OpenAPI specifications
- [ ] Contract tests pass for all services
- [ ] API documentation auto-generated
- [ ] Breaking changes detected automatically

### Quality Requirements:
- [ ] Consumer-driven contract development
- [ ] Comprehensive error scenario testing
- [ ] API versioning strategy implemented
- [ ] Schema validation active

### Operational Requirements:
- [ ] Contract tests in CI/CD pipeline
- [ ] API documentation published
- [ ] Contract validation on releases
- [ ] Consumer notification of changes

---

## 🛠️ Tools & Frameworks

### Contract Testing:
- **Pact**: Consumer-driven contract testing
- **Spring Cloud Contract**: Alternative framework
- **OpenAPI Generator**: API client/server generation

### API Documentation:
- **Swagger UI**: Interactive API documentation
- **Redoc**: Alternative documentation generator
- **Spectral**: OpenAPI linting

### Schema Validation:
- **JSON Schema**: Request/response validation
- **OpenAPI Schema**: API specification validation
- **Serde**: Rust type validation

---

## 📊 Metrics & Monitoring

### Contract Test Metrics:
- Contract test pass rate: >99%
- API compatibility score: >95%
- Breaking change detection: 100%

### API Quality Metrics:
- OpenAPI compliance: 100%
- Schema validation: 100%
- Documentation coverage: 100%

---

## 🚨 Risk Assessment

### High Risk Items:
1. **API Breaking Changes**: Risk of service integration failures
2. **Missing Contract Coverage**: Risk of undetected incompatibilities
3. **Documentation Drift**: Risk of outdated API docs

### Mitigation Strategies:
- **Comprehensive contract testing** before releases
- **Automated compatibility checking** in CI/CD
- **API versioning** to manage breaking changes
- **Consumer notification system** for API changes
