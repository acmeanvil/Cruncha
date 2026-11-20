# SPDX-License-Identifier: GPL-3.0-or-later
"""Which names a Python cell defines and reads (design 02 §3.3).

* **Defines:** names bound at the top level of the cell (assignments, ``for`` and ``with``
  targets, ``def``, ``class``). Imported names are local to the cell, so every cell can
  ``import numpy as np`` without defining ``np`` twice. Names starting with ``_`` are private.
* **Reads:** free names the cell uses (also inside its functions and classes) that it doesn't
  bind itself and that aren't builtins.
* ``# cruncha: defines a, b`` replaces the defines; ``# cruncha: reads c`` adds reads. Dynamic
  code (``exec``, ``eval``, ``globals()``, ``import *``) gets a warning: the scan can't see it.
"""

import ast
import builtins
import re
import symtable

BUILTINS = frozenset(dir(builtins)) | {"__name__", "__file__", "__builtins__"}
DYNAMIC_CALLS = {"exec", "eval", "globals", "locals", "vars", "__import__"}
_DECL = re.compile(r"^\s*#\s*cruncha:\s*(defines|reads)\b\s*(.*)$", re.MULTILINE)


def _names(text):
    return [n for n in re.split(r"[\s,]+", text.strip()) if n.isidentifier()]


_SCOPES = (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Lambda)
_COMPREHENSIONS = (ast.ListComp, ast.SetComp, ast.DictComp, ast.GeneratorExp)


def _top_level_binds(tree):
    """Names bound by the cell's own top level, in source order, with where. Comprehension
    variables are skipped: since Python 3.12 the symbol table shows them at module level
    (PEP 709), but they don't outlive the comprehension. So are bindings in nested scopes."""
    pos = {}

    def visit(node):
        if isinstance(node, _COMPREHENSIONS):
            walrus_targets(node)
            return
        if isinstance(node, _SCOPES):
            if not isinstance(node, ast.Lambda):
                pos.setdefault(node.name, (node.lineno, node.col_offset))
            for d in getattr(node, "decorator_list", []):
                visit(d)
            return
        if isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store):
            pos.setdefault(node.id, (node.lineno, node.col_offset))
        elif isinstance(node, ast.NamedExpr):
            pos.setdefault(node.target.id, (node.lineno, node.col_offset))
        for child in ast.iter_child_nodes(node):
            visit(child)

    def walrus_targets(node):
        # `:=` inside a comprehension binds in the enclosing scope (PEP 572).
        for n in ast.walk(node):
            if isinstance(n, ast.NamedExpr):
                pos.setdefault(n.target.id, (n.lineno, n.col_offset))

    for stmt in tree.body:
        visit(stmt)
        # `del x` at the top level: x doesn't outlive the cell (unless bound again later).
        if isinstance(stmt, ast.Delete):
            for t in stmt.targets:
                for n in ast.walk(t):
                    if isinstance(n, ast.Name):
                        pos.pop(n.id, None)
    return pos


def scan(source):
    """``{"defines": [...], "reads": [...], "warnings": [...]}``, or ``{"error": {...}}`` for a
    syntax error."""
    try:
        tree = ast.parse(source, "<cell>", "exec")
        table = symtable.symtable(source, "<cell>", "exec")
    except SyntaxError as e:
        return {"error": {"type": "SyntaxError", "message": e.msg, "line": e.lineno, "column": e.offset}}

    defined, imported, reads = set(), set(), set()
    for sym in table.get_symbols():
        name = sym.get_name()
        if sym.is_imported():
            imported.add(name)
        elif sym.is_assigned() or sym.is_namespace():
            defined.add(name)
        elif sym.is_referenced():
            reads.add(name)

    def nested(t):
        for child in t.get_children():
            for sym in child.get_symbols():
                if (
                    sym.is_referenced()
                    and (sym.is_global() or sym.is_declared_global())
                    and not sym.is_assigned()
                    and not sym.is_parameter()  # seen marked global too (stdlib poplib)
                ):
                    reads.add(sym.get_name())
            nested(child)

    nested(table)

    warnings = []
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and any(a.name == "*" for a in node.names):
            warnings.append(f"`from {node.module} import *` hides which names this cell defines")
        elif isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in DYNAMIC_CALLS:
            warnings.append(f"`{node.func.id}()` can define or read names the dependency scan can't see")
        elif isinstance(node, ast.Global):
            warnings.append("`global` statements don't share names between cells; define them at the top level")

    pos = _top_level_binds(tree)
    defined = set(pos) - imported  # the syntax tree decides what's bound at the top level
    defines = sorted((n for n in defined if not n.startswith("_")), key=lambda n: pos[n])
    reads = sorted(n for n in reads - defined - imported - BUILTINS if not n.startswith("_"))

    for kind, text in _DECL.findall(source):
        names = _names(text)
        if kind == "defines":
            defines = names
            warnings = [w for w in warnings if "dependency scan" not in w and "import *" not in w]
        else:
            reads = sorted(set(reads) | set(names))
    # Dedupe warnings, keeping order.
    return {"defines": defines, "reads": reads, "warnings": list(dict.fromkeys(warnings))}
