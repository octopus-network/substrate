# rm -rf /tmp/alice
#../../target/debug/node-template purge-chain --base-path /tmp/alice --chain local 
RUST_LOG=runtime::octopus-appchain ../../target/debug/node-template \
--base-path /tmp/alice \
--chain=local \
--alice \
--node-key 0000000000000000000000000000000000000000000000000000000000000001 \
--no-telemetry \
--execution Native \
--rpc-methods=unsafe \
--ws-external
# -ltxpool
# --validator \
