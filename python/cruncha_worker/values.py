# SPDX-License-Identifier: GPL-3.0-or-later
"""Calc values (``Plain``, cruncha-eval's exchange form) ↔ Python objects.

=================  ==========================================================
Cruncha            Python
=================  ==========================================================
exact number       ``fractions.Fraction`` (``int`` when whole)
float              ``float``
quantity           ``pint.Quantity`` from ``cruncha.ureg``, in the calc's unit
vector / matrix    NumPy ``ndarray`` (wrapped in a ``Quantity`` when it has a unit)
data frame         ``polars.DataFrame`` (column units: not carried yet)
bool, text         ``bool``, ``str``
=================  ==========================================================

Units: a ``UnitView`` has Hurmet's name, ``factor`` and ``gauge`` (a value ``v`` is
``(v + gauge) * factor`` in SI base units) and the exponents of the eight base dimensions in
cruncha-units' order. The worker uses pint's unit of the same name when pint knows it and its
dimensions agree; otherwise it uses SI base units. Coming back, pint's pretty unit name
(``kip·ft``, ``m/s²``) is sent with its factor, so the evaluator can fall back to SI when the
name means nothing to Hurmet.
"""

import math
from fractions import Fraction

# cruncha-units' order of base dimensions, as pint dimension names.
DIMS = ["[length]", "[mass]", "[time]", "[current]", "[temperature]", "[substance]", "[luminosity]", "[currency]"]
SI = ["meter", "kilogram", "second", "ampere", "kelvin", "mole", "candela", "EUR"]
MAX_ARRAY = 100_000
MAX_FRAME_ROWS = 10_000

_SUPER = str.maketrans("⁰¹²³⁴⁵⁶⁷⁸⁹⁻", "0123456789-")


def _ureg():
    import cruncha

    return cruncha.ureg


def pint_name(name):
    """A Hurmet unit name in pint's syntax: `kip·ft` → `kip*ft`, `in²` → `in**2`, `°C` → `degC`."""
    out = name.replace("·", "*").replace("°C", "degC").replace("°F", "degF").replace("^", "**").replace("⋅", "*")
    res, i = "", 0
    while i < len(out):
        c = out[i]
        if c in "⁰¹²³⁴⁵⁶⁷⁸⁹⁻":
            j = i
            while j < len(out) and out[j] in "⁰¹²³⁴⁵⁶⁷⁸⁹⁻":
                j += 1
            res += "**" + out[i:j].translate(_SUPER)
            i = j
        else:
            res += c
            i += 1
    return res


def _dims_of(units):
    d = _ureg().get_dimensionality(units)
    return [float(d.get(k, 0)) for k in DIMS]


def _si_expr(dims):
    parts = [f"{u}**{e:g}" for u, e in zip(SI, dims) if e]
    return "*".join(parts) or "dimensionless"


def _unit(view):
    """The pint unit for a UnitView, and whether the values must first go to SI base."""
    ureg = _ureg()
    name = view.get("name")
    if name:
        try:
            u = ureg.parse_units(pint_name(name))
            if all(math.isclose(a, b, abs_tol=1e-12) for a, b in zip(_dims_of(u), view["dims"])):
                return u, False
        except Exception:
            pass
    return ureg.parse_units(_si_expr(view["dims"])), True


def _number(num):
    if num["type"] == "exact":
        f = Fraction(int(num["num"]), int(num["den"]))
        return f.numerator if f.denominator == 1 else f
    return float(num["value"])


def from_plain(p):
    """A Python object for a calc value."""
    t = p["type"]
    if t == "missing":
        return None
    if t in ("bool", "text"):
        return p["value"]
    if t == "complex":
        return complex(p["re"], p["im"])
    if t == "number":
        v = _number(p["value"])
        view = p.get("unit")
        if not view or not any(view["dims"]):
            return v
        u, to_si = _unit(view)
        if to_si:
            v = (float(v) + view.get("gauge", 0)) * view["factor"]
        elif isinstance(v, Fraction):
            v = float(v)
        return _ureg().Quantity(v, u)
    if t == "array":
        import numpy as np

        a = np.asarray(p["values"], dtype=float)
        shape = p["shape"]
        if a.size != math.prod(shape):
            raise ValueError(f"array of {a.size} values doesn't have shape {shape}")
        a = a.reshape(shape)
        view = p.get("unit")
        if not view or not any(view["dims"]):
            return a
        u, to_si = _unit(view)
        if to_si:
            a = (a + view.get("gauge", 0)) * view["factor"]
        return _ureg().Quantity(a, u)
    if t == "frame":
        import polars as pl

        cols = {}
        for c in p["columns"]:
            cols[c["name"]] = c["data"]["values"]
        return pl.DataFrame(cols)
    raise ValueError(f"{p.get('kind', t)} values can't be used in Python yet")


