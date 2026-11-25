# SPDX-License-Identifier: GPL-3.0-or-later
"""Regenerate the conversion status table in docs/research/02-lacewing-lab-inventory.md
(ADR-0014 item 7): every Lacewing Lab calculator in the inventory, by section, with the
built-in templates whose `from` names it.

    python3 scripts/lacewing-status.py          # rewrite the table
    python3 scripts/lacewing-status.py --check  # fail if the table is out of date
"""

import pathlib
import re
import sys
from collections import defaultdict

ROOT = pathlib.Path(__file__).resolve().parent.parent
INVENTORY = ROOT / "docs/research/02-lacewing-lab-inventory.md"
TEMPLATES = ROOT / "templates"
# Calculators that are lookup tables of copyrighted standards or reference pages, not
# calculations (ADR-0014 §6): not converted until the owner decides how their data is sourced.
NOT_TEMPLATES = {
    "mechanical.hexScrews", "mechanical.heavyHexScrews", "mechanical.socketHeadScrews",
    "mechanical.lowHeadScrews", "mechanical.buttonHeadScrews", "mechanical.flatHeadScrews",
    "mechanical.hexNuts", "mechanical.washers", "mechanical.boltGrades",
    "mechanical.discSpringSizes", "mechanical.pipeSizes", "mechanical.gdtSymbols",
}
BEGIN = "<!-- status:begin (scripts/lacewing-status.py writes this table) -->"
END = "<!-- status:end -->"


def calculators(text):
    """(section, id) in inventory order, from section 2's headings and `(id)` mentions."""
    out, seen, section = [], set(), None
    for line in text.splitlines():
        if line.startswith("## 3"):
            break
        m = re.match(r"### ([A-Z][A-Z &]+?)\s*\(", line)
        if m:
            section = m.group(1).capitalize()
            continue
        for cid in re.findall(r"\(([a-z]+\.[A-Za-z0-9.]+)\)", line):
            if section and cid not in seen:
                seen.add(cid)
                out.append((section, cid))
    return out


def templates():
    """Calculator id -> the built-in template ids converted from it."""
    by = defaultdict(list)
    for p in sorted(TEMPLATES.rglob("*.cruncha.md")):
        s = p.read_text()
        tid = re.search(r'^id = "([^"]+)"', s, re.M)
        src = re.search(r'^from = "lacewing:([^"]+)"', s, re.M)
        if tid and src:
            by[src.group(1)].append(tid.group(1))
    return by


def table(text):
    calcs = calculators(text)
    by = templates()
    known = {c for _, c in calcs}
    lines = [BEGIN, ""]
    unknown = sorted(set(by) - known)
    if unknown:
        sys.exit(f"templates name calculators the inventory doesn't list: {', '.join(unknown)}")
    counts = defaultdict(lambda: [0, 0])
    for section, cid in calcs:
        counts[section][0] += 1
        counts[section][1] += bool(by.get(cid))
    done = sum(c[1] for c in counts.values())
    lines.append(f"{done} of {len(calcs)} calculators have at least one template "
                 f"({sum(len(v) for v in by.values())} templates).")
    lines.append("")
    lines.append("| Section | Calculators | With templates |")
    lines.append("|---|---|---|")
    for section, (n, k) in counts.items():
        lines.append(f"| {section} | {n} | {k} |")
    lines.append("")
    lines.append("| Section | Calculator | Status | Templates |")
    lines.append("|---|---|---|---|")
    for section, cid in calcs:
        ts = by.get(cid, [])
        status = "Templates" if ts else ("Data, not a template: owner's decision" if cid in NOT_TEMPLATES else "Not started")
        lines.append(f"| {section} | `{cid}` | {status} | {', '.join(f'`{t}`' for t in ts)} |")
    lines.append("")
    lines.append(END)
    return "\n".join(lines)


def main():
    text = INVENTORY.read_text()
    new_table = table(text)
    if BEGIN in text:
        head, rest = text.split(BEGIN, 1)
        _, tail = rest.split(END, 1)
        new = head + new_table + tail
    else:
        new = text.rstrip("\n") + "\n\n## 5. Conversion status (ADR-0014 item 7)\n\n" \
            "A calculator has \"Templates\" once at least one built-in template names it in `from`; " \
            "what each template leaves out is in its own prose. Each converted template gets the owner's review.\n\n" \
            + new_table + "\n"
    if "--check" in sys.argv:
        if new != text:
            sys.exit("the status table is out of date: run python3 scripts/lacewing-status.py")
        return
    INVENTORY.write_text(new)


if __name__ == "__main__":
    main()
