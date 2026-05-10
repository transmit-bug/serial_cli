# Documentation Structure

This directory contains all project documentation organized by purpose and audience.

## Directory Hierarchy

```
docs/
├── user/           # End-user documentation
│   ├── getting-started.md
│   └── interactive-shell.md
├── dev/            # Internal development docs
│   ├── ARCH.md                    # Architecture overview
│   ├── SERVER_MODE.md             # Server Mode design doc (Chinese)
│   └── SERVER_MODE_DECISION.md    # Design decision record
├── reference/      # Reference material
│   ├── configuration.md
│   ├── lua-scripting.md
│   ├── protocols.md
│   └── troubleshooting.md
├── commands/       # Per-command documentation
│   ├── batch.md
│   ├── benchmark.md
│   ├── config.md
│   └── ...
└── ai/             # AI/automation workflow specific docs
    ├── USAGE.md
    └── SERVER_MODE.md
```

## Documentation Guidelines

### When to Create Documentation

**Before creating any new .md file:**
1. Search existing docs to avoid duplication
2. Verify the file belongs in the correct directory
3. Consider if an existing doc can be updated instead

### Directory Purposes

- **user/** - Tutorials, guides, and feature explanations for end users
- **dev/** - Architecture, design decisions, technical specifications (internal)
- **reference/** - Configuration reference, API docs, troubleshooting guides
- **commands/** - Command-specific documentation (one file per command)
- **ai/** - Specialized documentation for AI/automation workflows

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

- ❌ Creating root-level `docs/*.md` files (except this README.md)
- ❌ Duplicating content across multiple files
- ❌ Mixing user-facing and internal content in the same file
- ❌ Creating docs for transient issues (use issue tracker instead)
