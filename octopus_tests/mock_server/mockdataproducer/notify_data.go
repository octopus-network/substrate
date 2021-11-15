package mockdataproducer

import (
	"encoding/json"
	"time"
)

type LockTokenData struct {
	Symbol   string `json:"symbol"`
	SenderId string `json:"sender_id_in_near"`
	Receiver string `json:"receiver_id_in_appchain"`
	Amount   string `json:"amount"`
}

type TokenBurntData struct {
	SenderId string `json:"sender_id_in_near"`
	Receiver string `json:"receiver_id_in_appchain"`
	Amount   string `json:"amount"`
}

type AppchainNotificationHistory struct {
	Notification interface{} `json:"appchain_notification"`
	BlockHeight  uint64      `json:"block_height"`
	TimeStamp    uint64      `json:"timestamp"`
	Index        uint32      `json:"index"`
}

type MockDataType int32

const (
	LockToken MockDataType = iota
	BurntToken
)

func ProduceLockTokenData(receiver string, amount string) LockTokenData {
	data := LockTokenData{
		Symbol:   "mock.token",
		SenderId: "mock-sender.testnet",
		Receiver: receiver,
		Amount:   amount,
	}

	return data
}

func ProduceTokenBurntData(receiver string, amount string) TokenBurntData {
	data := TokenBurntData{
		SenderId: "mock-sender.testnet",
		Receiver: receiver,
		Amount:   amount,
	}

	return data
}

type MockInfo struct {
	Receiver    string
	Amount      string
	BlockHeight uint64
	TimeStamp   uint64
	Index       uint32
}

func ProduceNotificationHistory(mockInfo MockInfo, dataType MockDataType) AppchainNotificationHistory {
	var notify interface{}
	switch dataType {
	case LockToken:
		notify = ProduceLockTokenData(mockInfo.Receiver, mockInfo.Amount)
	case BurntToken:
		notify = ProduceTokenBurntData(mockInfo.Receiver, mockInfo.Amount)
	default:
		panic("Mock data type is error")
	}

	history := AppchainNotificationHistory{
		Notification: notify,
		BlockHeight:  mockInfo.BlockHeight,
		TimeStamp:    mockInfo.TimeStamp,
		Index:        mockInfo.Index,
	}

	return history
}

type SimulationData struct {
	Receiver string
	Amount   string
	DataType MockDataType
}

var Index uint32 = 0

func ProduceNotificationHistories(testData []SimulationData) []AppchainNotificationHistory {
	histories := []AppchainNotificationHistory{}
	for i := 0; i < len(testData); i++ {
		ti := time.Now().Unix()
		BlockHeight++
		Index++
		mockInfo := MockInfo{
			Receiver:    testData[i].Receiver,
			Amount:      testData[i].Amount,
			BlockHeight: BlockHeight,
			TimeStamp:   uint64(ti),
			Index:       Index,
		}
		history := ProduceNotificationHistory(mockInfo, testData[i].DataType)
		histories = append(histories, history)
	}

	return histories
}

func ProduceNewResponseForNotifyHistories(testData []SimulationData) Ret {
	var innerResult interface{}
	innerResult = ProduceNewResponseForNotifyHistories(testData)

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
