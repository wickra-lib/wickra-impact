// A runnable C++ example: back-test a buy-and-hold strategy against a thin order
// book and print the market impact.
//
// This goes through `wickra_impact.hpp`, the C++ hull shipped beside the C
// header, because that hull is what a C++ caller is meant to use: it owns and
// frees the handle, runs the two-call length protocol behind
// `wickra_impact_command` for you, and turns a refusal into an exception rather
// than a negative integer that is easy to ignore. Calling the C functions
// directly from C++ works too -- `run.c` shows that -- but then the hull would
// be shipped without anything building it.
#include <cstdio>
#include <string>

#include "wickra_impact.hpp"

static const char *SPEC =
    "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\","
    "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},\"exit\":{\"in_position\":true},"
    "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
    "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}},"
    "\"book_model\":{\"kind\":\"orderbook_walk\"},\"participation_cap\":1.0,\"latency_ms\":0}";

static const char *RUN_CMD =
    "{\"cmd\":\"run\",\"data\":{\"candles\":["
    "{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
    "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}],"
    "\"books\":[{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]},"
    "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":3},"
    "{\"price\":100.3,\"size\":3},{\"price\":100.8,\"size\":4}]}]}}";

int main() {
    try {
        wickra::Impact impact(SPEC);
        const std::string report = impact.command(RUN_CMD);
        std::printf("wickra-impact %s\n", wickra::Impact::version().c_str());
        std::printf("report bytes: %d\n", static_cast<int>(report.size()));
    } catch (const wickra::ImpactError &err) {
        // Every failure arrives here: a spec the core rejects, a command it does
        // not know, a response that changed length between the two ABI calls.
        std::fprintf(stderr, "%s\n", err.what());
        return 1;
    }
    return 0;
}
