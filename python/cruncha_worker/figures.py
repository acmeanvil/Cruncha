# SPDX-License-Identifier: GPL-3.0-or-later
"""Interactive figures (ADR-0017): JSON specs keyed by MIME type, never HTML.

A cell shows a figure by leaving it as its last expression, calling ``fig.show()``, or
``cruncha.show(fig)``. Each becomes a ``figure`` output: the spec as a binary frame, its MIME
type, a text summary (for agents and the mirror) and the data range of each trace. The app
draws it in a sandboxed frame; reports use its SVG.

Plotly is adapted when it is imported (so cells that don't use it don't pay for it):

- ``fig.show()`` and ``plotly.io.show`` are captured instead of opening a browser;
- the ``cruncha`` template is the default, so figures match the app and the report;
- pint quantities are unwrapped to magnitudes, and an axis without a title gets the unit;
- trace ``uid``s come from the cell, so the same input draws the same SVG.

Other libraries are offered through ``_repr_mimebundle_``; only allowlisted MIME types pass.
"""

import importlib.abc
import importlib.machinery
import json
import sys

PLOTLY = "application/vnd.plotly.v1+json"
VEGALITE = "application/vnd.vegalite.v5+json"
ALLOWED = (PLOTLY, VEGALITE)
MAX_SPEC = 8 << 20
UNITS_KEY = "_cruncha_units"
FONT = "Inter, Helvetica Neue, Helvetica, Arial, sans-serif"

_pending = []  # (mime, spec, static bytes or None)
_seen = set()  # ids of figure objects already captured in this run
_cell = "cell"


def begin(cell):
    """A run starts: nothing captured yet."""
    global _cell
    _cell = cell
    _pending.clear()
    _seen.clear()


def unit_text(u):
    return f"{u:~P}".replace("⋅", "·")


def _quantity(v):
    return getattr(v, "units", None) is not None and hasattr(v, "magnitude")


# --- capture -------------------------------------------------------------------------------


def offer(obj, static=None, explicit=False):
    """Capture ``obj`` if it is a figure. Returns whether it was (or already had been)."""
    if id(obj) in _seen:
        return True
    mod = type(obj).__module__.split(".")[0]
    if mod == "plotly" and hasattr(obj, "to_dict") and hasattr(obj, "data"):
        _seen.add(id(obj))
        _pending.append((PLOTLY, obj.to_dict(), static))
        return True
    if mod == "altair" and hasattr(obj, "to_dict"):
        _seen.add(id(obj))
        _pending.append((VEGALITE, obj.to_dict(), static))
        return True
    if mod == "matplotlib":
        return True  # open figures are collected as PNGs anyway
    bundle = None
    if hasattr(obj, "_repr_mimebundle_") and not isinstance(obj, type):
        try:
            bundle = obj._repr_mimebundle_(include=None, exclude=None)
        except Exception:
            bundle = None
        if isinstance(bundle, tuple):
            bundle = bundle[0]
    if isinstance(bundle, dict):
        for mime in ALLOWED:
            spec = bundle.get(mime)
            if isinstance(spec, str):
                spec = json.loads(spec)
            if isinstance(spec, dict):
                _seen.add(id(obj))
                _pending.append((mime, spec, static))
                return True
        if explicit:
            kinds = ", ".join(sorted(bundle)) or "nothing"
            raise TypeError(f"Cruncha shows only {', '.join(ALLOWED)} figures; this object offers {kinds}")
    if explicit:
        raise TypeError(f"Cruncha can't show a {type(obj).__name__} as a figure")
    return False


def _show_dict(fig_dict):
    _pending.append((PLOTLY, fig_dict, None))


# --- Plotly adaptation (installed when plotly is imported) ---------------------------------


