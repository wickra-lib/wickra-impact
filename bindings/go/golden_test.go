package wickra

// The cross-language golden invariant seen from Go: the same request yields
// byte-identical output across calls and across instances. The response bytes are
// what every other binding produces too, because the whole fill engine lives once
// in the Rust core and this binding forwards its JSON verbatim.

import (
	"encoding/json"
	"testing"
)

func TestRunByteIdenticalAcrossInstances(t *testing.T) {
	cmd := runCmd()

	a, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer a.Close()
	b, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer b.Close()

	ra, err := a.Command(cmd)
	if err != nil {
		t.Fatal(err)
	}
	rb, err := b.Command(cmd)
	if err != nil {
		t.Fatal(err)
	}
	if ra != rb {
		t.Fatalf("expected byte-identical output, got:\n a: %s\n b: %s", ra, rb)
	}
}

func TestReportCarriesImpactStats(t *testing.T) {
	i, err := New(spec)
	if err != nil {
		t.Fatal(err)
	}
	defer i.Close()
	raw, err := i.Command(runCmd())
	if err != nil {
		t.Fatal(err)
	}
	var out struct {
		ImpactStats json.RawMessage `json:"impact_stats"`
	}
	if err := json.Unmarshal([]byte(raw), &out); err != nil {
		t.Fatal(err)
	}
	if len(out.ImpactStats) == 0 {
		t.Fatal("expected impact_stats in the report")
	}
}
