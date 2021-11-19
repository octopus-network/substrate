#!/bin/bash
sed -i "s/anchor_contract: \"\".to_string()/anchor_contract: \"oct-it-test\".to_string()/g" ../bin/node-template/node/src/chain_spec.rs
line=`sed -n "/appchain_id:/p" ../bin/node-template/node/src/chain_spec.rs`
echo "$line"

#sed -i 's/vec!\[authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")\],/vec!\[authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob"), authority_keys_from_seed("Charlie"), authority_keys_from_seed("Dave"), authority_keys_from_seed("Eve")\],/g' ../bin/node-template/node/src/chain_spec.rs
sed -i 's/authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")/\
					authority_keys_from_seed("Alice"),\
					authority_keys_from_seed("Bob"),\
					authority_keys_from_seed("Charlie"),\
					authority_keys_from_seed("Dave"),\
					authority_keys_from_seed("Eve"),\
				/g' ../bin/node-template/node/src/chain_spec.rs

sed -i "s/https:\/\/rpc.testnet.near.org/http:\/\/127.0.0.1:8080\/handler/g" ../bin/node-template/octopus-pallets/appchain/src/mainchain.rs
line=`sed -n "/url/p" ../bin/node-template/octopus-pallets/appchain/src/mainchain.rs`
echo "$line"
