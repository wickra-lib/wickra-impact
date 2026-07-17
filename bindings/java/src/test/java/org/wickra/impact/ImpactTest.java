package org.wickra.impact;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

class ImpactTest {
    static final String SPEC =
            "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\","
                    + "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},"
                    + "\"exit\":{\"in_position\":true},"
                    + "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
                    + "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}},"
                    + "\"book_model\":{\"kind\":\"orderbook_walk\"},"
                    + "\"participation_cap\":1.0,\"latency_ms\":0}";

    static final String DATA =
            "{\"candles\":["
                    + "{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
                    + "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}],"
                    + "\"books\":["
                    + "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]},"
                    + "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":["
                    + "{\"price\":100.1,\"size\":3},{\"price\":100.3,\"size\":3},"
                    + "{\"price\":100.8,\"size\":4}]}]}";

    // A run command over the thin-book worked example.
    static String runCmd() {
        return "{\"cmd\":\"run\",\"data\":" + DATA + "}";
    }

    @Test
    void versionIsNonEmpty() {
        assertFalse(Impact.version().isEmpty());
    }

    @Test
    void runMeasuresImpact() {
        try (Impact impact = new Impact(SPEC)) {
            String out = impact.command(runCmd());
            // The walk sees the 44 bps of slippage a naive backtest hides.
            assertTrue(out.contains("\"avg_slippage_bps\":44.0"), out);
            assertTrue(out.contains("\"entry_price\":100.44"), out);
        }
    }

    @Test
    void invalidSpecThrows() {
        assertThrows(IllegalArgumentException.class, () -> new Impact("{ not valid json"));
    }

    @Test
    void setSpecThenRun() {
        try (Impact impact = new Impact("{}")) {
            String ok = impact.command("{\"cmd\":\"set_spec\",\"spec\":" + SPEC + "}");
            assertTrue(ok.contains("\"ok\":true"), ok);
            assertTrue(impact.command(runCmd()).contains("\"impact_stats\""));
        }
    }
}
