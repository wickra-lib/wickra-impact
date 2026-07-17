package org.wickra.impact.examples;

import org.wickra.impact.Impact;

/**
 * A runnable Java example: back-test a buy-and-hold strategy against a thin order
 * book and print the market impact the walk measured.
 *
 * <pre>
 *   mvn -q compile exec:java -Dexec.mainClass=org.wickra.impact.examples.Run
 * </pre>
 *
 * Every language example runs the same thin_book request and prints the same
 * summary — that is the cross-language guarantee.
 */
public final class Run {
    private Run() {}

    private static final String SPEC =
            "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\","
                    + "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},"
                    + "\"exit\":{\"in_position\":true},"
                    + "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
                    + "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}},"
                    + "\"book_model\":{\"kind\":\"orderbook_walk\"},"
                    + "\"participation_cap\":1.0,\"latency_ms\":0}";

    // The thin_book worked example: the second bar's ask ladder is thin.
    private static final String RUN_CMD =
            "{\"cmd\":\"run\",\"data\":{\"candles\":["
                    + "{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
                    + "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}],"
                    + "\"books\":[{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]},"
                    + "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":3},"
                    + "{\"price\":100.3,\"size\":3},{\"price\":100.8,\"size\":4}]}]}}";

    public static void main(String[] args) {
        try (Impact impact = new Impact(SPEC)) {
            String report = impact.command(RUN_CMD);
            System.out.printf("wickra-impact %s%n", Impact.version());
            System.out.println(report.contains("\"avg_slippage_bps\":44.0")
                    ? "avg slippage: 44.0 bps" : "avg slippage: (see report)");
            System.out.println(report.contains("\"entry_price\":100.44")
                    ? "entry price: 100.44" : "entry price: (see report)");
        }
    }
}
