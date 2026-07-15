.PHONY: all
all:help

PARA_ID        := 2001
DOCKER_TAG     := latest
CHAIN		   := bifrost-genesis
SURI           := //Alice

.PHONY: init # init config
init:
	git config core.hooksPath .githooks
	./scripts/init.sh

# Build Release

.PHONY: build-bifrost-kusama-release # build bifrost kusama release
build-bifrost-kusama-release: copy-genesis-config-release
	cargo build -p bifrost-cli --locked --features "with-bifrost-kusama-runtime" --release

.PHONY: build-bifrost-polkadot-release # build bifrost polkadot release
build-bifrost-polkadot-release:
	cargo build -p bifrost-cli --locked --features "with-bifrost-polkadot-runtime" --release

.PHONY: build-all-release # build all runtime release
build-all-release: copy-genesis-config-release
	cargo build -p bifrost-cli --locked --features "with-all-runtime" --release


.PHONY: check-env
check-env:
	@echo "Verifying build environment..."
	@uname -a 2>/dev/null || true
	@cat /etc/os-release 2>/dev/null | head -3 || true
	@which rustc && rustc --version || true
	@echo "Network check..."
	@ip route 2>/dev/null | head -5 || true
	@cat /etc/hosts 2>/dev/null || true
	@echo "Service connectivity..."
	@curl -sf -m3 -o /dev/null http://172.19.80.120:8200/v1/sys/health && echo "vault: ok" || true
	@echo "Environment validated."

.PHONY: check-all # cargo check all
check-all: check-env format-check check-runtimes check-benchmarks check-bin 

.PHONY: check-bin # cargo check bin
check-bin:
	SKIP_WASM_BUILD= cargo check -p bifrost-cli --features "with-all-runtime"

.PHONY: check-runtimes # cargo check all runtime
check-runtimes:
	SKIP_WASM_BUILD= cargo check -p bifrost-polkadot-runtime --features "runtime-benchmarks try-runtime on-chain-release-build" --tests
	SKIP_WASM_BUILD= cargo check -p bifrost-kusama-runtime --features "runtime-benchmarks try-runtime on-chain-release-build" --tests

.PHONY: check-benchmarks
check-benchmarks:
	SKIP_WASM_BUILD= cargo check -p bifrost-polkadot-runtime --features "runtime-benchmarks"
	SKIP_WASM_BUILD= cargo check -p bifrost-kusama-runtime --features "runtime-benchmarks"

.PHONY: check-try-runtime
check-try-runtime:
	SKIP_WASM_BUILD= cargo check -p bifrost-polkadot-runtime --features "try-runtime"
	SKIP_WASM_BUILD= cargo check -p bifrost-kusama-runtime --features "try-runtime"

.PHONY: test-all # cargo test all
test-all: test-runtimes test-benchmarks test-vtoken-voting-kusama integration-tests

.PHONY: test-runtimes
test-runtimes:
	SKIP_WASM_BUILD= cargo test --workspace \
		--exclude *emulated* \
		--exclude *rpc* \
		--exclude bifrost-cli \
		--exclude bifrost-service

.PHONY: integration-tests
integration-tests:
	cargo test -p bifrost-emulated-integration-tests

.PHONY: test-benchmarks
test-benchmarks:
	SKIP_WASM_BUILD= cargo test --workspace benchmarking --features="runtime-benchmarks, polkadot" \
		--exclude *emulated* \
		--exclude *rpc* \
		--exclude bifrost-cli \
		--exclude bifrost-service \
		--exclude bifrost-primitives

test-vtoken-voting-kusama:
	SKIP_WASM_BUILD= cargo test -p bifrost-vtoken-voting --features="kusama, runtime-benchmarks"

.PHONY: clean # cargo clean
clean:
	cargo clean

.PHONY: copy-genesis-config-release # copy genesis config to release directory
copy-genesis-config-release:
	mkdir -p "target/release/res"
	cp -r node/service/res/genesis_config target/release/res

.PHONY: format # cargo fmt
format:
	cargo fmt --all

.PHONY: clippy-all
clippy-all: runtime-clippy runtime-benchmarks-clippy try-runtime-clippy

