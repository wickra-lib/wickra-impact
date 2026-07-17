// A runnable Go example: back-test a buy-and-hold strategy against a thin order
// book and print the market impact the walk measured.
//
//	go run examples/go/run.go
//
// Every language example runs the same thin_book request and prints the same
// summary — that is the cross-language guarantee.
package main

import (
	"encoding/json"
	"fmt"

	wickra "github.com/wickra-lib/wickra-impact/bindings/go"
)

const spec = `{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",` +
	`"indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},` +
	`"sizing":{"type":"fixed_qty","qty":10.0},` +
	`"execution":{"order_type":"market","fill_timing":"next_open"}},` +
	`"book_model":{"kind":"orderbook_walk"},"participation_cap":1.0,"latency_ms":0}`

// The thin_book worked example: the second bar's ask ladder is thin, so a market
// order walks up it and pays slippage a naive backtest would never see.
const runCmd = `{"cmd":"run","data":{"candles":[` +
	`{"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},` +
	`{"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}],` +
	`"books":[{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},` +
	`{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":3},` +
	`{"price":100.3,"size":3},{"price":100.8,"size":4}]}]}}`

func main() {
	impact, err := wickra.New(spec)
	if err != nil {
		panic(err)
	}
	defer impact.Close()
	raw, err := impact.Command(runCmd)
	if err != nil {
		panic(err)
	}
	var report struct {
		Report struct {
			Trades []struct {
				EntryPrice float64 `json:"entry_price"`
			} `json:"trades"`
		} `json:"report"`
		ImpactStats struct {
			AvgSlippageBps float64 `json:"avg_slippage_bps"`
		} `json:"impact_stats"`
	}
	if err := json.Unmarshal([]byte(raw), &report); err != nil {
		panic(err)
	}
	fmt.Printf("wickra-impact %s\n", wickra.Version())
	fmt.Printf("avg slippage: %g bps\n", report.ImpactStats.AvgSlippageBps)
	fmt.Printf("entry price: %g\n", report.Report.Trades[0].EntryPrice)
}
