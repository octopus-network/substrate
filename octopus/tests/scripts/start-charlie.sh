rm -rf /tmp/charlie

RUST_LOG="info,runtime::octopus-appchain=debug,runtime::octopus-lpos=debug,runtime::octopus-upward-messages=debug,runtime::octopus-support=debug" \
../../target/debug/node-template \
--base-path /tmp/charlie \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD \
--chain=local \
--charlie \
--node-key 0000000000000000000000000000000000000000000000000000000000000003 \
--port 30335 \
--rpc-port 9935 \
--ws-port 9946 \
--no-telemetry \
--execution=Native \
--enable-offchain-indexing=true