def _adapt_traces(basedatatypes):
    """Unwrap pint quantities set on a trace, keeping their unit in the trace's meta."""
    import numpy as np

    orig = basedatatypes.BasePlotlyType._set_prop
    if getattr(orig, "_cruncha", False):
        return

    def _set_prop(self, prop, val):
        if _quantity(val) and isinstance(self, basedatatypes.BaseTraceType):
            meta = self.meta
            m = dict(meta) if isinstance(meta, dict) else ({} if meta is None else {"user": meta})
            m.setdefault(UNITS_KEY, {})[prop] = unit_text(val.units)
            orig(self, "meta", m)
            val = np.asarray(val.magnitude)
        return orig(self, prop, val)

    _set_prop._cruncha = True
    basedatatypes.BasePlotlyType._set_prop = _set_prop

    def show(self, *args, **kwargs):
        offer(self)

    basedatatypes.BaseFigure.show = show


def _adapt_express(core):
    """Plotly Express: quantities passed as x/y/z become magnitudes, labelled with the unit."""
    import numpy as np

    orig = core.build_dataframe
    if getattr(orig, "_cruncha", False):
        return

    def build_dataframe(args, constructor):
        labels = dict(args.get("labels") or {})
        for k in ("x", "y", "z"):
            v = args.get(k)
            if _quantity(v):
                labels.setdefault(k, f"{k} ({unit_text(v.units)})")
                args[k] = np.asarray(v.magnitude)
        if labels:
            args["labels"] = labels
        return orig(args, constructor)

    build_dataframe._cruncha = True
    core.build_dataframe = build_dataframe


def _adapt_io(pio):
    """The default renderer captures; the cruncha template is the default."""
    from plotly.io._base_renderers import ExternalRenderer

    class Capture(ExternalRenderer):
        def render(self, fig_dict):
            _show_dict(fig_dict)

    pio.renderers["cruncha"] = Capture()
    pio.renderers.default = "cruncha"
    import plotly.graph_objects as go

    pio.templates["cruncha"] = go.layout.Template(
        layout={
            "font": {"family": FONT, "size": 12, "color": "#1f2328"},
            "title": {"font": {"size": 15}, "x": 0.02, "xanchor": "left"},
            "margin": {"l": 60, "r": 20, "t": 50, "b": 50},
        }
    )
    pio.templates.default = "plotly_white+cruncha"


_HOOKS = {
    "plotly.basedatatypes": _adapt_traces,
    "plotly.express._core": _adapt_express,
    "plotly.io": _adapt_io,
}


class _Finder(importlib.abc.MetaPathFinder):
    """Runs an adapter right after one of the hooked modules finishes importing."""

    def find_spec(self, name, path, target=None):
        hook = _HOOKS.get(name)
        if hook is None:
            return None
        spec = importlib.machinery.PathFinder.find_spec(name, path)
        if spec is None or spec.loader is None:
            return spec
        loader = spec.loader
        exec_module = loader.exec_module

        def exec_and_adapt(module):
            exec_module(module)
            try:
                hook(module)
            except Exception as e:  # a plotly change must not break the cell
                print(f"cruncha: couldn't adapt {name}: {e}", file=sys.stderr)

        loader.exec_module = exec_and_adapt
        return spec


def install():
    """Adapt Plotly whenever it gets imported (and now, if it already was)."""
    if not any(isinstance(f, _Finder) for f in sys.meta_path):
        sys.meta_path.insert(0, _Finder())
    for name, hook in _HOOKS.items():
        if name in sys.modules:
            hook(sys.modules[name])


# --- outputs -------------------------------------------------------------------------------


def _numbers(v):
    """A trace field as a flat float array, or None when it isn't numeric."""
    try:
        import numpy as np

        if isinstance(v, dict) and "bdata" in v:  # Plotly's typed-array encoding (little-endian)
            import base64

            v = np.frombuffer(base64.b64decode(v["bdata"]), dtype=np.dtype("<" + v["dtype"]))
        a = np.asarray(v, dtype=float).ravel()
        a = a[np.isfinite(a)]
        return a if a.size else None
    except Exception:
        return None


