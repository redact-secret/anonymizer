# Claude Code

This is `anonymizer`, the Rust forward transformation engine for the Redact Secret
ecosystem. It normalizes and arbitrates recognizer findings, plans replacements,
and constructs irreversible or vault-backed reversible output.

Shared repository instructions are maintained in one place:

@AGENTS.md

Read the authoritative documents named there before changing contracts or code.
The repository has a dependency-free Rust scaffold and CI. Consult README for
implemented behavior; open issues describe requirements, not current guarantees.

Local skills are canonical in `.agents/skills/` and exposed through relative
symlinks in `.claude/skills/`. Edit canonical files, not copies through the links.
Preserve any separate tool-managed `graft` skill. Use only context indexed for
this repository; a graph fallback into a sibling repository is not local coverage.
