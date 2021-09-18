#!/bin/bash
sed -i "s/appchain_id: \"\".to_string()/appchain_id: \"oct-it-test\".to_string()/g" ../../bin/node-template/node/src/chain_spec.rs
line=`sed -n "/appchain_id:/p" ../../bin/node-template/node/src/chain_spec.rs`
echo "$line"

sed -i "s/https:\/\/near-rpc.testnet.octopus.network/http:\/\/127.0.0.1:8080\/handler/g" ../../bin/node-template/octopus-pallets/appchain/src/mainchain.rs
line=`sed -n "/url/p" ../../bin/node-template/octopus-pallets/appchain/src/mainchain.rs`
echo "$line"

sed -i "s/pub const EPOCH_DURATION_IN_BLOCKS\: BlockNumber = 10 \* MINUTES/pub const EPOCH_DURATION_IN_BLOCKS\: BlockNumber = 1 \* MINUTES/g" ../../bin/node-template/runtime/src/lib.rs
line=`sed -n "/pub const EPOCH_DURATION_IN_BLOCKS\: BlockNumber =/p" ../../bin/node-template/runtime/src/lib.rs `
echo "$line"