def _text(t):
    if isinstance(t, dict):
        t = t.get("text")
    return t if isinstance(t, str) else ""


def _prepare_plotly(fig, index):
    """Units into axis titles, stable uids, and the summary and ranges (from the raw arrays)."""
    layout = fig.setdefault("layout", {})
    traces = fig.get("data") or []
    ranges = []
    parts = []
    for i, t in enumerate(traces):
        meta = t.get("meta")
        units = {}
        if isinstance(meta, dict) and UNITS_KEY in meta:
            units = meta.pop(UNITS_KEY) or {}
            if not meta:
                t.pop("meta")
            elif list(meta) == ["user"]:
                t["meta"] = meta["user"]
        for axis in ("x", "y"):
            unit = units.get(axis)
            if not unit:
                continue
            ref = t.get(f"{axis}axis", axis)
            key = f"{axis}axis" + ref[1:]
            ax = layout.setdefault(key, {})
            if not _text(ax.get("title")):
                ax["title"] = {"text": unit if not t.get("name") or axis == "x" else f"{t['name']} ({unit})"}
        t["uid"] = f"{_cell}-{index}-{i}"
        r = {"trace": t.get("name") or f"trace {i}", "type": t.get("type", "scatter")}
        n = 0
        for axis in ("x", "y", "z"):
            a = _numbers(t[axis]) if axis in t else None
            if a is not None:
                r[axis] = [float(a.min()), float(a.max())]
                if units.get(axis):
                    r[f"{axis}_unit"] = units[axis]
                n = max(n, a.size)
        r["points"] = n
        ranges.append(r)
        parts.append(f"{r['trace']} ({r['type']}, {n} points)" if n else f"{r['trace']} ({r['type']})")
    title = _text(layout.get("title"))
    xt = _text(layout.get("xaxis", {}).get("title"))
    yt = _text(layout.get("yaxis", {}).get("title"))
    head = title or "Figure"
    if xt or yt:
        head += f": {yt or 'y'} vs {xt or 'x'}"
    summary = f"{head}; {len(traces)} trace{'s' if len(traces) != 1 else ''}: {', '.join(parts)}" if traces else head
    return summary, ranges


def _encode_plotly(fig):
    import plotly.io as pio

    return pio.to_json(fig, validate=False, engine="json", remove_uids=False).encode()


def collect(blobs):
    """The figures captured during the run, as outputs (specs appended to ``blobs``)."""
    out = []
    for index, (mime, spec, static) in enumerate(_pending):
        try:
            if mime == PLOTLY:
                summary, ranges = _prepare_plotly(spec, index)
                data = _encode_plotly(spec)
            else:
                title = spec.get("title")
                summary = f"{_text(title) or 'Chart'} ({spec.get('mark', {}).get('type') if isinstance(spec.get('mark'), dict) else spec.get('mark', 'chart')})"
                ranges = []
                data = json.dumps(spec, separators=(",", ":"), default=str).encode()
        except Exception as e:
            out.append({"kind": "text", "text": f"(couldn't show a figure: {type(e).__name__}: {e})"})
            continue
        if len(data) > MAX_SPEC:
            mb = len(data) / (1 << 20)
            out.append({"kind": "text", "text": f"(figure not shown: its spec is {mb:.1f} MB, over the 8 MB limit; "
                                                "downsample it, or use scattergl with fewer points)"})
            continue
        blobs.append(data)
        o = {"kind": "figure", "mime": mime, "blob": len(blobs) - 1, "summary": summary, "ranges": ranges}
        if static is not None:
            blobs.append(bytes(static))
            o["static"] = len(blobs) - 1
            o["static_mime"] = "image/svg+xml" if bytes(static[:5]).lstrip().startswith(b"<") else "image/png"
        out.append(o)
    _pending.clear()
    _seen.clear()
    return out
