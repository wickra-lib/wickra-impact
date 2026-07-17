package org.wickra.impact;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

// The cross-language golden invariant seen from Java: the same request yields
// byte-identical output across calls and across instances. The response bytes are
// what every other binding produces too, because the whole fill engine lives once
// in the Rust core and this binding forwards its JSON verbatim.
class GoldenTest {
    @Test
    void runIsByteIdenticalAcrossInstances() {
        String cmd = ImpactTest.runCmd();
        try (Impact a = new Impact(ImpactTest.SPEC);
                Impact b = new Impact(ImpactTest.SPEC)) {
            assertEquals(a.command(cmd), b.command(cmd));
        }
    }

    @Test
    void reportCarriesImpactStats() {
        try (Impact impact = new Impact(ImpactTest.SPEC)) {
            String out = impact.command(ImpactTest.runCmd());
            assertTrue(out.contains("\"avg_slippage_bps\":44.0"), out);
        }
    }
}
