# NetworkX-RS Makefile

.PHONY: help install dev test bench clean build release

help:
	@echo "NetworkX-RS Development Commands"
	@echo "================================"
	@echo "make install    - Install dependencies"
	@echo "make dev        - Build for development"
	@echo "make test       - Run all tests"
	@echo "make bench      - Run benchmarks"
	@echo "make clean      - Clean build artifacts"
	@echo "make release    - Build optimized release"
	@echo "make docs       - Generate documentation"

install:
	pip install maturin pytest pytest-benchmark networkx hypothesis
	pip install black ruff mypy
	cargo install flamegraph
	rustup component add rustfmt clippy

dev:
	maturin develop --release

test: test-rust test-python

test-rust:
	cargo test --all-features

test-python: dev
	pytest python/tests/ -v

bench: bench-rust bench-python

bench-rust:
	cargo bench

bench-python: dev
	python benches/compare_networkx.py

format:
	cargo fmt
	black python/
	ruff check --fix python/

lint:
	cargo clippy -- -D warnings
	ruff check python/
	mypy python/

clean:
	cargo clean
	rm -rf target/
	rm -rf python/networkx_rs.egg-info/
	rm -rf python/networkx_rs/*.so
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete

build:
	cargo build --release
	maturin build --release

release: clean build test
	@echo "Ready for release!"

docs:
	cargo doc --no-deps --open
	cd python && sphinx-build -b html docs docs/_build

profile-dijkstra:
	cargo flamegraph --bench dijkstra -- --bench

check: format lint test
	@echo "All checks passed!"