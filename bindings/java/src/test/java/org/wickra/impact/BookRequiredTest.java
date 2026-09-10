package org.wickra.impact;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * The book the walk needs, through the JSON command boundary.
 *
 * <p>There is no streaming half in IMPACT: a run is a batch computation. What
 * stands in its place is the property the whole repository exists for — that
 * slippage is measured rather than guessed — and the one way it can fail
 * quietly.
 *
 * <p>{@code orderbook_walk} needs a book. Without one the core refuses the run.
 * A binding that swallowed that refusal would hand back a report showing zero
 * slippage from the model whose entire purpose is to find some, and nothing
 * downstream could tell "no impact" from "no book".
 */
class BookRequiredTest {
    private static final String STRATEGY =
            "{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\","
                    + "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},"
                    + "\"exit\":{\"in_position\":true},"
                    + "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
                    + "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}}";

    private static final String CANDLES =
            "[{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
                    + "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,"
                    + "\"volume\":1000}]";

    private static final String BOOKS =
            "[{\"bids\":[{\"price\":99.9,\"size\":100}],"
                    + "\"asks\":[{\"price\":100.1,\"size\":100}]},"
                    + "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":["
                    + "{\"price\":100.1,\"size\":3},{\"price\":100.3,\"size\":3},"
                    + "{\"price\":100.8,\"size\":4}]}]";

    private static String spec(String model) {
        return "{\"strategy\":" + STRATEGY + ",\"book_model\":" + model
                + ",\"participation_cap\":1.0,\"latency_ms\":0}";
    }

    private static String run(String model, String data) {
        try (Impact impact = new Impact(spec(model))) {
            return impact.command("{\"cmd\":\"run\",\"data\":" + data + "}");
        } catch (RuntimeException err) {
            // The refusal may surface as an exception rather than an error
            // document; either is a refusal, which is what this test is about.
            return String.valueOf(err.getMessage());
        }
    }

    @Test
    void theWalkRefusesARunWithNoBook() {
        String response = run("{\"kind\":\"orderbook_walk\"}", "{\"candles\":" + CANDLES + "}");
        assertTrue(response.contains("book"), response);
        assertFalse(response.contains("\"impact_stats\""),
                "a walk with no book must be refused, not answered with zero slippage");
    }

    @Test
    void theWalkMeasuresSlippageWhenTheBookIsThere() {
        String report = run("{\"kind\":\"orderbook_walk\"}",
                "{\"candles\":" + CANDLES + ",\"books\":" + BOOKS + "}");
        // Walking three ask levels for ten units pays 100.44, not the 100.10 top
        // of book a naive fill would take.
        assertTrue(report.contains("\"avg_slippage_bps\":44.0"), report);
    }

    @Test
    void anAnalyticModelNeedsNoBook() {
        String report = run("{\"kind\":\"linear_impact\",\"coef\":0.1}",
                "{\"candles\":" + CANDLES + "}");
        assertTrue(report.contains("\"impact_stats\""),
                "linear_impact prices from a curve, not a book: " + report);
    }

    @Test
    void theBatchRunIsReproducible() {
        String data = "{\"candles\":" + CANDLES + ",\"books\":" + BOOKS + "}";
        assertEquals(run("{\"kind\":\"orderbook_walk\"}", data),
                run("{\"kind\":\"orderbook_walk\"}", data));
    }
}
