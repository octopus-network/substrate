rm -rf /tmp/alice

RUST_LOG="info,runtime::octopus-appchain=debug,runtime::octopus-lpos=debug,runtime::octopus-upward-messages=debug,runtime::octopus-support=debug" \
../../../target/debug/appchain-barnacle \
--base-path /tmp/alice \
--chain=local \
--alice \
--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
--no-telemetry \
--rpc-external \
--rpc-cors=all \
--rpc-methods=Unsafe \
--ws-external \
--execution=Native \
--enable-offchain-indexing=true \
--pruning=archive
