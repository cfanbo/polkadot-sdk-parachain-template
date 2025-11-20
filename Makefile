# Makefile for parachain-template-node

# 配置变量
BINARY_NAME := parachain-template-node
CARGO_FLAGS :=
RELEASE_FLAGS := --release
FEATURES :=

# 默认目标
.DEFAULT_GOAL := help

# 构建 release 版本
build:
	@echo "Building $(BINARY_NAME) in release mode..."
	cargo build $(RELEASE_FLAGS) -p $(BINARY_NAME) $(CARGO_FLAGS)

serve:
	zombienet -p native spawn zombienet.toml

# 构建 debug 版本
debug:
	@echo "Building $(BINARY_NAME) in debug mode..."
	cargo build -p $(BINARY_NAME) $(CARGO_FLAGS)

# 安装到 Cargo bin 目录
install: build
	@echo "Installing $(BINARY_NAME)..."
	cargo install --path node --locked --force

# 清理构建文件
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# 运行测试
test:
	@echo "Running tests..."
	cargo test -p $(BINARY_NAME)

# 格式检查
fmt:
	@echo "Checking formatting..."
	cargo fmt -- --check

# Clippy 检查
clippy:
	@echo "Running clippy..."
	cargo clippy -p $(BINARY_NAME) -- -D warnings

# 构建并显示二进制信息
info: build
	@echo "=== Binary Information ==="
	@ls -lh ./target/release/$(BINARY_NAME)
	@echo ""

# 快速开发构建（增量）
dev:
	@echo "Fast development build..."
	cargo build -p $(BINARY_NAME)

# 构建 WASM runtime
wasm:
	@echo "Building WASM runtime..."
	cargo build -p parachain-template-runtime --target wasm32-unknown-unknown --release

# 显示帮助信息
help:
	@echo "Available targets:"
	@echo "  build     - Build release version (default)"
	@echo "  debug     - Build debug version"
	@echo "  dev       - Fast development build"
	@echo "  install   - Install binary to cargo bin"
	@echo "  clean     - Clean build artifacts"
	@echo "  test      - Run tests"
	@echo "  fmt       - Check code formatting"
	@echo "  clippy    - Run clippy linting"
	@echo "  wasm      - Build WASM runtime only"
	@echo "  info      - Build and show binary info"
	@echo "  help      - Show this help message"

.PHONY: build serve debug install clean test fmt clippy wasm info help dev
