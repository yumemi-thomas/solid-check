RUST_TOOLCHAIN ?= 1.97
SOLID_CHECK_BUILD_ID ?= dev
COMPILER_MANIFEST := third_party/dom-expressions/packages/jsx-compiler/Cargo.toml
COMPILER_BIN := third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts
RUST_MANIFEST := rust/Cargo.toml

.PHONY: build build-typefacts build-compiler build-rust package test test-go test-rust test-cli test-compiler test-zed verify conformance clean

build: build-rust

build-typefacts:
	mkdir -p bin
	go build -ldflags "-X main.buildID=$(SOLID_CHECK_BUILD_ID)" -o bin/solid-typefacts ./cmd/solid-typefacts

build-compiler:
	cargo +$(RUST_TOOLCHAIN) build --manifest-path $(COMPILER_MANIFEST) --no-default-features --features sidecar --bin solid-compiler-facts

build-rust: build-typefacts
	mkdir -p bin
	SOLID_CHECK_BUILD_ID="$(SOLID_CHECK_BUILD_ID)" cargo +$(RUST_TOOLCHAIN) build --manifest-path $(RUST_MANIFEST) --workspace
	cp rust/target/debug/solid-check-rust bin/solid-check-rust
	cp rust/target/debug/solid-checkd-rust bin/solid-checkd-rust

package: build-typefacts
	SOLID_CHECK_BUILD_ID="$(SOLID_CHECK_BUILD_ID)" cargo +$(RUST_TOOLCHAIN) build --release --manifest-path $(RUST_MANIFEST) --workspace
	SOLID_CHECK_BUILD_ID="$(SOLID_CHECK_BUILD_ID)" node scripts/package-rust.mjs --output dist/solid-check

test: test-go test-rust test-cli test-compiler test-zed

test-go:
	go test ./cmd/solid-typefacts ./internal/typefacts/... ./internal/wirecbor

test-rust: build-typefacts build-compiler
	SOLID_CHECK_BUILD_ID="$(SOLID_CHECK_BUILD_ID)" SOLID_TYPEFACTS_BIN="$(CURDIR)/bin/solid-typefacts" SOLID_COMPILER_FACTS_BIN="$(CURDIR)/$(COMPILER_BIN)" cargo +$(RUST_TOOLCHAIN) test --manifest-path $(RUST_MANIFEST) --workspace

test-cli:
	npm ci --ignore-scripts --prefix packages/cli
	npm test --prefix packages/cli

test-compiler:
	cargo +$(RUST_TOOLCHAIN) test --manifest-path $(COMPILER_MANIFEST) --no-default-features --features sidecar

test-zed:
	cargo +$(RUST_TOOLCHAIN) test --manifest-path packages/zed-solid-check/Cargo.toml

verify:
	scripts/verify.sh

conformance: build-compiler
	pnpm --dir third_party/dom-expressions install --frozen-lockfile --ignore-scripts
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) pnpm --dir third_party/dom-expressions --filter @dom-expressions/jsx-compiler run build:debug
	node scripts/compiler-conformance.mjs third_party/dom-expressions/packages/jsx-compiler $(COMPILER_BIN)

clean:
	rm -rf bin dist rust/target
