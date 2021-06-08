.DEFAULT_GOAL := prod

# Validation
.PHONY: docs coverage expand
docs:
	cargo doc --document-private-items
coverage:
	cargo tarpaulin --avoid-cfg-tarpaulin --workspace --no-fail-fast --verbose \
		-e snip20-reference-impl --exclude-files=token/* \
		-o Html --output-dir=./coverage
expand:
	cargo expand --manifest-path=mgmt/Cargo.toml --color=always 2>&1 | less -R
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
# Integration testing
# You need to get one of the 4 mnemonics that are initially created by your
# test `secretd` node (can be seen in `docker-compose logs`), and populate
# your `.env` file accordingly (see `README.md` for info about `.env`)
.PHONY: test-localnet
test-localnet:
	docker-compose up -d
	docker-compose exec localnet /sienna/scripts/test.js

# Compilation
.PHONY: prod
_optimizer: optimizer/*
	docker build                                 \
		-f optimizer/Dockerfile                    \
		-t hackbg/secret-contract-optimizer:latest \
		optimizer
prod: _optimizer
	time build/working-tree
# TODO: see if there's any value in keeping these around:
#compile: _compile sienna_token.wasm sienna_mgmt.wasm
#_compile:
	#cargo build --target wasm32-unknown-unknown --locked
	#cp ./target/wasm32-unknown-unknown/debug/*.wasm ./mgmt.wasm
	#wasm-opt -Os ./target/wasm32-unknown-unknown/release/sienna_mgmt.wasm -o ./sienna_mgmt.wasm
#compile-optimized: _compile-optimized sienna_token.wasm sienna_mgmt.wasm
#_compile-optimized:
	#RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
	#@# The following line is not necessary, may work only on linux (extra size optimization)
	#wasm-opt -Os ./target/wasm32-unknown-unknown/release/snip20_reference_impl.wasm -o ./sienna_token.wasm
	#wasm-opt -Os ./target/wasm32-unknown-unknown/release/sienna_mgmt.wasm -o ./sienna_mgmt.wasm

# Configuration
.PHONY: schema schedule
schema:
	cargo run --manifest-path=mgmt/Cargo.toml --example mgmt_schema
schedule:
	./schedule/tsv2json.js

# TODO: update
## Local deployment
#.PHONY: localnet-deploy localnet-configure localnet-launch localnet-claim
#localnet-deploy:
	#docker-compose up -d
	#docker-compose exec localnet /sienna/scripts/deploy.js
#localnet-configure:
	#docker-compose up -d
	#docker-compose exec localnet /sienna/scripts/configure.js
#localnet-status:
	#docker-compose up -d
	#docker-compose exec localnet /sienna/scripts/status.js
#localnet-launch:
	#docker-compose up -d
	#docker-compose exec localnet /sienna/scripts/launch.js
#localnet-claim:
	#docker-compose up -d
	#docker-compose exec localnet /sienna/scripts/claim.js

## Real deployment
#.PHONY: deploy configure launch claim
#deploy:
	#scripts/deploy.js
#configure:
	#scripts/configure.js
#status:
	#scripts/status.js
#launch:
	#scripts/launch.js
#claim:
	#scripts/claim.js

# Debugging
.PHONY: expand
