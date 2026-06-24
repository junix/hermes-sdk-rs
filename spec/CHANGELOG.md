# Spec CHANGELOG

## 2026-06-05T01:47 — CREATED

Initial specification for the Hermes Agent API client SDK, reverse-engineered
from source code, e2e tests, README, and examples.

- Files: 00, 01, 02, 03, 04, 05
- Code basis: 7669957
- Small project (< 10 source files); chapters kept as standard 5-file split plus glossary
- Glossary created: 13 canonical terms
- DoD: 24 items total, 18 [T] (e2e-verified), 6 [U]
- No feature matrix: feature matrix omitted (small SDK, single contract surface, < 15 features)
- Sweeps A-H executed; fixes applied: added client-gateway sequence diagram (Sweep H.3), added self-contained failure-mode tables per Algorithm (Sweep E)

## 2026-06-24T13:10 — DRIFT-UPDATED

Synced spec with commit ecedfed, which added unit tests (builder/response/client `*_test.rs`) locking previously-uncovered behaviors. Public API unchanged.

- Files: 02, 04, 05
- Code basis: ecedfed
- Drift commit (ecedfed) added 15 unit tests; no production behavior change. Findings classified: H.3/H.4 [U]→[T] (non-matching accessor variants now tested); added B.4/B.5/B.6 (messages+all-fields build, conversation-only build, builder-setter overwrite) and H.5/H.6 (empty-content→None, response.text() skips non-Message items); B.1/B.2 observable-result column tightened with ConfigError substrings.
- 02/04: clarified `object_type` is a free-form string the SDK deserializes without enforcing the literal `"response"` (observed gateway value unchanged). Library unit test's `"response.deleted"` mock fixture is incidental (test-harness choice), not a gateway contract — judged per evidence priority (e2e > lib mock).
- feature matrix: omitted (unchanged — small SDK, single contract surface, < 15 features)
