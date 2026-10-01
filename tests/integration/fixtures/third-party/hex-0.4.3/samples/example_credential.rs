// A committed fixture specimen proving the test-material exemption. This file is not part
// of hex 0.4.3's own source: it exists only so the calibration can show that a repository
// declaring a fixture location is exempted there. The path sits under `samples/`, which
// ../nomos-test-material.json declares as a fixture location, so the credential-shaped
// literal below is exempted rather than reported by a-credential-is-not-hardcoded-in-source.
// Without that declaration, this file would be judged and the literal reported as a real
// credential -- the false positive the test-material family exists to close.
const EXAMPLE_ACCESS_KEY: &str = "AKIAABCDEFGHIJKLMNOP";
