.DEFAULT_GOAL := compile-optimized-reproducible
.PHONY: test test-less test-loop
.PHONY: compile _compile
.PHONY: compile-optimized _compile-optimized
.PHONY: optimizer
.PHONY: compile-optimized-reproducible

# Build binaries
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
	docker build                                 \
		-f optimizer/Dockerfile                    \
		-t hackbg/secret-contract-optimizer:latest \
		optimizer
compile-optimized-reproducible: _optimizer
	for contract in sienna-mgmt snip20-reference-impl; do                             \
		echo "Now building $$contract:";                                                \
		docker run -it --rm                                                             \
			-v "$$(pwd)":/contract                                                        \
			-e CARGO_NET_GIT_FETCH_WITH_CLI=true                                          \
			-e CARGO_TERM_VERBOSE=true                                                    \
			-e CARGO_HTTP_TIMEOUT=240                                                     \
			-e USER=$$(id -u)                                                             \
			-e GROUP=$$(id -g)                                                            \
			--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/code/target \
			--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry    \
			hackbg/secret-contract-optimizer:latest $$contract &&                         \
		mv "$$contract.wasm.gz" "dist/$$contract.wasm.gz"; done
	gzip -df dist/*.wasm.gz
	sha256sum -b dist/*.wasm > dist/checksums.sha256.txt

# Integration testing
deploy-localnet:
	docker-compose up -d
	docker-compose exec localnet /sienna/deployer/deploy.js
test-localnet:
	docker-compose up -d
	docker-compose exec localnet /sienna/deployer/test.js

# Unit testing
test:
	clear
	tmux clear-history || true
	cargo --color always test
test-docker:
	docker run -it \
		-v "$$(pwd)":/contract \
		-w /contract \
		--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry    \
		rustlang/rust:nightly-slim \
		cargo --color always test
test-less:
	make test 2>&1|less -R
test-loop:
	find . | entr make test
