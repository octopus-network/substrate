package main

import (
	"bufio"
	"fmt"
	"log"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"sync"
	"time"

	"encoding/json"
)

//const (
//	LockToken          = "LockToken"
//	UpdateValidatorSet = "UpdateValidatorSet"
//)

type LockTokenData struct {
	SeqNum   uint64 `json:"seq_num"`
	TokenId  string `json:"token_id"`
	SenderId string `json:"sender_id"`
	Receiver string `json:"receiver"`
	Amount   string `json:"amount"`
}

type ValidatorInfo struct {
	Id        string
	AccountId string
	Weight    string
}

const PRESET_VALIDATORS_SIZE = 5

//preset validators
var presetValidators = [...]ValidatorInfo{
	{"0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", "Alice-octopus.testnet", "10000000000"},
	{"0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", "Bob-octopus.testnet", "10000000000"},
	{"0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22", "Charlie-octopus.testnet", "100000000000"},
	{"0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20", "Dave-octopus.testnet", "10000000000"},
	{"0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e", "Eve-octopus.testnet", "10000000000"},
}

type ValidatorData struct {
	Id          string        `json:"id"`
	AccountId   string        `json:"account_id"`
	Weight      string        `json:"weight"`
	BlockHeight uint64        `json:"block_weight"`
	Delegators  []interface{} `json:"delegators"`
}

type UpdateValidatorSetData struct {
	SeqNum       uint64          `json:"seq_num"`
	SetId        uint64          `json:"set_id"`
	ValidatorSet []ValidatorData `json:"validators"`
}

type InnerResultValidatorSet struct {
	UpdateValidatorSet UpdateValidatorSetData `json:"UpdateValidatorSet"`
}

type InnerResultLockToken struct {
	LockToken LockTokenData `json:"LockAsset"`
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

var blockHeight uint64 = 1

func ProduceValidatorData(curr int) ValidatorData {
	rand.Seed(time.Now().UnixNano())
	num := rand.Intn(5)

	if curr > PRESET_VALIDATORS_SIZE {
		curr %= PRESET_VALIDATORS_SIZE
	}
	validator := presetValidators[curr]

	if num%2 == 0 {
		blockHeight++
	}

	return ValidatorData{
		Id:          validator.Id,
		AccountId:   validator.AccountId,
		Weight:      validator.Weight,
		BlockHeight: blockHeight,
		Delegators:  []interface{}{}, //delegators is null
	}
}

var setId uint64 = 1
var seqNum uint64 = 0

func ProduceLockTokenData() LockTokenData {
	ret := LockTokenData{
		SeqNum:   seqNum,
		TokenId:  "test-stable.testnet",
		SenderId: "yuanchao.testnet",
		Receiver: "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22",
		Amount:   "123000000000000",
	}

	seqNum++

	return ret
}

var blockHeight2 uint64 = 1

func StringToInts(s string) []int {
	intSlice := make([]int, len(s))
	for i, _ := range s {
		intSlice[i] = int(s[i])
	}
	return intSlice
}

func ProduceUpdateValidatorSets(curr []int) UpdateValidatorSetData {
	num := len(curr)

	var validatorSet = []ValidatorData{}
	for i := 0; i < num; i++ {
		if curr[i] < 0 {
			continue
		}
		data := ProduceValidatorData(curr[i])
		validatorSet = append(validatorSet, data)
	}

	ret := UpdateValidatorSetData{
		SeqNum:       seqNum,
		SetId:        setId,
		ValidatorSet: validatorSet,
	}

	seqNum++
	setId++

	return ret
}

// write mock data to file which used to compare
func writeResult(vals []int, outfile string) error {

	file, err := os.Create(outfile)
	if err != nil {
		fmt.Println("writer", err)
		return err
	}
	defer file.Close()

	writer := bufio.NewWriter(file)
	for _, v := range vals {
		writer.WriteString(strconv.Itoa(v))
		writer.WriteString(",")
		writer.Flush()
	}

	writer.WriteString("\n")
	writer.Flush()

	return err
}

//case 1
func Test1() []interface{} {
	innerResult := []interface{}{}

	//This part will be replaced by random data
	var curr = []int{0, 1, 2}
	if writeResult(curr, "./test1.data") != nil {
		panic("Write data to file error in mock server!")
	}

	innerResult = append(innerResult,
		InnerResultValidatorSet{UpdateValidatorSet: ProduceUpdateValidatorSets(curr)})

	return innerResult
}

//case 2
func Test2() []interface{} {
	innerResult := []interface{}{}

	var curr = []int{2, 1, 3}
	innerResult = append(innerResult,
		InnerResultValidatorSet{UpdateValidatorSet: ProduceUpdateValidatorSets(curr)})

	return innerResult
}

var testData []interface{}
var once sync.Once

func ProduceResponse() Ret {
	//test use case
	once.Do(func() {
	    testData = Test1()
	})
	result, _ := json.Marshal(testData)

	retData := ResultData{
		BlockHash:   "EczErquQLMpUvTQpKupoQp5yNkgNbniMSHq1gVvhAf84", //mock hash
		BlockHeight: blockHeight2,
		Logs:        []string{},
		InnerResult: StringToInts(string(result)),
	}

	return Ret{
		Jsonrpc: "2.0",
		Id:      "dontcare",
		Result:  retData,
	}
}

type ParamsData struct {
	RequestType string
	Finality    string
	AccountId   string
	MethodName  string
	ArgsBase64  string
}

type Req struct {
	Jsonrpc string
	Id      string
	Method  string
	Params  ParamsData
}

var handleCnt uint64 = 0

const HANDLER_TIMES = 200

func Handler(w http.ResponseWriter, r *http.Request) {
	defer func() {
		if handleCnt > HANDLER_TIMES {
			os.Exit(1)
		}
		handleCnt++
	}()

	//fmt.Println("method:", r.Method)
	//body, err := ioutil.ReadAll(r.Body)
	//if err != nil {
	//	fmt.Printf("read body err, %v\n", err)
	//	return
	//}
	//println("request json:", string(body))

	////parse json to struct
	//var req Req
	//req.Jsonrpc = gjson.Get(string(body), "jsonrpc").String()
	//req.Id = gjson.Get(string(body), "id").String()
	//req.Method = gjson.Get(string(body), "method").String()
	//req.Params.RequestType = gjson.Get(string(body), "params.request_type").String()
	//req.Params.AccountId = gjson.Get(string(body), "params.account_id").String()
	//req.Params.MethodName = gjson.Get(string(body), "params.method_name").String()
	//req.Params.ArgsBase64 = gjson.Get(string(body), "params.args_base64").String()

	//fmt.Printf("%+v\n", req)

	//dealt the req
	response := ProduceResponse()

	w.Header().Set("content-type", "text/json")
	ret, err := json.Marshal(response)
	if err == nil {
		fmt.Printf("some error: %v\n", err)
	}

	w.Write(ret)
}

func main() {
	http.HandleFunc("/handler", Handler)

	err := http.ListenAndServe(":8080", nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}