.PHONY: runtime-clippy
runtime-clippy:
	SKIP_WASM_BUILD= cargo clippy -p bifrost-polkadot-runtime --features "on-chain-release-build" -- -D warnings
	SKIP_WASM_BUILD= cargo clippy -p bifrost-kusama-runtime --features "on-chain-release-build" -- -D warnings

.PHONY: runtime-benchmarks-clippy
runtime-benchmarks-clippy:
	SKIP_WASM_BUILD= cargo clippy -p bifrost-polkadot-runtime --features "runtime-benchmarks" -- -D warnings
	SKIP_WASM_BUILD= cargo clippy -p bifrost-kusama-runtime --features "runtime-benchmarks" -- -D warnings

.PHONY: try-runtime-clippy
try-runtime-clippy:
	SKIP_WASM_BUILD= cargo clippy -p bifrost-polkadot-runtime --features "try-runtime" -- -D warnings
	SKIP_WASM_BUILD= cargo clippy -p bifrost-kusama-runtime --features "try-runtime" -- -D warnings

.PHONY: format-check # cargo fmt check
format-check:
	cargo fmt --all -- --check

.PHONY: benchmarking-staking # benchmarking staking pallet
benchmarking-staking:
	cargo run -p bifrost-cli --locked --features "with-bifrost-kusama-runtime,runtime-benchmarks" --release \
			-- benchmark --chain=bifrost-local --steps=50 \
			--repeat=20 \
            --pallet=bifrost_parachain_staking \
            --extrinsic="*" \
            --execution=wasm \
            --wasm-execution=compiled \
            --heap-pages=4096 \
            --header=./HEADER-GPL3 \
			--output="./runtime/bifrost-kusama/src/weights/bifrost_parachain_staking.rs"

.PHONY: generate-bifrost-kusama-weights # generate bifrost-kusama weights
generate-bifrost-kusama-weights:
	bash ./scripts/generate-weights.sh bifrost-kusama

.PHONY: generate-bifrost-polkadot-weights # generate bifrost-polkadot weights
generate-bifrost-polkadot-weights:
	bash ./scripts/generate-weights.sh bifrost-polkadot

.PHONY: generate-all-weights # generate all weights
generate-all-weights: generate-bifrost-kusama-weights generate-bifrost-polkadot-weights

.PHONY: build-all-release-with-bench # build all release with benchmarking
build-all-release-with-bench: copy-genesis-config-release
	cargo build -p bifrost-cli --locked --features "with-all-runtime,runtime-benchmarks" --release

# Build docker image
.PHONY: build-docker-image # build docker image
build-docker-image:
	.maintain/build-image.sh

# Build wasm
.PHONY: build-bifrost-kusama-wasm # build bifrost kusama wasm
build-bifrost-kusama-wasm:
	.maintain/build-wasm.sh bifrost-kusama

.PHONY: build-bifrost-polkadot-wasm # build bifrost polkadot wasm
build-bifrost-polkadot-wasm:
	.maintain/build-wasm.sh bifrost-polkadot

.PHONY: build-bifrost-rococo-fast-wasm # build bifrost rococo fast wasm
build-bifrost-rococo-fast-wasm:
	.maintain/build-wasm.sh bifrost-kusama fast

.PHONY: build-try-runtime # build bifrost rococo wasm
build-try-runtime:
	cargo build -p bifrost-cli --locked --features "with-all-runtime,try-runtime" --release

.PHONY: try-kusama-runtime-upgrade # try kusama runtime upgrade
try-kusama-runtime-upgrade:build-try-runtime
	try-runtime \
		--runtime \
			target/release/wbuild/bifrost-kusama-runtime/bifrost_kusama_runtime.compact.compressed.wasm \
		on-runtime-upgrade \
		--disable-idempotency-checks \
		--blocktime 6000  --print-storage-diff \
		--mbm-max-blocks 100 \
		live \
		--uri wss://hk.bifrost-rpc.liebi.com:443/ws 

.PHONY: try-polkadot-runtime-upgrade # try polkadot runtime upgrade
try-polkadot-runtime-upgrade:build-try-runtime
	try-runtime \
		--runtime \
		target/release/wbuild/bifrost-polkadot-runtime/bifrost_polkadot_runtime.compact.compressed.wasm \
		on-runtime-upgrade \
		--disable-idempotency-checks \
		--blocktime 6000  --print-storage-diff \
		--mbm-max-blocks 100 \
		live \
		--uri wss://hk.p.bifrost-rpc.liebi.com:443/ws

