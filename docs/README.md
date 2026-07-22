# Documentation Structure

This directory contains all project documentation organized by purpose and audience.

## Directory Hierarchy

```
docs/
├── guides/         # End-user documentation
│   ├── getting-started.md
│   └── interactive-shell.md
├── dev/            # Internal development docs
│   ├── ARCH.md                      # Architecture overview
│   ├── FRONTEND-REWRITE-DESIGN.md   # Frontend rewrite design spec
│   └── UNIFIED-SCRIPT-SYSTEM.md     # Unified script system design decision
├── reference/      # Reference material
│   ├── configuration.md
│   ├── events.md
│   ├── lua-scripting.md
│   ├── protocols.md                 # Script/protocol reference
│   ├── terminology.md               # English-Chinese glossary
│   └── troubleshooting.md
├── commands/       # Per-command documentation
│   ├── batch.md
│   ├── config.md
│   └── ...
├── features/       # Feature-specific documentation
│   └── ui-actions.md
├── ui/             # GUI design documentation
│   ├── README.md
│   ├── UI-Design-Spec.md
│   └── User-Flow-Diagrams.md
├── ai/             # AI/automation workflow docs
│   ├── USAGE.md
│   └── SERVER_MODE.md   # Server mode API reference (user-facing)
└── agents/         # Agent skill configuration
    ├── issue-tracker.md     # Issue tracker settings (GitHub/GitLab/local)
    ├── triage-labels.md     # Triage label vocabulary mapping
    └── domain.md            # Domain doc consumer rules + layout
```

## Documentation Guidelines

### When to Create Documentation

**Before creating any new .md file:**
1. Search existing docs to avoid duplication
2. Verify the file belongs in the correct directory
3. Consider if an existing doc can be updated instead

### Directory Purposes

- **guides/** - Tutorials, guides, and feature explanations for end users
- **dev/** - Architecture, design decisions, technical specifications (internal)
- **reference/** - Configuration reference, API docs, troubleshooting guides
- **commands/** - Command-specific documentation (one file per command)
- **features/** - Feature-specific documentation
- **ui/** - GUI design specifications and user flow diagrams
- **ai/** - Specialized documentation for AI/automation workflows
- **agents/** - Agent skill configuration (issue tracker, triage labels, domain docs)

### File Naming Conventions

- Use kebab-case for filenames: `getting-started.md`
- Design decisions: `FEATURE_DECISION.md`
- Architecture: `ARCH.md` (in dev/)
- Command docs: command name matching CLI: `config.md`, `sniff.md`

### Content Guidelines

- **Keep docs minimal** - Only create docs that serve a clear, ongoing purpose
- **Avoid redundancy** - One source of truth for each topic
- **Audience-appropriate** - User docs are non-technical, dev docs assume Rust knowledge
- **Maintain synchrony** - Update docs when code changes

## Prohibited Actions

- Creating root-level `docs/*.md` files (except this README.md)
- Duplicating content across multiple files
- Mixing user-facing and internal content in the same file
- Creating docs for transient issues (use issue tracker instead)
