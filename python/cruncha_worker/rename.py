# SPDX-License-Identifier: GPL-3.0-or-later
"""Rename names in a cell's source (ADR-0024: cells pasted or copied with renames).

Python's own tokenizer finds the names, so strings and comments are left alone; names inside
f-string replacement fields are renamed (Python 3.12+ tokenizes them). Not renamed:

- attributes (``obj.M``), which belong to the object, not the document;
- keyword arguments in a call (``f(M=1)``), which name the callee's parameters;
- names imported from a module (``from m import M``): ``M`` is the module's name. The import
  gets an alias instead (``from m import M as M_2``), so the cell's own name changes.

The app checks the result by scanning it: its definitions and reads must be the old ones,
renamed, or the rename is refused.
"""

import io
import keyword
import tokenize


def rename(source, names):
    """``source`` with each name in ``names`` (old -> new) renamed."""
    tokens = list(tokenize.generate_tokens(io.StringIO(source).readline))
    edits = []  # (row, col_start, col_end, text)
    depth = 0
    in_def_header = False
    from_import = False
    sig = []  # significant tokens so far (no comments, newlines, indents)
    for i, tok in enumerate(tokens):
        if tok.type == tokenize.OP and tok.string in "([{":
            depth += 1
        elif tok.type == tokenize.OP and tok.string in ")]}":
            depth -= 1
        if tok.type == tokenize.NAME and tok.string in ("def", "lambda"):
            in_def_header = True
        elif tok.type == tokenize.OP and tok.string == ":" and depth == 0:
            in_def_header = False
        if tok.type == tokenize.NAME and tok.string == "from" and not sig:
            from_import = True
        if tok.type in (tokenize.NEWLINE, tokenize.ENDMARKER):
            from_import = False
        prev = sig[-1] if sig else None
        if tok.type in (tokenize.NL, tokenize.NEWLINE, tokenize.COMMENT, tokenize.INDENT, tokenize.DEDENT):
            if tok.type == tokenize.NEWLINE:
                sig = []
            continue
        sig.append(tok)
        if tok.type != tokenize.NAME or tok.string not in names or keyword.iskeyword(tok.string):
            continue
        if prev is not None and prev.type == tokenize.OP and prev.string == ".":
            continue
        nxt = next((t for t in tokens[i + 1:] if t.type not in (tokenize.NL, tokenize.COMMENT)), None)
        is_kwarg = (
            depth > 0
            and not in_def_header
            and nxt is not None
            and nxt.type == tokenize.OP
            and nxt.string == "="
        )
        if is_kwarg:
            continue
        new = names[tok.string]
        if from_import and any(t.type == tokenize.NAME and t.string == "import" for t in sig[:-1]):
            # `from m import M` -> `from m import M as M_2`; `import ... as M` renames the alias.
            if prev is not None and prev.type == tokenize.NAME and prev.string == "as":
                pass
            elif nxt is not None and nxt.type == tokenize.NAME and nxt.string == "as":
                continue
            else:
                new = f"{tok.string} as {new}"
        (row, col), (_, end) = tok.start, tok.end
        edits.append((row, col, end, new))
    lines = source.splitlines(keepends=True)
    for row, col, end, text in sorted(edits, reverse=True):
        line = lines[row - 1]
        lines[row - 1] = line[:col] + text + line[end:]
    return {"source": "".join(lines)}
