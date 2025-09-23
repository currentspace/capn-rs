# Cap'n Web Rust Implementation - Development Makefile

.PHONY: help test test-unit test-integration test-interop test-all coverage coverage-html coverage-xml build clean fmt lint check bench docs

# Default target
help:
	@echo "🛠️  Cap'n Web Rust Development Commands"
	@echo "========================================"
	@echo ""
	@echo "🧪 Testing:"
	@echo "  make test          - Run all Rust tests"
	@echo "  make test-unit     - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make test-interop  - Run TypeScript interoperability tests"
	@echo "  make test-all      - Run all tests including interop"
	@echo ""
	@echo "📊 Coverage:"
	@echo "  make coverage      - Generate comprehensive coverage report"
	@echo "  make coverage-html - Generate HTML coverage report"
	@echo "  make coverage-xml  - Generate XML coverage report"
	@echo ""
	@echo "🏗️  Building:"
	@echo "  make build         - Build all workspace crates"
	@echo "  make build-release - Build optimized release version"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "🔧 Development:"
	@echo "  make fmt           - Format code with rustfmt"
	@echo "  make lint          - Run clippy linter"
	@echo "  make check         - Type check without building"
	@echo ""
	@echo "📚 Documentation:"
	@echo "  make docs          - Generate documentation"
	@echo "  make docs-open     - Generate and open documentation"
	@echo ""
	@echo "⚡ Performance:"
	@echo "  make bench         - Run benchmarks"
	@echo ""
	@echo "🔄 CI/CD:"
	@echo "  make ci            - Run full CI pipeline locally"

# Testing targets
test:
	@echo "🧪 Running all Rust tests..."
	cargo test --workspace

test-unit:
	@echo "🧪 Running unit tests..."
	cargo test --workspace --lib

test-integration:
	@echo "🔧 Running integration tests..."
	cargo test --workspace --test '*'

test-interop:
	@echo "🌐 Running TypeScript interoperability tests..."
	cd typescript-interop && ./setup.sh && npm test

test-all: test test-interop
	@echo "✅ All tests completed!"

# Coverage targets
coverage:
	@echo "📊 Generating comprehensive coverage report..."
	./scripts/coverage.sh

coverage-html:
	@echo "📊 Generating HTML coverage report..."
	cargo tarpaulin --out Html --output-dir target/coverage --timeout 300 --exclude-files "examples/*" "benches/*" "typescript-interop/*"
	@echo "📈 Coverage report: file://$(PWD)/target/coverage/tarpaulin-report.html"

coverage-xml:
	@echo "📊 Generating XML coverage report..."
	cargo tarpaulin --out Xml --output-dir target/coverage --timeout 300 --exclude-files "examples/*" "benches/*" "typescript-interop/*"
	@echo "📈 Coverage report: $(PWD)/target/coverage/cobertura.xml"

# Build targets
build:
	@echo "🏗️  Building all workspace crates..."
	cargo build --workspace

build-release:
	@echo "🏗️  Building optimized release version..."
	cargo build --workspace --release

build-examples:
	@echo "🏗️  Building examples..."
	cargo build --examples

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf target/coverage
	rm -rf typescript-interop/dist
	rm -rf typescript-interop/node_modules

# Development targets
fmt:
	@echo "🎨 Formatting code with rustfmt..."
	cargo fmt --all

lint:
	@echo "🔍 Running clippy linter..."
	cargo clippy --workspace --all-targets --all-features -- -D warnings

check:
	@echo "🔍 Type checking..."
	cargo check --workspace --all-targets --all-features

# Documentation targets
docs:
	@echo "📚 Generating documentation..."
	cargo doc --workspace --no-deps

docs-open:
	@echo "📚 Generating and opening documentation..."
	cargo doc --workspace --no-deps --open

# Performance targets
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench --workspace

# CI/CD pipeline
ci: clean fmt lint check test coverage
	@echo "🎯 CI pipeline completed successfully!"

# Development setup
setup:
	@echo "⚙️  Setting up development environment..."
	@echo "📦 Installing Rust components..."
	rustup component add rustfmt clippy
	@echo "🔧 Installing cargo tools..."
	cargo install cargo-tarpaulin || echo "tarpaulin already installed"
	@echo "🌐 Setting up TypeScript environment..."
	cd typescript-interop && ./setup.sh
	@echo "✅ Development environment ready!"

# Quick development commands
dev-test: fmt check test
	@echo "🚀 Quick development test cycle completed!"

dev-server:
	@echo "🚀 Starting development server..."
	cargo run --example basic_server

# Protocol validation
validate-protocol: test-interop coverage
	@echo "✅ Protocol validation completed!"

# Release preparation
prepare-release: clean fmt lint check test-all coverage docs
	@echo "🎊 Release preparation completed!"
	@echo "📋 Checklist:"
	@echo "  ✅ Code formatted"
	@echo "  ✅ Linting passed"
	@echo "  ✅ All tests passed"
	@echo "  ✅ Interop tests passed"
	@echo "  ✅ Coverage generated"
	@echo "  ✅ Documentation updated"