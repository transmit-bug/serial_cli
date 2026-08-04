# Task for scout

Investigate the frontend subproject at /Users/pony/codehub/rust/serial_cli/frontend/ for writing an AGENTS.md. Focus on:

1. **Architecture & patterns**: State management approach (Zustand stores), component structure, how Tauri invoke/event bridges work, routing if any
2. **Key conventions**: Component naming, store patterns, how frontend communicates with Tauri backend (invoke vs events), styling approach (Tailwind + shadcn/ui patterns)
3. **Non-obvious rules**: What NOT to do, cross-cutting concerns, how data flows between stores
4. **Build & dev**: pnpm scripts, biome config, vitest setup, any special env vars
5. **Project structure**: Key directories and their purpose, but only high-level ones

Output a structured summary. Skip things that are obvious from package.json or that an agent could infer by reading code. Focus on architectural decisions and non-obvious patterns that would confuse a new agent.

---
Update progress at: /Users/pony/codehub/rust/serial_cli/.pi-subagents/artifacts/progress/042b57be/progress.md

---
**Output:**
Write your findings to exactly this path: /Users/pony/codehub/rust/serial_cli/.pi-subagents/artifacts/outputs/042b57be/frontend-investigation.md
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