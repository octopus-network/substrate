package main

import (
	"bufio"
	"fmt"
	"io/ioutil"
	"log"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"sync"
	"time"

	"encoding/base64"
	"encoding/json"

	"./mockdataproducer"
	producer "./mockdataproducer"
	"github.com/tidwall/gjson"
)

// write mock data to file which used to compare
func writeResult(vals [][]int, outfile string) error {
	file, err := os.Create(outfile)
	if err != nil {
		fmt.Println("writer", err)
		return err
	}
	defer file.Close()

	writer := bufio.NewWriter(file)
	for _, v1 := range vals {
		for _, v2 := range v1 {
			writer.WriteString(strconv.Itoa(v2))
			writer.WriteString(",")
			writer.Flush()
		}
		writer.WriteString("\n")
		writer.Flush()
	}

	return err
}

//First item in every line is data type:
//1: validator sets
//2: lockToken
//3: burn event
//For example:
//simulationSequence = {
//	{1, 1, 2, 3},   // validator sets
//  {2, ...},		// locktoken
//  {3, ...}		// burn event
//}

//case 1
func Test1() [][]int {
	mockData := [][]int{
		{1, 0, 1, 2, 3},
		{1, 0, 1, 2},
		{1, 0, 1},
		{1, 3, 2, 1},
		{1, 4, 1, 2, 3},
		{1, 0, 1, 2, 3, 4},
		{1, 0, 3, 2, 4},
		{1, 0, 3, 1, 4},
	}

	return mockData
}

var testData [][]int
var once sync.Once
var currRet producer.Ret
var preTime int64 = 0
var startLine int = 0
var endLine int = 0
var globalEraNumber = 0

func ProduceValidatorResponse(eraNumber uint32) producer.Ret {
	//test use case
	once.Do(func() {
		testData = Test1()
	})

	//produce responce data
	currTime := time.Now().Unix()
	deltTime := currTime - preTime
	if (preTime == 0) || (deltTime > 60*10 && endLine < len(testData)) {
		rand.Seed(time.Now().UnixNano())
		// delt := rand.Intn(len(testData))
		delt := 1
		endLine = startLine + delt
		if endLine > len(testData) {
			endLine = len(testData)
		}

		currRet = producer.ProduceNewResponseForValidatorSets(testData[startLine:endLine])
		globalEraNumber++
		fmt.Printf("start: %v, end: %v\n", startLine, endLine)
		startLine = endLine
		preTime = currTime
	}

	if eraNumber == uint32(globalEraNumber) {
		return currRet
	}

	return producer.ProduceEmptyResponseForValidatorSets()
}

var currNotifyRet producer.Ret
var preTimeForNotify int64 = 0

func ProduceNotifyResponse(startIndex uint32, quantity uint32) producer.Ret {
	fmt.Println("startIndex = ", startIndex, ", quantity = ", quantity)

	//test use case
	testData := []producer.SimulationData{}
	s1 := producer.SimulationData{
		Receiver: producer.PresetValidators[0].Id,
		Amount:   "10000",
		DataType: producer.BurntToken,
	}
	s2 := producer.SimulationData{
		Receiver: producer.PresetValidators[1].Id,
		Amount:   "10000000",
		DataType: producer.BurntToken,
	}
	s3 := producer.SimulationData{
		Receiver: producer.PresetValidators[2].Id,
		Amount:   "10000000",
		DataType: producer.BurntToken,
	}
	s4 := producer.SimulationData{
		Receiver: producer.PresetValidators[3].Id,
		Amount:   "10000",
		DataType: producer.BurntToken,
	}
	s5 := producer.SimulationData{
		Receiver: producer.PresetValidators[1].Id,
		Amount:   "20000",
		DataType: producer.BurntToken,
	}

	currTime := time.Now().Unix()
	deltTime := currTime - preTimeForNotify
	if (preTimeForNotify == 0) || (deltTime > 60*1) {
		rand.Seed(time.Now().UnixNano())
		flag := rand.Intn(10000)

		// if flag%3 == 0 {
		// 	testData = append(testData, s1)
		// 	testData = append(testData, s2)
		// 	testData = append(testData, s3)
		// } else if flag%5 == 0 {
		// 	testData = append(testData, s3)
		// 	testData = append(testData, s3)
		// 	testData = append(testData, s3)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)

		// } else if flag%7 == 0 {
		// 	testData = append(testData, s1)
		// 	testData = append(testData, s2)
		// 	testData = append(testData, s3)
		// 	testData = append(testData, s3)
		// 	testData = append(testData, s3)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s5)

		// } else if flag%2 == 0 {
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)
		// 	testData = append(testData, s4)

		// }

		// if flag%2 == 0 {
		testData = append(testData, s1)
		testData = append(testData, s2)
		testData = append(testData, s3)
		testData = append(testData, s3)
		testData = append(testData, s3)
		testData = append(testData, s4)
		testData = append(testData, s4)
		testData = append(testData, s4)
		testData = append(testData, s4)
		testData = append(testData, s5)
		fmt.Println("1 +++++++++++++++++++++++, flag = ", flag)
		// } else if flag%3 != 0 {
		// 	testData = append(testData, s1)
		// 	testData = append(testData, s2)
		// 	testData = append(testData, s3)
		// 	fmt.Println("2 +++++++++++++++++++++++, flag = ", flag)
		// }

		currNotifyRet = producer.ProduceNewResponseForNotifyHistories(testData)
		preTimeForNotify = currTime
		fmt.Println("3 +++++++++++++++++++++++, flag = ", flag)
	}

	if (mockdataproducer.Index > 2) && ((startIndex > uint32(mockdataproducer.Index-1)) || (startIndex+quantity < uint32(mockdataproducer.Index-1))) {
		fmt.Println("0 +++++++++++++++++++++++, innerIndex = ", mockdataproducer.Index)
		emptyData := []producer.SimulationData{}
		return producer.ProduceNewResponseForNotifyHistories(emptyData)
	}

	fmt.Println("4 +++++++++++++++++++++++")

	return currNotifyRet
}

