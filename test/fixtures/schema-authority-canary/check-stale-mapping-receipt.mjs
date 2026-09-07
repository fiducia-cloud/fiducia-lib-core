import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const [reportArgument] = process.argv.slice(2);
assert.ok(reportArgument, "usage: check-stale-mapping-receipt.mjs <report.json>");

const reportPath = resolve(reportArgument);
const report = JSON.parse(await readFile(reportPath, "utf8"));

assert.equal(
  report.schema,
  "ores.typespec-json-schema-validator.report/v1",
  "negative lane must retain a validator-owned deterministic receipt",
);
assert.equal(
  report.status,
  "stopped_for_evaluation",
  "a stale mapping is an attributable parity finding, not an execution failure",
);
assert.equal(report.authorities?.precedence, "none");
assert.equal(
  report.authorities?.onUnexplainedMismatch,
  "STOPPED_FOR_EVALUATION",
);
assert.ok(Array.isArray(report.findings));

const stale = report.findings.filter(
  (finding) => finding.ruleId === "mapping-typespec-declaration-missing",
);
assert.equal(
  stale.length,
  1,
  "the negative fixture must produce exactly one stale TypeSpec mapping finding",
);
assert.equal(stale[0].comparison, "mapping-integrity");
assert.equal(
  stale[0].declaration,
  "OreSchemaAuthority.MissingRegistration",
);
assert.equal(stale[0].resolutionState, "unexplained");
assert.match(stale[0].fingerprint, /^[0-9a-f]{64}$/u);
assert.equal(
  report.findings.some((finding) => finding.ruleId === "run-failed"),
  false,
  "the negative lane must not disguise a mapping finding as a tool failure",
);

process.stdout.write(
  JSON.stringify(
    {
      findingCount: report.findingCount,
      ruleId: stale[0].ruleId,
      runId: report.runId,
      status: report.status,
    },
    null,
    2,
  ) + "\n",
);
