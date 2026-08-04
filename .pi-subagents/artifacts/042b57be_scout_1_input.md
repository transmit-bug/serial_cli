# Task for scout

Investigate the Tauri backend subproject at /Users/pony/codehub/rust/serial_cli/src-tauri/ for writing an AGENTS.md. Focus on:

1. **Architecture & patterns**: How Tauri commands are organized, state management (how Rust state is shared), event system usage, how it bridges to the main serial-cli library
2. **Key conventions**: Command naming, error handling patterns in Tauri context, how frontend events map to Rust handlers
3. **Non-obvious rules**: What NOT to do, thread safety considerations, how the Tauri app relates to the CLI binary
4. **Build & dev**: Cargo workspace setup, Tauri-specific config, build differences from the CLI
5. **Project structure**: Key directories and their purpose at a high level

Output a structured summary. Skip things obvious from Cargo.toml or inferable from code. Focus on architectural decisions and non-obvious patterns that would confuse a new agent.

---
Update progress at: /Users/pony/codehub/rust/serial_cli/.pi-subagents/artifacts/progress/042b57be/progress.md

---
**Output:**
Write your findings to exactly this path: /Users/pony/codehub/rust/serial_cli/.pi-subagents/artifacts/outputs/042b57be/backend-investigation.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: attested
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Return concrete findings with file paths and severity when applicable

Required evidence: review-findings, residual-risks

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
`criteriaSatisfied[].status` must be exactly one of: satisfied, not-satisfied, not-applicable.
`commandsRun[].result` must be exactly one of: passed, failed, not-run.
`manualNotes` and `notes` are optional strings; an empty string means no note and does not satisfy `manual-notes` evidence.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```