def _unit_view(units):
    """UnitView of a pint unit: from two points, so offset units (°C, °F) get their gauge."""
    ureg = _ureg()
    if ureg.Quantity(1, units).unitless:
        return None
    zero = float(ureg.Quantity(0, units).to_base_units().magnitude)
    one = float(ureg.Quantity(1, units).to_base_units().magnitude)
    factor = one - zero
    dims = _dims_of(units)
    if not any(dims):
        return None
    name = f"{units:~P}".replace("⋅", "·").replace(" ", "")
    return {"name": name, "factor": factor, "gauge": zero / factor if factor else 0.0, "dims": dims}


def _plain_number(v):
    if isinstance(v, bool):
        return {"type": "bool", "value": v}
    if isinstance(v, int):
        return {"type": "number", "value": {"type": "exact", "num": str(v), "den": "1"}, "unit": None}
    if isinstance(v, Fraction):
        return {"type": "number", "value": {"type": "exact", "num": str(v.numerator), "den": str(v.denominator)}, "unit": None}
    if isinstance(v, float):
        if not math.isfinite(v):
            return None
        return {"type": "number", "value": {"type": "float", "value": v}, "unit": None}
    return None


def to_plain(v):
    """The calc form of a Python value, or an ``unsupported`` summary."""
    simple = _plain_number(v)
    if simple is not None:
        return simple
    if isinstance(v, str):
        return {"type": "text", "value": v}
    if isinstance(v, complex):
        return {"type": "complex", "re": v.real, "im": v.imag}
    mod = type(v).__module__.split(".")[0]
    if mod == "numpy":
        import numpy as np

        if isinstance(v, np.generic):
            return to_plain(v.item())
        if isinstance(v, np.ndarray):
            return _plain_array(v, None)
    if mod == "pint":
        mag = v.magnitude
        view = _unit_view(v.units)
        if hasattr(mag, "ndim") and getattr(mag, "ndim", 0) > 0:
            return _plain_array(mag, view)
        num = _plain_number(float(mag) if not isinstance(mag, (int, Fraction)) else mag)
        if num is None or num["type"] != "number":
            return unsupported(v)
        if view and num["value"]["type"] == "exact" and view["gauge"]:
            num["value"] = {"type": "float", "value": float(mag)}
        num["unit"] = view
        return num
    if mod == "polars":
        import polars as pl

        if isinstance(v, pl.DataFrame) and v.height <= MAX_FRAME_ROWS:
            cols = []
            for name, s in zip(v.columns, v.get_columns()):
                if s.dtype.is_numeric():
                    data = {"type": "number", "values": [None if x is None or (isinstance(x, float) and not math.isfinite(x)) else float(x) for x in s.to_list()]}
                else:
                    data = {"type": "text", "values": [None if x is None else str(x) for x in s.to_list()]}
                cols.append({"name": name, "unit": None, "data": data})
            return {"type": "frame", "columns": cols}
    return unsupported(v)


def _plain_array(a, view):
    import numpy as np

    a = np.asarray(a)
    if a.ndim not in (1, 2) or a.size > MAX_ARRAY or not np.issubdtype(a.dtype, np.number) or np.iscomplexobj(a):
        return unsupported(a)
    if not np.all(np.isfinite(a)):
        return unsupported(a)
    return {
        "type": "array",
        "shape": list(a.shape),
        "orientation": "column" if a.ndim == 1 else "matrix",
        "values": [float(x) for x in a.reshape(-1)],
        "unit": view,
    }


def summary(v):
    """A short description of any value, e.g. ``ndarray float64 (50,) m``."""
    t = type(v)
    mod = t.__module__.split(".")[0]
    try:
        if mod == "pint":
            inner = summary(v.magnitude) if hasattr(v.magnitude, "shape") else f"{v.magnitude:.6g}"
            return f"{inner} {v.units:~P}"
        if mod == "numpy" and hasattr(v, "shape"):
            return f"ndarray {v.dtype} {tuple(v.shape)}"
        if mod == "polars" and hasattr(v, "height"):
            return f"{t.__name__} {v.height:,} × {v.width}"
        if callable(v):
            return f"{t.__name__} {getattr(v, '__name__', '')}".strip()
        r = repr(v)
        return r if len(r) <= 80 else r[:79] + "…"
    except Exception:
        return t.__name__


def unsupported(v):
    t = type(v)
    return {"type": "unsupported", "kind": f"{t.__module__}.{t.__qualname__}", "summary": summary(v)}
