package mockdataproducer

import (
	"encoding/json"
	"fmt"
	"time"
)

type MockDataType int32

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

type WrappedAppchainTokenBurnt struct {
	Data TokenBurntData `json:"WrappedAppchainTokenBurnt"`
}

type NearFungibleTokenLocked struct {
	Data LockTokenData `json:"NearFungibleTokenLocke"`
}

type AppchainNotificationHistory struct {
	Notification interface{} `json:"appchain_notification"`
	BlockHeight  uint64      `json:"block_height"`
	TimeStamp    uint64      `json:"timestamp"`
	Index        string      `json:"index"`
}

const (
	LockToken MockDataType = iota
	BurntToken
)

func ProduceLockTokenData(receiver string, amount string) NearFungibleTokenLocked {
	data := LockTokenData{
		Symbol:   "mock.token",
		SenderId: "mock-sender.testnet",
		Receiver: receiver,
		Amount:   amount,
	}

	notify := NearFungibleTokenLocked{data}
	return notify
}

func ProduceTokenBurntData(receiver string, amount string) WrappedAppchainTokenBurnt {
	data := TokenBurntData{
		SenderId: "mock-sender.testnet",
		Receiver: receiver,
		Amount:   amount,
	}
	notify := WrappedAppchainTokenBurnt{data}

	return notify
}

type MockInfo struct {
	Receiver    string
	Amount      string
	BlockHeight uint64
	TimeStamp   uint64
	Index       string
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

var Index int = 0

func ProduceNotificationHistories(testData []SimulationData) []AppchainNotificationHistory {
	histories := []AppchainNotificationHistory{}
	for i := 0; i < len(testData); i++ {
		ti := time.Now().Unix()
		BlockHeight++
		str := fmt.Sprintf("%d", Index)
		mockInfo := MockInfo{
			Receiver:    testData[i].Receiver,
			Amount:      testData[i].Amount,
			BlockHeight: BlockHeight,
			TimeStamp:   uint64(ti),
			Index:       str,
		}
		history := ProduceNotificationHistory(mockInfo, testData[i].DataType)
		histories = append(histories, history)
		Index++
	}

	return histories
}

func ProduceNewResponseForNotifyHistories(testData []SimulationData) Ret {
	innerResult := ProduceNotificationHistories(testData)
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
