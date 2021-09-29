rm -rf /tmp/dave

RUST_LOG="info,runtime::octopus-appchain=debug,runtime::octopus-lpos=debug,runtime::octopus-upward-messages=debug,runtime::octopus-support=debug" \
../../target/debug/node-template \
--base-path /tmp/dave \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD \
--bootnodes /ip4/127.0.0.1/tcp/30335/p2p/12D3KooWSCufgHzV4fCwRijfH2k3abrpAJxTKxEvN1FDuRXA2U9x \
--chain=local \
--dave \
--node-key 0000000000000000000000000000000000000000000000000000000000000004 \
--port 30336 \
--rpc-port 9936 \
--ws-port 9947 \
--no-telemetry \
--execution=Native \
--enable-offchain-indexing=true
