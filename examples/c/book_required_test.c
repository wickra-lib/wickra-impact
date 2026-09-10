/* The book the walk needs, through the C ABI's two-call idiom.
 *
 * There is no streaming half in IMPACT: a run is a batch computation. What
 * stands in its place is the property the whole repository exists for — that
 * slippage is measured rather than guessed — and the one way it can fail
 * quietly.
 *
 * `orderbook_walk` needs a book. Without one the core refuses the run. A caller
 * that read past the refusal would take a report showing zero slippage from the
 * model whose entire purpose is to find some, and nothing downstream could tell
 * "no impact" from "no book".
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wickra_impact.h"

static const char *STRATEGY =
    "{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\",\"indicators\":{},"
    "\"entry\":{\"ge\":[{\"price\":\"close\"},0]},\"exit\":{\"in_position\":true},"
    "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0},"
    "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}}";

static const char *CANDLES =
    "[{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000},"
    "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}]";

static const char *BOOKS =
    "[{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]},"
    "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":["
    "{\"price\":100.1,\"size\":3},{\"price\":100.3,\"size\":3},"
    "{\"price\":100.8,\"size\":4}]}]";

/* Run one command through the documented two-call idiom. Caller frees; NULL
 * means the ABI itself refused, which is one of the two shapes a refusal takes. */
static char *run_cmd(WickraImpact *handle, const char *cmd) {
    int32_t len = wickra_impact_command(handle, cmd, NULL, 0);
    if (len < 0) {
        return NULL;
    }
    char *buf = (char *)malloc((size_t)len + 1);
    if (!buf) {
        return NULL;
    }
    if (wickra_impact_command(handle, cmd, buf, (size_t)len + 1) != len) {
        free(buf);
        return NULL;
    }
    return buf;
}

/* Run `model` over `data`. Returns the response, or NULL if the ABI refused.
 * Caller frees. */
static char *run(const char *model, const char *data) {
    size_t scap = strlen(STRATEGY) + strlen(model) + 128;
    char *spec = (char *)malloc(scap);
    if (!spec) {
        return NULL;
    }
    snprintf(spec, scap,
             "{\"strategy\":%s,\"book_model\":%s,\"participation_cap\":1.0,\"latency_ms\":0}",
             STRATEGY, model);

    WickraImpact *handle = wickra_impact_new(spec);
    free(spec);
    if (!handle) {
        return NULL;
    }

    size_t ccap = strlen(data) + 64;
    char *cmd = (char *)malloc(ccap);
    if (!cmd) {
        wickra_impact_free(handle);
        return NULL;
    }
    snprintf(cmd, ccap, "{\"cmd\":\"run\",\"data\":%s}", data);
    char *out = run_cmd(handle, cmd);
    free(cmd);
    wickra_impact_free(handle);
    return out;
}

/* `{"candles": …}` with no books. Caller frees. */
static char *bookless(void) {
    size_t cap = strlen(CANDLES) + 32;
    char *out = (char *)malloc(cap);
    if (out) {
        snprintf(out, cap, "{\"candles\":%s}", CANDLES);
    }
    return out;
}

/* `{"candles": …, "books": …}`. Caller frees. */
static char *with_books(void) {
    size_t cap = strlen(CANDLES) + strlen(BOOKS) + 48;
    char *out = (char *)malloc(cap);
    if (out) {
        snprintf(out, cap, "{\"candles\":%s,\"books\":%s}", CANDLES, BOOKS);
    }
    return out;
}

int main(void) {
    int failures = 0;

    char *no_books = bookless();
    char *books = with_books();
    if (!no_books || !books) {
        fprintf(stderr, "could not build the inputs\n");
        free(no_books);
        free(books);
        return 1;
    }

    /* A walk with no book is refused, not answered with zero slippage. */
    char *refused = run("{\"kind\":\"orderbook_walk\"}", no_books);
    if (refused == NULL) {
        printf("refused at the ABI, as expected\n");
    } else {
        printf("refused: %s\n", refused);
        if (strstr(refused, "book") == NULL) {
            fprintf(stderr, "the refusal must name the book\n");
            failures++;
        }
        if (strstr(refused, "\"impact_stats\"") != NULL) {
            fprintf(stderr, "a walk with no book must be refused, not answered\n");
            failures++;
        }
        free(refused);
    }

    /* With the book, the walk pays 100.44 rather than the 100.10 top of book. */
    char *measured = run("{\"kind\":\"orderbook_walk\"}", books);
    if (!measured) {
        fprintf(stderr, "the walk did not run with a book present\n");
        failures++;
    } else {
        if (strstr(measured, "\"avg_slippage_bps\":44") == NULL) {
            fprintf(stderr, "expected 44 bps of slippage: %.200s\n", measured);
            failures++;
        }
        free(measured);
    }

    /* An analytic model prices from a curve and needs no book. */
    char *analytic = run("{\"kind\":\"linear_impact\",\"coef\":0.1}", no_books);
    if (!analytic || strstr(analytic, "\"impact_stats\"") == NULL) {
        fprintf(stderr, "linear_impact prices from a curve, not a book\n");
        failures++;
    }
    free(analytic);

    /* The batch run is reproducible. */
    char *first = run("{\"kind\":\"orderbook_walk\"}", books);
    char *second = run("{\"kind\":\"orderbook_walk\"}", books);
    if (!first || !second || strcmp(first, second) != 0) {
        fprintf(stderr, "the same inputs produced two different runs\n");
        failures++;
    }
    free(first);
    free(second);

    free(no_books);
    free(books);

    if (failures > 0) {
        fprintf(stderr, "%d check(s) failed\n", failures);
        return 1;
    }
    printf("the walk needs its book, and measures with it\n");
    return 0;
}
