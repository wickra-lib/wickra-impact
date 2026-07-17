/* A runnable C example: back-test a buy-and-hold strategy against a thin order
 * book through the wickra-impact C ABI and print the market impact. Every
 * language example runs the same thin_book request and prints the same summary. */
#include <stdio.h>
#include <stdlib.h>

#include "wickra_impact.h"

static const char *SPEC =
    "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\","
    "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},\"exit\":{\"in_position\":true},"
    "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
    "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}},"
    "\"book_model\":{\"kind\":\"orderbook_walk\"},\"participation_cap\":1.0,\"latency_ms\":0}";

/* The thin_book worked example: the second bar's ask ladder is thin. */
static const char *RUN_CMD =
    "{\"cmd\":\"run\",\"data\":{\"candles\":["
    "{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
    "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}],"
    "\"books\":[{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]},"
    "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":3},"
    "{\"price\":100.3,\"size\":3},{\"price\":100.8,\"size\":4}]}]}}";

int main(void) {
    WickraImpact *impact = wickra_impact_new(SPEC);
    if (!impact) {
        fprintf(stderr, "failed to build impact\n");
        return 1;
    }
    int len = wickra_impact_command(impact, RUN_CMD, NULL, 0);
    if (len < 0) {
        fprintf(stderr, "command failed: code %d\n", len);
        wickra_impact_free(impact);
        return 1;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        wickra_impact_free(impact);
        return 1;
    }
    wickra_impact_command(impact, RUN_CMD, buf, (size_t)len + 1);

    printf("wickra-impact %s\n", wickra_impact_version());
    printf("report bytes: %d\n", len);

    free(buf);
    wickra_impact_free(impact);
    return 0;
}
