// SPDX-License-Identifier: GPL-3.0-or-later
//! The built-in agent's instructions (ADR-0009): a system prompt per document, and a skills
//! loader. Cruncha writes each agent's working directory with `AGENTS.md` and the Agent
//! Skills in `.claude/skills/<name>/SKILL.md`. Each skill's front matter (name, description)
//! goes into the prompt; the body is loaded on demand with the `load_skill` tool, so the
//! prompt stays short.

use std::path::Path;

/// A skill: its front matter and where the body is.
#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
}

/// `---\nname: …\ndescription: …\n---\nbody` → (fields, body).
fn front_matter(text: &str) -> Option<(Vec<(String, String)>, String)> {
    let rest = text.strip_prefix("---")?.trim_start_matches(['\r', '\n']);
    let end = rest.find("\n---")?;
    let fields = rest[..end]
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((k.trim().to_owned(), v.trim().trim_matches('"').to_owned()))
        })
        .collect();
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']).to_owned();
    Some((fields, body))
}

/// The skills under `cwd/.claude/skills`, by name.
pub fn load(cwd: &Path) -> Vec<Skill> {
    let Ok(dirs) = std::fs::read_dir(cwd.join(".claude/skills")) else { return vec![] };
    let mut out: Vec<Skill> = dirs
        .flatten()
        .filter_map(|d| {
            let text = std::fs::read_to_string(d.path().join("SKILL.md")).ok()?;
            let (fields, body) = front_matter(&text)?;
            let get = |k: &str| fields.iter().find(|(f, _)| f == k).map(|(_, v)| v.clone());
            Some(Skill { name: get("name")?, description: get("description").unwrap_or_default(), body })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The system prompt for a session working in `cwd`.
pub fn system_prompt(cwd: &Path, skills: &[Skill], tools: &[String]) -> String {
    let mut p = String::from(
        "You are the built-in agent of Cruncha, an engineering calculation app. You work on one \
         Cruncha document through its tools; there are no files or commands for you here.\n\n\
         The engine computes every number: quote results from the tools with their cell IDs, never \
         compute values yourself or present an estimate as a result. To change the document, use \
         propose_edit; the user accepts or rejects it, and nothing changes until they do. Never say \
         a proposal was applied.\n\n\
         Format replies in Markdown. Refer to cells by their IDs.\n",
    );
    if let Ok(agents) = std::fs::read_to_string(cwd.join("AGENTS.md")) {
        p.push_str("\n# Instructions for this document\n\n");
        p.push_str(agents.trim());
        p.push('\n');
    }
    if !tools.is_empty() {
        p.push_str(&format!("\nYour tools: {}.\n", tools.join(", ")));
    }
    if !skills.is_empty() {
        p.push_str(
            "\n# Skills\n\nLoad a skill with load_skill before doing what it covers (load cruncha-core \
             first). Available:\n",
        );
        for s in skills {
            p.push_str(&format!("- {}: {}\n", s.name, s.description));
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skills_and_agents_md_make_the_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let skill = dir.path().join(".claude/skills/cruncha-core");
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), "---\nname: cruncha-core\ndescription: Ground rules.\n---\n\n# Body\nRules.").unwrap();
        std::fs::create_dir_all(dir.path().join(".claude/skills/broken")).unwrap();
        std::fs::write(dir.path().join(".claude/skills/broken/SKILL.md"), "no front matter").unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "Use read_doc first.").unwrap();
        let skills = load(dir.path());
        assert_eq!(skills, vec![Skill { name: "cruncha-core".into(), description: "Ground rules.".into(), body: "# Body\nRules.".into() }]);
        let p = system_prompt(dir.path(), &skills, &["read_doc".into(), "load_skill".into()]);
        assert!(p.contains("Use read_doc first."));
        assert!(p.contains("- cruncha-core: Ground rules."));
        assert!(p.contains("Your tools: read_doc, load_skill."));
        assert!(!p.contains("# Body"), "bodies load on demand");
    }
}
