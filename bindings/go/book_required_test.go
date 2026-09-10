package wickra

// The book the walk needs, through the JSON command boundary.
//
// There is no streaming half in IMPACT: a run is a batch computation. What stands
// in its place is the property the whole repository exists for -- that slippage is
// measured rather than guessed -- and the one way it can fail quietly.
//
// `orderbook_walk` needs a book. Without one the core refuses the run. A binding
// that swallowed that refusal would hand back a report showing zero slippage from
// the model whose entire purpose is to find some, and nothing downstream could
// tell the difference between "no impact" and "no book".

import (
	"encoding/json"
	"strings"
	"testing"
)

const bookStrategy = `{"spec_version":1,"symbol":"IMPACT","timeframe":"1h",` +
	`"indicators":{},"entry":{"ge":[{"price":"close"},0]},"exit":{"in_position":true},` +
	`"sizing":{"type":"fixed_qty","qty":10.0},` +
	`"execution":{"order_type":"market","fill_timing":"next_open"}}`

const bookCandles = `[` +
	`{"time":0,"open":100,"high":100,"low":100,"close":100,"volume":1000},` +
	`{"time":3600,"open":100,"high":103,"low":100,"close":102,"volume":1000}]`

const bookLevels = `[` +
	`{"bids":[{"price":99.9,"size":100}],"asks":[{"price":100.1,"size":100}]},` +
	`{"bids":[{"price":99.9,"size":100}],"asks":[` +
	`{"price":100.1,"size":3},{"price":100.3,"size":3},{"price":100.8,"size":4}]}]`

func bookSpec(model string) string {
	return `{"strategy":` + bookStrategy + `,"book_model":` + model +
		`,"participation_cap":1.0,"latency_ms":0}`
}

func bookRun(t *testing.T, model, data string) string {
	t.Helper()
	h, err := New(bookSpec(model))
	if err != nil {
		t.Fatalf("new impact: %v", err)
	}
	defer h.Close()
	out, err := h.Command(`{"cmd":"run","data":` + data + `}`)
	if err != nil {
		// The refusal may surface as an error rather than an error document;
		// either is a refusal, which is what this test is about.
		return err.Error()
	}
	return out
}

func TestTheWalkRefusesARunWithNoBook(t *testing.T) {
	out := bookRun(t, `{"kind":"orderbook_walk"}`, `{"candles":`+bookCandles+`}`)
	if !strings.Contains(out, "book") {
		t.Errorf("the refusal must name the book: %s", out)
	}
	if strings.Contains(out, `"impact_stats"`) {
		t.Error("a walk with no book must be refused, not answered with zero slippage")
	}
}

func TestTheWalkMeasuresSlippageWhenTheBookIsThere(t *testing.T) {
	out := bookRun(t, `{"kind":"orderbook_walk"}`,
		`{"candles":`+bookCandles+`,"books":`+bookLevels+`}`)
	var report struct {
		ImpactStats struct {
			AvgSlippageBps float64 `json:"avg_slippage_bps"`
		} `json:"impact_stats"`
	}
	if err := json.Unmarshal([]byte(out), &report); err != nil {
		t.Fatalf("parse report: %v (%s)", err, out)
	}
	// Walking three ask levels for ten units pays 100.44, not the 100.10 top of
	// book a naive fill would take.
	if report.ImpactStats.AvgSlippageBps != 44.0 {
		t.Errorf("expected 44 bps of slippage, got %v", report.ImpactStats.AvgSlippageBps)
	}
}

func TestAnAnalyticModelNeedsNoBook(t *testing.T) {
	out := bookRun(t, `{"kind":"linear_impact","coef":0.1}`, `{"candles":`+bookCandles+`}`)
	if !strings.Contains(out, `"impact_stats"`) {
		t.Errorf("linear_impact prices from a curve, not a book: %s", out)
	}
}

func TestTheBatchRunIsReproducible(t *testing.T) {
	data := `{"candles":` + bookCandles + `,"books":` + bookLevels + `}`
	if bookRun(t, `{"kind":"orderbook_walk"}`, data) != bookRun(t, `{"kind":"orderbook_walk"}`, data) {
		t.Error("the same inputs produced two different runs")
	}
}
