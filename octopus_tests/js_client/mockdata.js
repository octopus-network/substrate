const readline = require('readline');
const fs = require('fs');

//all validators public key
var presetValidators = new Array(
  '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY', 
  '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', 
  '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y',
  '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
  '5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw',
  '5CiPPseXPECbkjWCa6MnjNokrgYjMqmKndv2rSnekmSK2DjL',
);

async function getMockDataFromServer(filepath, number) {
  const rl = readline.createInterface({
    input: fs.createReadStream(filepath)
  });

  f = 0
  const promise = new Promise(resolve => {
    rl.on('line', (str) => {
      f++
      if (f == number) {
        str = str.slice(0, str.length-1);
        arr = str.split(',');

        console.log(arr.toString()) 

        var validators= new Array();
        for (i = 1; i < arr.length; i++) {
          validators.push(presetValidators[arr[i]]); 
          // console.log(validators[i]);
        }

        // console.log("va: ", validators);
        resolve(validators);
      }
    });
  });

  const validators = await promise;
  // console.log("va: ", validators);
  return validators;
}

// (async () => { 
//   const va = await getMockDataFromServer("../mock_server/test1.data", 1);
//   console.log("va: ", va);
// })();

module.exports = getMockDataFromServer;
