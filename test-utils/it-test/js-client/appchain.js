const { ApiPromise, WsProvider } = require('@polkadot/api');
const getMockDataFromServer = require('./mockdata.js');
const assert = require("assert");

const customTypes = {
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
    }
};


async function monitAppChain(testDataPath) {
    const provider = new WsProvider('ws://127.0.0.1:9945', );
    const api = await ApiPromise.create({ provider: provider, types: customTypes});
    
    api.query.system.events(events => {
        events.forEach((record) => {
            const { event, phase } = record;

            if (event.section == "octopusLpos" && event.method == "StakingElection") {
            //if (event.section == "system" && event.method == "ExtrinsicSuccess") {
                api.query.session.validators(async validators1 => {
                    validators2 = await getMockDataFromServer(testDataPath);
                    validators1.sort();
                    validators2.sort();

                    console.log(`vs1.length: ${validators1.length}`);
                    console.log(`vs1: ${validators1[0]}`);
                    console.log(`vs1: ${validators1[1]}`);
                    console.log(`vs2 length: ${validators2.length}`);
                    console.log(`vs2: ${validators2[0]}`);
                    console.log(`vs2: ${validators2[1]}`);

                    //compare
                    assert((validators1.length == validators2.length), 
                        'validators1.length != validators2.length !');
                    
                    for (i = 0; i < validators1.length; i++) {
                        assert((validators1[i].toString() == validators2[i].toString()), 
                            'validator not match!');
                    }
                    
                    //just compare one time
                    process.exit();
                })
            } 
        });
    });	

}

(async () => { 
    await monitAppChain("../mock_server/test1.data");
})();
module.exports = monitAppChain;

