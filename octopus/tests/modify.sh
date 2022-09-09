#!/bin/bash
# sed -i "s/anchor_contract: \"\".to_string()/anchor_contract: \"oct-it-test\".to_string()/g" ../bin/node-template/node/src/chain_spec.rs
sed -i "" `s/anchor_contract: "octopus-anchor.testnet".to_string()/anchor_contract: "oct-it-test".to_string()/g` ../barnacle/node/src/chain_spec.rs
line=`sed -n "/appchain_id:/p" ../barnacle/node/src/chain_spec.rs`
echo "$line"

#sed -i 's/vec!\[authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")\],/vec!\[authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"), authority_keys_from_seed("Charlie"), authority_keys_from_seed("Dave"), authority_keys_from_seed("Eve")\],/g' ../bin/node-template/node/src/chain_spec.rs
sed -i 's/authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")/\
					authority_keys_from_seed("Alice"),\
					authority_keys_from_seed("Bob"),\
					authority_keys_from_seed("Charlie"),\
					authority_keys_from_seed("Dave"),\
					authority_keys_from_seed("Eve"),\
				/g' ../barnacle/node/src/chain_spec.rs

sed -i "s/https:\/\/rpc.testnet.near.org/http:\/\/127.0.0.1:8080\/handler/g" ../octopus-pallets/appchain/src/lib.rs
sed -i "s/https:\/\/rpc.mainnet.near.org/http:\/\/127.0.0.1:8080\/handler/g" ../octopus-pallets/appchain/src/lib.rs
#line=`sed -n "/url/p" ../bin/node-template/octopus-pallets/appchain/src/lib.rs`
#echo "$line"

sed -i "s/OctopusPalletId::<T>::put(Some(account));/OctopusPalletId::<T>::put(Some(account));\
	<IsActivated<T>>::put(true);/g" ../octopus-pallets/appchain/src/lib.rs

sed -i "s/pub const EPOCH_DURATION_IN_BLOCKS\: BlockNumber = 4 \* HOURS/pub const EPOCH_DURATION_IN_BLOCKS\: BlockNumber = 1 \* MINUTES/g" ../barnacle/runtime/src/lib.rs
line=`sed -n "/pub const EPOCH_DURATION_IN_BLOCKS\: BlockNumber =/p" ../barnacle/runtime/src/lib.rs `
echo "$line"
