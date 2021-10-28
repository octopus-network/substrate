const { ApiPromise, WsProvider } = require('@polkadot/api');
const getMockDataFromServer = require('./mockdata.js');
const assert = require("assert");

const customTypes = {
    "IdentificationTuple": "(ValidatorId, FullIdentification)",
    "FullIdentification": "u128",
    "Message": {
        "nonce": "u64",
        "payload_type": "PayloadType",
        "payload": "Vec<u8>"
    },
    "BeefyKey": "[u8; 33]",
    "SessionKeys5B": "(AccountId, AccountId, AccountId, AccountId, BeefyKey)",
    "Validator": {
        "id": "AccountId",
        "weight": "u128"
    },
    "ValidatorSet": {
        "sequence_number": "u32",
        "set_id": "u32",
        "validators": "Vec<Validator>"
    },
    "BurnEvent": {
        "sequence_number": "u32",
        "sender_id": "Vec<u8>",
        "receiver": "AccountId",
        "amount": "u128"
    },
    "LockEvent": {
        "sequence_number": "u32",
        "token_id": "Vec<u8>",
        "sender_id": "Vec<u8>",
        "receiver": "AccountId",
        "amount": "u128"
    },
    "AssetIdOf": "u32",
    "AssetBalanceOf": "u128",
    "TAssetBalance": "u128",
    "Observation": {
        "_enum": {
            "UpdateValidatorSet": "(ValidatorSet<AccountId>)",
            "Burn": "(BurnEvent<AccountId>)",
            "LockAsset": "(LockEvent<AccountId>)"
        }
    },
    "ObservationsPayload": {
        "public": "[u8; 33]",
        "block_number": "BlockNumber",
        "next_fact_sequence": "u32",
        "observations": "Vec<Observation<AccountId>>"
    },
    "ObservationType": {
        "_enum": {
            "UpdateValidatorSet": "UpdateValidatorSet",
            "Burn": "Burn",
            "LockAsset": "LockAsset"
        }
    }
};

function compare(v1, v2) {
    v1.sort();
    v2.sort();
    
    if (v1.length != v2.length) {
        return false;
    }

    for (i = 0; i < v1.length; i++) {
        if (v1[i].toString() != v2[i].toString()) {
            return false;
        }
    }

    return true;
}

async function monitAppChain(testDataPath) {
    const provider = new WsProvider('ws://127.0.0.1:9944', );
    const api = await ApiPromise.create({ provider: provider, types: customTypes});
   
    cnt = 0;
    await api.query.system.events(events => {
        events.forEach(async (record) => {
            const { event, phase } = record;

            if (event.section == "grandpa" && event.method == "NewAuthorities") 
            {
                cnt ++;
                console.log(`cnt =============== ${cnt}`);
                v1 = await api.query.session.validators();
                v2 = await getMockDataFromServer(testDataPath, cnt);
               
                v1.sort();
                v2.sort();

                console.log(`vs1.length: ${v1.length}`);
                console.log(`vs1: ${v1[0]}`);
                console.log(`vs1: ${v1[1]}`);
                console.log(`vs2 length: ${v2.length}`);
                console.log(`vs2: ${v2[0]}`);
                console.log(`vs2: ${v2[1]}`);

                compare_flag = compare(v1, v2);
                if (!compare_flag) {
                    if (cnt == 1) {
                        assert(false, "Validators switch failed")
                    }

                    cnt --;
                    v3 = await getMockDataFromServer(testDataPath, cnt);
                    v3.sort();
                    console.log(`vs3 length: ${v3.length}`);
                    console.log(`vs3: ${v3[0]}`);
                    console.log(`vs3: ${v3[1]}`);

                    compare_flag = compare(v1, v3);
                    if (!compare_flag) {
                        assert(false, "Validators switch failed")
                    }
                }

                console.log(`cnt %%%%%%%%%%%%%%%%% ${cnt}`);
                if (cnt == 6) {
                    console.log(`use case passed!`);
                    //just compare one time
                    process.exit();
                }
                
            } 
        });
    });	

}

(async () => { 
    await monitAppChain("../mock_server/test1.data");
    process.on('unhandledRejection', (reason, promise) => {
        console.log('Unhandled Rejection at:', promise, 'reason:', reason);
        process.exit();
    });
})();
module.exports = monitAppChain;

