using Wickra.Impact;
using Xunit;

namespace WickraImpact.Tests;

/// <summary>
/// The book the walk needs, through the JSON command boundary.
///
/// There is no streaming half in IMPACT: a run is a batch computation. What
/// stands in its place is the property the whole repository exists for — that
/// slippage is measured rather than guessed — and the one way it can fail
/// quietly.
///
/// <c>orderbook_walk</c> needs a book. Without one the core refuses the run. A
/// binding that swallowed that refusal would hand back a report showing zero
/// slippage from the model whose entire purpose is to find some, and nothing
/// downstream could tell "no impact" from "no book".
/// </summary>
public class BookRequiredTests
{
    private const string Strategy =
        "{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\"," +
        "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]}," +
        "\"exit\":{\"in_position\":true}," +
        "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0}," +
        "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}}";

    private const string Candles =
        "[{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000}," +
        "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}]";

    private const string Books =
        "[{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]}," +
        "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[" +
        "{\"price\":100.1,\"size\":3},{\"price\":100.3,\"size\":3}," +
        "{\"price\":100.8,\"size\":4}]}]";

    private static string Spec(string model) =>
        "{\"strategy\":" + Strategy + ",\"book_model\":" + model +
        ",\"participation_cap\":1.0,\"latency_ms\":0}";

    private static string Run(string model, string data)
    {
        try
        {
            using var impact = new Impact(Spec(model));
            return impact.Command("{\"cmd\":\"run\",\"data\":" + data + "}");
        }
        catch (Exception err)
        {
            // The refusal may surface as an exception rather than an error
            // document; either is a refusal, which is what this test is about.
            return err.Message;
        }
    }

    [Fact]
    public void TheWalkRefusesARunWithNoBook()
    {
        string response = Run("{\"kind\":\"orderbook_walk\"}", "{\"candles\":" + Candles + "}");
        Assert.Contains("book", response);
        Assert.DoesNotContain("\"impact_stats\"", response);
    }

    [Fact]
    public void TheWalkMeasuresSlippageWhenTheBookIsThere()
    {
        string report = Run("{\"kind\":\"orderbook_walk\"}",
            "{\"candles\":" + Candles + ",\"books\":" + Books + "}");
        // Walking three ask levels for ten units pays 100.44, not the 100.10 top
        // of book a naive fill would take.
        Assert.Contains("\"avg_slippage_bps\":44.0", report);
    }

    [Fact]
    public void AnAnalyticModelNeedsNoBook()
    {
        string report = Run("{\"kind\":\"linear_impact\",\"coef\":0.1}",
            "{\"candles\":" + Candles + "}");
        Assert.Contains("\"impact_stats\"", report);
    }

    [Fact]
    public void TheBatchRunIsReproducible()
    {
        string data = "{\"candles\":" + Candles + ",\"books\":" + Books + "}";
        Assert.Equal(Run("{\"kind\":\"orderbook_walk\"}", data),
            Run("{\"kind\":\"orderbook_walk\"}", data));
    }
}
