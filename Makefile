.DEFAULT_GOAL := compile-optimized-reproducible

# Build binaries
.PHONY: compile _compile compile-optimized _compile-optimized compile-optimized-reproducible
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
# You need to get one of the 4 mnemonics that are initially created by your
# test `secretd` node (can be seen in `docker-compose logs`), and populate
# your `.env` file accordingly (see `README.md` for info about `.env`)
.PHONY: test-localnet
test-localnet:
	docker-compose up -d
	docker-compose exec localnet /sienna/scripts/test.js

# Unit testing
.PHONY: test test-docker test-less test-loop coverage
test:
	clear
	tmux clear-history || true
	cargo --color always test --no-fail-fast
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

# Extra artifacts
.PHONY: docs coverage schema schedule
docs:
	cargo doc --document-private-items
coverage:
	cargo tarpaulin --avoid-cfg-tarpaulin --workspace --no-fail-fast --verbose \
		-e snip20-reference-impl --exclude-files=token/* \
		-o Html --output-dir=./coverage
schema:
	cargo run --manifest-path=mgmt/Cargo.toml --example mgmt_schema
schedule:
	./scripts/tsv2json.js

# Local deployment
.PHONY: localnet-deploy localnet-configure localnet-launch localnet-claim
localnet-deploy:
	docker-compose up -d
	docker-compose exec localnet /sienna/scripts/deploy_mgmt_and_token.js
localnet-configure:
	docker-compose up -d
	docker-compose exec localnet /sienna/scripts/configure.js
localnet-launch:
	docker-compose up -d
	docker-compose exec localnet /sienna/scripts/launch.js
localnet-claim:
	docker-compose up -d
	docker-compose exec localnet /sienna/scripts/claim.js

# Local deployment
.PHONY: localnet-deploy localnet-configure localnet-launch localnet-claim
deploy:
	echo "Not implemented"
configure:
	echo "Not implemented"
launch:
	echo "Not implemented"
claim:
	echo "Not implemented"

# Debugging
.PHONY: expand
expand:
	cargo expand --manifest-path=mgmt/Cargo.toml --color=always 2>&1 | less -R
