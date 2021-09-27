# rm -rf /tmp/eve
#../../target/debug/node-template purge-chain --base-path /tmp/eve --chain local

RUST_LOG=runtime::octopus-appchain ../../target/debug/node-template \
--base-path /tmp/eve \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWHdiAxVd8uMQR1hGWXccidmfCwLqcMpGwR6QcTP6QRMuD \
--bootnodes /ip4/127.0.0.1/tcp/30335/p2p/12D3KooWSCufgHzV4fCwRijfH2k3abrpAJxTKxEvN1FDuRXA2U9x \
--chain=local \
--eve \
--node-key 0000000000000000000000000000000000000000000000000000000000000005 \
--port 30337 \
--rpc-port 9937 \
--ws-port 9948 \
--no-telemetry \
--rpc-external \
--rpc-cors all \
--execution Native \
--rpc-methods=unsafe \
--ws-external
# --validator \
