package wickra

import (
	"encoding/json"
	"strings"
	"testing"
)

const spec = `{"strategy":{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",` +
	`"indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},` +
	`"sizing":{"type":"fixed_qty","qty":10.0},` +
	`"execution":{"order_type":"market","fill_timing":"next_open"}},` +
	`"book_model":{"kind":"orderbook_walk"},"participation_cap":1.0,"latency_ms":0}`

const data = `{"candles":[` +
	`{"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},` +
	`{"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}],` +
	`"books":[` +
	`{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},` +
	`{"bids":[{"price":99.9,"size":100}],"asks":[` +
	`{"price":100.1,"size":3},{"price":100.3,"size":3},{"price":100.8,"size":4}]}]}`

// runCmd builds a run command over the thin-book worked example.
func runCmd() string {
	return `{"cmd":"run","data":` + data + `}`
}

type report struct {
	Report struct {
		Trades []struct {
			EntryPrice float64 `json:"entry_price"`
		} `json:"trades"`
	} `json:"report"`
	ImpactStats struct {
		AvgSlippageBps float64 `json:"avg_slippage_bps"`
	} `json:"impact_stats"`
}

func TestVersion(t *testing.T) {
	if Version() == "" {
		t.Fatal("empty version")
	}
}

func TestRunMeasuresImpact(t *testing.T) {
	i, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer i.Close()
	raw, err := i.Command(runCmd())
	if err != nil {
		t.Fatal(err)
	}
	var out report
	if err := json.Unmarshal([]byte(raw), &out); err != nil {
		t.Fatal(err)
	}
	// The walk sees the 44 bps of slippage a naive backtest hides.
	if out.ImpactStats.AvgSlippageBps != 44.0 {
		t.Fatalf("expected 44 bps of slippage, got %v: %s", out.ImpactStats.AvgSlippageBps, raw)
	}
	if len(out.Report.Trades) == 0 || out.Report.Trades[0].EntryPrice != 100.44 {
		t.Fatalf("expected an entry price of 100.44, got: %s", raw)
	}
}

func TestInvalidSpecIsError(t *testing.T) {
	if _, err := New("{ not valid json"); err == nil {
		t.Fatal("expected an error for an invalid spec")
	}
}

func TestSetSpecThenRun(t *testing.T) {
	i, err := New("{}")
	if err != nil {
		t.Fatal(err)
	}
	defer i.Close()
	ok, err := i.Command(`{"cmd":"set_spec","spec":` + spec + `}`)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(ok, `"ok":true`) {
		t.Fatalf("expected ok:true, got: %s", ok)
	}
	if _, err := i.Command(runCmd()); err != nil {
		t.Fatal(err)
	}
}