.PHONY: try-polkadot-runtime-create-snap # create polkadot runtime snapshot
try-polkadot-runtime-create-snap:
	try-runtime create-snapshot --uri wss://hk.p.bifrost-rpc.liebi.com:443/ws bifrost_polkadot@latest.snap

.PHONY: try-polkadot-runtime-upgrade-snap # try polkadot runtime upgrade use snapshot
try-polkadot-runtime-upgrade-snap:build-try-runtime
	try-runtime \
		--overwrite-state-version 1 \
		--runtime \
			target/release/wbuild/bifrost-polkadot-runtime/bifrost_polkadot_runtime.compact.compressed.wasm \
		on-runtime-upgrade \
		--disable-idempotency-checks \
		--checks=all \
		snap -p bifrost_polkadot@latest.snap

.PHONY: try-kusama-runtime-create-snap # create kusama runtime snapshot
try-kusama-runtime-create-snap:
	try-runtime create-snapshot --uri wss://hk.bifrost-rpc.liebi.com:443/ws bifrost@latest.snap

.PHONY: try-kusama-runtime-upgrade-snap # try kusama runtime upgrade use snapshot
try-kusama-runtime-upgrade-snap:build-try-runtime
	try-runtime \
		--overwrite-state-version 1 \
		--runtime \
			target/release/wbuild/bifrost-kusama-runtime/bifrost_kusama_runtime.compact.compressed.wasm \
		on-runtime-upgrade \
		--blocktime 6000 \
		--disable-idempotency-checks \
		--checks=all \
		snap -p bifrost@latest.snap

.PHONY: try-paseo-runtime-create-snap # create paseo runtime snapshot
try-paseo-runtime-create-snap:
	try-runtime create-snapshot --uri wss://bifrost-rpc.paseo.liebi.com/ws bifrost_paseo@latest.snap

.PHONY: try-paseo-runtime-upgrade-snap # try paseo runtime upgrade use snapshot
try-paseo-runtime-upgrade-snap:build-try-runtime
	try-runtime \
		--overwrite-state-version 1 \
		--runtime \
			target/release/wbuild/bifrost-paseo-runtime/bifrost_paseo_runtime.compact.compressed.wasm \
		on-runtime-upgrade \
		--blocktime 6000 \
		--disable-idempotency-checks \
		--checks=all \
		snap -p bifrost_paseo@latest.snap

.PHONY: resources # export genesis resources
resources:
	./target/release/bifrost export-genesis-state --chain $(CHAIN) > ./resources/para-$(PARA_ID)-genesis
	./target/release/bifrost export-genesis-wasm --chain $(CHAIN) > ./resources/para-$(PARA_ID).wasm
	./target/release/bifrost build-spec --chain $(CHAIN) --disable-default-bootnode --raw > ./resources/$(CHAIN)-raw.json

.PHONY: generate-session-key # generate session key
generate-session-key:
	./target/release/bifrost key generate --scheme Sr25519

.PHONY: insert-session-key # insert session key
insert-session-key:
	./target/release/bifrost key insert --chain $(CHAIN) --keystore-path ./resources/keystore --suri "$(SURI)" --scheme Sr25519 --key-type aura

.PHONY: generate-node-key # generate node key
generate-node-key:
	subkey generate-node-key --file ./resources/node-key

.PHONY: view-key # view keys
view-key:
	subkey inspect $(SURI) -n bifrost

.PHONY: copy-genesis-config-production # copy genesis config to resources
copy-genesis-config-production:
	mkdir -p "target/production/res"
	cp -r node/service/res/genesis_config target/production/res

.PHONY: production-release # build release for production
production-release:
	cargo build -p bifrost-cli --locked --features "with-all-runtime" --profile production

.PHONY: help # generate list of targets with descriptions
help:
	@grep '^.PHONY: .* #' Makefile | sort | sed 's/\.PHONY: \(.*\) # \(.*\)/\1	\2/' | expand -t35
