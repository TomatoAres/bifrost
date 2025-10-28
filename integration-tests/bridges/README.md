# Bridges Tests for Local Polkadot <> Kusama Bridge

This folder contains zombienet based integration test for both onchain and offchain bridges code.
The tests are designed to be run manually.

## setup

```
mkdir -p ~/local_bridge_testing/bin
cd ~/local_bridge_testing/bin
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2503/polkadot
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2503/polkadot-execute-worker
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2503/polkadot-prepare-worker
wget https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2503/polkadot-parachain
wget https://github.com/paritytech/zombienet/releases/download/v1.3.133/zombienet-linux-x64 -O zombienet
chmod +x polkadot*
chmod +x zombienet

yarn global add @polkadot/api-cli

# in another folder of your choice, build the relay
cd
git clone https://github.com/paritytech/parity-bridges-common.git
cd parity-bridges-common
cargo +nightly build -p substrate-relay --release
cp ./target/release/substrate-relay ~/local_bridge_testing/bin/

# in runtimes repo, build chainspec generator with sudo:
cd 
git clone https://github.com/polkadot-fellows/runtimes.git
cd runtimes
git checkout v1.7.1
# sudo must be enabled for relaychains
git apply ./integration-tests/bridges/sudo-relay.patch
cargo +nightly build --release -p chain-spec-generator --no-default-features --features fast-runtime,polkadot,kusama,bridge-hub-kusama,bridge-hub-polkadot,asset-hub-kusama,asset-hub-polkadot
cp ./target/release/chain-spec-generator ~/local_bridge_testing/bin/chain-spec-generator-kusama
cp ./target/release/chain-spec-generator ~/local_bridge_testing/bin/chain-spec-generator-polkadot

# in another folder of your choice, build the bifrost with sudo
cd
git clone https://github.com/bifrost-io/bifrost.git
cd bifrost
git apply ./integration-tests/bridges/sudo-bifrost.patch
cargo build -p bifrost-cli --features "with-all-runtime" --release
cp ./target/release/bifrost ~/local_bridge_testing/bin/
```

## manual testing

If you'd like to interact with the local test networks, run this instead

```
./run-test.sh 0000-manual
``` 

then you can point your browser to

* Bifrost (Kusama) https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9144#
* Bifrost (Polkadot) https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9244#
* Asset Hub Kusama https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9010#
* Asset Hub Polkadot https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9910#
* Bridge Hub Kusama https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8945#
* Bridge Hub Polkadot https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A8943#

### interventions

this setup is brittle and it can happen that not all setup calls succeed. To iterate a setup step, do:

```
cd integration-tests/bridges/environments/polkadot-kusama
export ENV_PATH=~/bifrost/integration-tests/bridges/environments/polkadot-kusama
export FRAMEWORK_PATH=~/local_bridge_testing/downloads/polkadot-sdk/bridges/testing/framework/
source "$FRAMEWORK_PATH/utils/bridges.sh"
source "$FRAMEWORK_PATH/utils/zombienet.sh"

# re-run init scripts
./helper.sh init-asset-hub-polkadot-local