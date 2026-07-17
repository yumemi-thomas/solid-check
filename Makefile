RUST_TOOLCHAIN ?= 1.93
COMPILER_MANIFEST := third_party/dom-expressions/packages/jsx-compiler/Cargo.toml
COMPILER_BIN := third_party/dom-expressions/packages/jsx-compiler/target/debug/solid-compiler-facts

.PHONY: build build-go build-compiler build-zed zed-setup test test-go test-cli test-eslint test-oxlint test-compiler test-zed benchmark profile verify conformance corpus clean

build: build-go build-compiler

build-go:
	mkdir -p bin
	go build -o bin/solid-check ./cmd/solid-check
	go build -o bin/solid-checkd ./cmd/solid-checkd

build-compiler:
	cargo +$(RUST_TOOLCHAIN) build --manifest-path $(COMPILER_MANIFEST) --no-default-features --features sidecar --bin solid-compiler-facts

build-zed:
	cargo check --manifest-path packages/zed-solid-check/Cargo.toml

zed-setup: build build-zed

test: test-go test-cli test-eslint test-compiler test-zed

test-go:
	go test ./...

test-cli:
	npm ci --ignore-scripts --prefix packages/cli
	npm test --prefix packages/cli

test-eslint:
	node --test packages/eslint-plugin-solid-check/test/*.test.cjs

test-oxlint: build-compiler
	SOLID_COMPILER_FACTS_BIN="$(CURDIR)/$(COMPILER_BIN)" node scripts/oxlint-conformance.mjs

test-compiler:
	cargo +$(RUST_TOOLCHAIN) test --manifest-path $(COMPILER_MANIFEST) --no-default-features --features sidecar

test-zed:
	cargo test --manifest-path packages/zed-solid-check/Cargo.toml

benchmark:
	go test ./internal/typefacts -run '^$$' -bench '^BenchmarkProjectReferenceLookups$$' -benchmem -count=5
	go test ./internal/engine -run '^$$' -bench '^BenchmarkNativeEngine' -benchmem -count=5

profile:
	mkdir -p .profiles
	go test ./internal/engine -run '^$$' -bench '^BenchmarkNativeEngineIncrementalSnapshot$$' -benchmem -benchtime=10s \
		-cpuprofile .profiles/incremental.cpu.pprof \
		-memprofile .profiles/incremental.mem.pprof
	go tool pprof -top .profiles/incremental.cpu.pprof
	go tool pprof -top -alloc_space .profiles/incremental.mem.pprof

verify:
	scripts/verify.sh

conformance: build-compiler
	SKIP_INSTALL_SIMPLE_GIT_HOOKS=1 pnpm --dir third_party/dom-expressions install --frozen-lockfile
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) pnpm --dir third_party/dom-expressions --filter @dom-expressions/jsx-compiler test
	node scripts/compiler-conformance.mjs third_party/dom-expressions/packages/jsx-compiler $(COMPILER_BIN)

corpus: build
	scripts/run-solid-primitives-corpus.sh

clean:
	rm -rf bin .solid-check
	cargo +$(RUST_TOOLCHAIN) clean --manifest-path $(COMPILER_MANIFEST)
	cargo clean --manifest-path packages/zed-solid-check/Cargo.toml
