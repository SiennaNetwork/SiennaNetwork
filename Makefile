.PHONY: compile _compile
compile: _compile contract.wasm.gz
_compile:
	cargo build --target wasm32-unknown-unknown --locked
	cp ./target/wasm32-unknown-unknown/debug/*.wasm ./contract.wasm

.PHONY: compile-optimized _compile-optimized
compile-optimized: _compile-optimized
_compile-optimized:
	RUSTFLAGS='-C link-arg=-s' cargo +nightly build --release --target wasm32-unknown-unknown --locked
	@# The following line is not necessary, may work only on linux (extra size optimization)
	# wasm-opt -Os ./target/wasm32-unknown-unknown/release/*.wasm -o .
	cp ./target/wasm32-unknown-unknown/release/*.wasm ./build/

.PHONY: compile-w-debug-print _compile-w-debug-print
compile-w-debug-print: _compile-w-debug-print
_compile-w-debug-print:
	RUSTFLAGS='-C link-arg=-s' cargo +nightly build --release --target wasm32-unknown-unknown --locked
	cd contracts/lp-staking && RUSTFLAGS='-C link-arg=-s' cargo build --release --features debug-print --target wasm32-unknown-unknown --locked
	#cd contracts/dummy_swap_data_receiver && RUSTFLAGS='-C link-arg=-s' cargo build --release --features debug-print --target wasm32-unknown-unknown --locked
	cp ./target/wasm32-unknown-unknown/release/*.wasm ./build/

.PHONY: compile-optimized-reproducible
compile-optimized-reproducible:
	docker run --rm -v "$$(pwd)":/contract \
		--mount type=volume,source="$$(basename "$$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		enigmampc/secret-contract-optimizer:1.0.4

.PHONY: start-server
start-server: # CTRL+C to stop
	docker run -it --rm \
		-p 26657:26657 -p 26656:26656 -p 1337:1337 \
		-v $$(pwd):/root/code \
		--name secretdev enigmampc/secret-network-sw-dev:latest



clean:
	cargo clean
	rm -f ./build/*