using System.Runtime.InteropServices;
using System.Text;

namespace Wickra.Impact;

/// <summary>
/// A market-impact backtest driven by JSON commands, built from a spec,
/// over the Wickra C ABI. Construct one from a spec JSON, drive it with command
/// JSON (<c>set_spec</c>, <c>run</c>, <c>version</c>) and read
/// back the response JSON — the same protocol as the CLI and every other binding.
/// </summary>
public sealed class Impact : IDisposable
{
    private readonly ImpactHandle _handle;

    /// <summary>Build a backtest handle from a spec JSON (<c>"{}"</c> defers to a later <c>set_spec</c>).</summary>
    /// <exception cref="ArgumentException">The spec was not a valid backtest spec.</exception>
    public Impact(string specJson)
    {
        IntPtr ptr = Native.wickra_impact_new(Utf8(specJson));
        if (ptr == IntPtr.Zero)
        {
            throw new ArgumentException("wickra-impact: invalid spec", nameof(specJson));
        }
        _handle = new ImpactHandle(ptr);
    }

    /// <summary>Apply a command JSON and return the response JSON.</summary>
    /// <remarks>
    /// Uses the C ABI's length-out protocol: a first call learns the length, then
    /// the response is read into a caller-owned buffer. Domain errors (a bad
    /// command, an unknown command) come back in-band as <c>{"ok":false,...}</c>
    /// JSON, not as an exception.
    /// </remarks>
    /// <exception cref="InvalidOperationException">A required argument was unusable or a panic was caught.</exception>
    public string Command(string cmdJson)
    {
        ObjectDisposedException.ThrowIf(_handle.IsInvalid, this);

        byte[] cmd = Utf8(cmdJson);
        IntPtr h = _handle.DangerousGetHandle();
        int n = Native.wickra_impact_command(h, cmd, null, 0);
        if (n < 0)
        {
            throw new InvalidOperationException($"wickra-impact: command failed (code {n})");
        }
        var buf = new byte[n + 1];
        Native.wickra_impact_command(h, cmd, buf, (nuint)buf.Length);
        return Encoding.UTF8.GetString(buf, 0, n);
    }

    /// <summary>The library version.</summary>
    public static string Version() =>
        Marshal.PtrToStringUTF8(Native.wickra_impact_version()) ?? string.Empty;

    /// <summary>Free the native backtest handle.</summary>
    public void Dispose() => _handle.Dispose();

    /// <summary>Encode a string as NUL-terminated UTF-8 for the C ABI.</summary>
    private static byte[] Utf8(string s)
    {
        int len = Encoding.UTF8.GetByteCount(s);
        var buf = new byte[len + 1];
        Encoding.UTF8.GetBytes(s, 0, s.Length, buf, 0);
        return buf;
    }
}

/// <summary>A safe handle owning a native backtest pointer.</summary>
internal sealed class ImpactHandle : SafeHandle
{
    public ImpactHandle(IntPtr handle)
        : base(IntPtr.Zero, ownsHandle: true) => SetHandle(handle);

    public override bool IsInvalid => handle == IntPtr.Zero;

    protected override bool ReleaseHandle()
    {
        Native.wickra_impact_free(handle);
        return true;
    }
}
