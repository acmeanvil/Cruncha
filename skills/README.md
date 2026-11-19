# Cruncha agent skills

Skills that teach an AI agent how to work **inside a Cruncha document**: authoring calcs, writing Python and Rust cells, reviewing a calc package, and using history. They follow the open Agent Skills format: one folder per skill, each with a `SKILL.md` whose YAML frontmatter holds `name` and `description`, plus optional `references/` files loaded on demand. Any skills-aware agent can use them:
- Cruncha's built-in agent;
- Claude Code (symlink or copy this folder into `.claude/skills/`);
- other ACP agents.

| Skill | Use when |
| --- | --- |
| [cruncha-core](cruncha-core/SKILL.md) | **Always.** Ground rules: the engine computes and the agent proposes. Tools and provenance. |
| [cruncha-calc-authoring](cruncha-calc-authoring/SKILL.md) | Writing or editing calc cells: statement form, display selectors, units, data frames, tests |
| [cruncha-code-cells](cruncha-code-cells/SKILL.md) | Writing Python (NumPy/SciPy/Polars) or Rust cells that join the document graph |
| [cruncha-calc-review](cruncha-calc-review/SKILL.md) | Checking a calc package as a reviewer: units, assumptions, independent re-derivation |
| [cruncha-history](cruncha-history/SKILL.md) | Questions about what changed, versions, tags, rollback |

**Status:** the engine isn't built yet. These skills describe the syntax and tools **as designed** in `docs/design/02-document-model.md` and `docs/adr/`. When an implementation diverges, update the skill in the same change. The skills are the agent-facing spec, so they must never drift from the implementation.
