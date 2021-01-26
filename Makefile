# Run tests

.PHONY: test test-less test-loop
test:
	clear
	tmux clear-history || true
	cargo --color always test
test-less:
	make test 2>&1|less -R
test-loop:
	find . | entr make test

# Build binaries

.PHONY: compile _compile
.PHONY: compile-optimized _compile-optimized
.PHONY: optimizer
.PHONY: compile-optimized-reproducible
compile: _compile sienna_token.wasm sienna_mgmt.wasm
_compile:
	cargo build --target wasm32-unknown-unknown --locked
	cp ./target/wasm32-unknown-unknown/debug/*.wasm ./mgmt.wasm
	wasm-opt -Os ./target/wasm32-unknown-unknown/release/sienna_mgmt.wasm -o ./sienna_mgmt.wasm
compile-optimized: _compile-optimized sienna_token.wasm sienna_mgmt.wasm
_compile-optimized:
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
	@# The following line is not necessary, may work only on linux (extra size optimization)
	wasm-opt -Os ./target/wasm32-unknown-unknown/release/snip20_reference_impl.wasm -o ./sienna_token.wasm
	wasm-opt -Os ./target/wasm32-unknown-unknown/release/sienna_mgmt.wasm -o ./sienna_mgmt.wasm
_optimizer: optimizer/*
	docker build                               \
		-f optimizer/Dockerfile                   \
		-t hackbg/secret-contract-optimizer:latest \
		optimizer
compile-optimized-reproducible: _optimizer
	for contract in token mgmt; do                                          \
		docker run --rm                                                        \
			-v "$$(pwd)/$$contract":/contract                                     \
			-v "$$(pwd)/fadroma":/fadroma                                          \
			-v "$$(pwd)/kukumba":/kukumba                                           \
			-e CARGO_NET_GIT_FETCH_WITH_CLI=true                                     \
			-e CARGO_TERM_VERBOSE=true                                                \
			-e CARGO_HTTP_TIMEOUT=120                                                  \
			-e USER=$$(id -u)                                                           \
			-e GROUP=$$(id -g)                                                           \
			--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/code/target \
			--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry     \
			hackbg/secret-contract-optimizer:latest;                                        \
		mv $$contract/contract.wasm.gz dist/$$contract.wasm.gz;                            \
	done
