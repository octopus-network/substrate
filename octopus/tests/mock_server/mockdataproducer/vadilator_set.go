package mockdataproducer

import (
	"encoding/json"
	"math/rand"
	"time"
)

type ValidatorInfo struct {
	Id        string
	AccountId string
	Weight    string
}

const PRESET_VALIDATORS_SIZE = 5

//preset validators
var PresetValidators = [...]ValidatorInfo{
	{"0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", "Alice-octopus.testnet", "10000000000"},
	{"0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", "Bob-octopus.testnet", "10000000000"},
	{"0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22", "Charlie-octopus.testnet", "10000000000"},
	{"0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20", "Dave-octopus.testnet", "10000000000"},
	{"0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e", "Eve-octopus.testnet", "10000000000"},
}

type ValidatorData struct {
	ValidatorId string `json:"validator_id_in_appchain"`
	TotalStake  string `json:"total_stake"`
}

type ResultData struct {
	BlockHash   string   `json:"block_hash"`
	BlockHeight uint64   `json:"block_height"`
	Logs        []string `json:"logs"`
	InnerResult []int    `json:"result"` //it's UpdateValidatorSetData and LockTokenData
}

type Ret struct {
	Jsonrpc string     `json:"jsonrpc"`
	Id      string     `json:"id"`
	Result  ResultData `json:"result"`
}

var BlockHeight uint64 = 1

func ProduceValidatorData(curr int) ValidatorData {
	rand.Seed(time.Now().UnixNano())
	num := rand.Intn(5)

	if curr > PRESET_VALIDATORS_SIZE {
		curr %= PRESET_VALIDATORS_SIZE
	}
	validator := PresetValidators[curr]

	if num%2 == 0 {
		BlockHeight++
	}

	return ValidatorData{
		ValidatorId: validator.Id,
		TotalStake:  validator.Weight,
	}
}

func ProduceUpdateValidatorSets(curr []int) []interface{} {
	num := len(curr)

	var validatorSet = []interface{}{}
	for i := 0; i < num; i++ {
		if curr[i] < 0 {
			continue
		}
		data := ProduceValidatorData(curr[i])
		validatorSet = append(validatorSet, data)
	}

	// fmt.Printf("%v\n", validatorSet...)
	return validatorSet
}

var BlockHeight2 uint64 = 1

func StringToInts(s string) []int {
	intSlice := make([]int, len(s))
	for i, _ := range s {
		intSlice[i] = int(s[i])
	}
	return intSlice
}

func ProduceNewResponseForValidatorSets(simulationSequence [][]int) Ret {
	innerResult := []interface{}{}

	if len(simulationSequence) != 1 {
		panic("Should 1")
	}

	for i := 0; i < len(simulationSequence); i++ {
		if simulationSequence[i][0] == 1 { //update validators
			curr := simulationSequence[i][1:]
			innerResult = ProduceUpdateValidatorSets(curr)
		}
	}

	result, _ := json.Marshal(innerResult)

	retData := ResultData{
		BlockHash:   "EczErquQLMpUvTQpKupoQp5yNkgNbniMSHq1gVvhAf84",
		BlockHeight: BlockHeight2,
		Logs:        []string{},
		InnerResult: StringToInts(string(result)),
	}

	return Ret{
		Jsonrpc: "2.0",
		Id:      "dontcare",
		Result:  retData,
	}
}

func ProduceEmptyResponseForValidatorSets() Ret {
	var validatorSet = []interface{}{}
	result, _ := json.Marshal(validatorSet)

	retData := ResultData{
		BlockHash:   "EczErquQLMpUvTQpKupoQp5yNkgNbniMSHq1gVvhAf84",
		BlockHeight: BlockHeight2,
		Logs:        []string{},
		InnerResult: StringToInts(string(result)),
	}

	return Ret{
		Jsonrpc: "2.0",
		Id:      "dontcare",
		Result:  retData,
	}
}
