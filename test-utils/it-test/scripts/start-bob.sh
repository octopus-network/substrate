# rm -rf /tmp/bob
#../../../target/debug/node-template purge-chain --base-path /tmp/bob --chain local

RUST_LOG=runtime::octopus-appchain ../../../target/debug/node-template \
--base-path /tmp/bob \
--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp \
--chain=local \
--bob \
--node-key 0000000000000000000000000000000000000000000000000000000000000002 \
--port 30334 \
--rpc-port 9934 \
--ws-port 9945 \
--no-telemetry \
--execution Native \
--rpc-methods=unsafe \
--ws-external
# --validator \
