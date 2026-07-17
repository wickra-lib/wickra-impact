using System.Text.Json;
using Wickra.Impact;
using Xunit;

namespace WickraImpact.Tests;

// The cross-language golden invariant seen from C#: the same request yields
// byte-identical output across calls and across instances. The response bytes are
// what every other binding produces too, because the whole fill engine lives once
// in the Rust core and this binding forwards its JSON verbatim.
public class GoldenTests
{
    [Fact]
    public void Run_IsByteIdenticalAcrossInstances()
    {
        string cmd = ImpactTests.RunCmd();
        using var a = new Impact(ImpactTests.Spec);
        using var b = new Impact(ImpactTests.Spec);
        Assert.Equal(a.Command(cmd), b.Command(cmd));
    }

    [Fact]
    public void Report_CarriesImpactStats()
    {
        using var impact = new Impact(ImpactTests.Spec);
        JsonElement outp = JsonDocument.Parse(impact.Command(ImpactTests.RunCmd())).RootElement;
        Assert.Equal(44.0, outp.GetProperty("impact_stats").GetProperty("avg_slippage_bps").GetDouble());
    }
}
