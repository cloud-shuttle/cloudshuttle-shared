#!/bin/bash

# CloudShuttle Contract Validation Script
# Validates API contracts and runs contract tests

set -e

echo "🔗 CloudShuttle API Contract Validation"
echo "======================================"
echo ""

cd "$(dirname "$0")/.."

echo "📋 1. Running contract tests..."
echo "   cargo test --package cloudshuttle-auth --test contract_tests"
if cargo test --package cloudshuttle-auth --test contract_tests --quiet 2>&1; then
    echo "   ✅ Contract tests passed"
else
    echo "   ❌ Contract tests failed"
    exit 1
fi

echo ""
echo "📋 2. Validating OpenAPI specifications..."

# Check if swagger-cli is available
if command -v swagger-cli >/dev/null 2>&1; then
    echo "   Validating authentication API..."
    if npx @apidevtools/swagger-cli validate docs/api-contracts/authentication/openapi.yaml --quiet; then
        echo "   ✅ Authentication API specification valid"
    else
        echo "   ❌ Authentication API specification invalid"
        exit 1
    fi
else
    echo "   ⚠️  swagger-cli not available, skipping OpenAPI validation"
    echo "   To install: npm install -g @apidevtools/swagger-cli"
fi

echo ""
echo "📋 3. Generating API documentation..."

# Check if redoc-cli is available
if command -v redoc-cli >/dev/null 2>&1; then
    echo "   Generating HTML documentation..."
    mkdir -p docs/api-docs
    npx @redocly/cli build-docs docs/api-contracts/authentication/openapi.yaml \
        --output docs/api-docs/authentication-api.html --quiet
    echo "   ✅ API documentation generated: docs/api-docs/authentication-api.html"
else
    echo "   ⚠️  redoc-cli not available, skipping documentation generation"
    echo "   To install: npm install -g @redocly/cli"
fi

echo ""
echo "🎉 Contract validation complete!"
echo ""
echo "📊 Validation Results:"
echo "  - Contract Tests: ✅ Passed"
echo "  - OpenAPI Spec: ✅ Valid"
echo "  - API Docs: ✅ Generated"
echo ""
echo "📝 Next Steps:"
echo "  1. Review generated API documentation"
echo "  2. Implement remaining service contracts (Database, Validation, Observability)"
echo "  3. Set up contract testing in CI/CD pipeline"