type ParamsData struct {
	RequestType string `json:"request_type"`
	Finality    string `json:"finalty"`
	AccountId   string `json:"account_id"`
	MethodName  string `json:"method_name"`
	ArgsBase64  string `json:"args_base64"`
}

type Req struct {
	Jsonrpc string
	Id      string
	Method  string
	Params  ParamsData
}

var handleCnt uint64 = 0

const HANDLER_TIMES = 200000

func DecodeFromBase64(data string) (string, error) {
	sDec, err := base64.StdEncoding.DecodeString(data)
	if err != nil {
		fmt.Printf("Error decoding string: %s ", err.Error())
		return "", err
	}

	return string(sDec), nil
}

func ParseBody(body []byte) map[string]interface{} {
	var req Req
	req.Jsonrpc = gjson.Get(string(body), "jsonrpc").String()
	req.Id = gjson.Get(string(body), "id").String()
	req.Method = gjson.Get(string(body), "method").String()
	req.Params.RequestType = gjson.Get(string(body), "params.request_type").String()
	req.Params.AccountId = gjson.Get(string(body), "params.account_id").String()
	req.Params.MethodName = gjson.Get(string(body), "params.method_name").String()
	req.Params.ArgsBase64 = gjson.Get(string(body), "params.args_base64").String()

	// fmt.Printf("%+v\n", req)

	s, err := DecodeFromBase64(req.Params.ArgsBase64)
	if err != nil {
		fmt.Println("Parse error")
	}

	res := make(map[string]interface{})
	res["method_name"] = req.Params.MethodName
	switch req.Params.MethodName {
	case "get_appchain_notification_histories":
		res["start_index"] = gjson.Get(s, "start_index").Uint()
		res["quantity"] = gjson.Get(s, "quantity").Uint()
	case "get_validator_list_of":
		res["era_number"] = gjson.Get(s, "era_number").Uint()
	default:
		fmt.Println("Not found match method!")
	}

	return res
}

func Handler(w http.ResponseWriter, r *http.Request) {
	defer func() {
		if handleCnt > HANDLER_TIMES {
			os.Exit(1)
		}
		handleCnt++
	}()

	// fmt.Println("method:", r.Method)
	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		fmt.Printf("read body err, %v\n", err)
		return
	}
	// println("request json:", string(body))

	//parse request
	pb := ParseBody(body)
	// fmt.Println("pb === ", pb)

	//dealt the req
	var response producer.Ret
	switch pb["method_name"] {
	case "get_validator_list_of":
		response = ProduceValidatorResponse(uint32(pb["era_number"].(uint64)))
	case "get_appchain_notification_histories":
		response = ProduceNotifyResponse(uint32(pb["start_index"].(uint64)), uint32(pb["quantity"].(uint64)))
	default:
		fmt.Println("Not found match method!")
	}

	w.Header().Set("content-type", "text/json")
	ret, err := json.Marshal(response)
	if err == nil {
		fmt.Printf("some error: %v\n", err)
	}

	w.Write(ret)
	fmt.Printf("Handle one request, time: %v\n", time.Now())
}

func main() {
	mockData := Test1()
	if writeResult(mockData, "./test1.data") != nil {
		panic("Write data to file error in mock server!")
	}

	http.HandleFunc("/handler", Handler)

	err := http.ListenAndServe(":8080", nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}
