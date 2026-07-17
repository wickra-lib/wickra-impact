using System.Text.Json;
using Wickra.Impact;
using Xunit;

namespace WickraImpact.Tests;

public class ImpactTests
{
    internal const string Spec =
        "{\"strategy\":{\"spec_version\":1,\"symbol\":\"IMPACT\",\"timeframe\":\"1h\"," +
        "\"indicators\":{},\"entry\":{\"ge\":[{\"price\":\"close\"},0]},\"exit\":{\"in_position\":true}," +
        "\"sizing\":{\"type\":\"fixed_qty\",\"qty\":10.0}," +
        "\"execution\":{\"order_type\":\"market\",\"fill_timing\":\"next_open\"}}," +
        "\"book_model\":{\"kind\":\"orderbook_walk\"},\"participation_cap\":1.0,\"latency_ms\":0}";

    internal const string Data =
        "{\"candles\":[" +
        "{\"time\":0,\"open\":100,\"high\":100,\"low\":100,\"close\":100,\"volume\":1000}," +
        "{\"time\":3600,\"open\":100,\"high\":103,\"low\":100,\"close\":102,\"volume\":1000}]," +
        "\"books\":[" +
        "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[{\"price\":100.1,\"size\":100}]}," +
        "{\"bids\":[{\"price\":99.9,\"size\":100}],\"asks\":[" +
        "{\"price\":100.1,\"size\":3},{\"price\":100.3,\"size\":3},{\"price\":100.8,\"size\":4}]}]}";

    // A run command over the thin-book worked example.
    internal static string RunCmd() => "{\"cmd\":\"run\",\"data\":" + Data + "}";

    [Fact]
    public void Version_IsNonEmpty()
    {
        Assert.False(string.IsNullOrEmpty(Impact.Version()));
    }

    [Fact]
    public void Run_MeasuresImpact()
    {
        using var impact = new Impact(Spec);
        JsonElement outp = JsonDocument.Parse(impact.Command(RunCmd())).RootElement;

        // The walk sees the 44 bps of slippage a naive backtest hides.
        Assert.Equal(44.0, outp.GetProperty("impact_stats").GetProperty("avg_slippage_bps").GetDouble());
        Assert.Equal(100.44, outp.GetProperty("report").GetProperty("trades")[0].GetProperty("entry_price").GetDouble());
    }

    [Fact]
    public void InvalidSpec_Throws()
    {
        Assert.Throws<ArgumentException>(() => new Impact("{ not valid json"));
    }

    [Fact]
    public void SetSpec_ThenRun()
    {
        using var impact = new Impact("{}");
        string ok = impact.Command("{\"cmd\":\"set_spec\",\"spec\":" + Spec + "}");
        Assert.Contains("\"ok\":true", ok);
        Assert.Contains("impact_stats", impact.Command(RunCmd()));
    }
}
