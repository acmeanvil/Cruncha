+++
cruncha-template = 1
id = "structural.beams.simplePointLoad"
version = "1.1.0"
title = "Simply supported beam, point load: reactions, moment, slopes, deflection and diagrams"
summary = "Reactions, the moment under the load, the end slopes, the deflection under the load and the largest deflection of a simply supported beam carrying one concentrated load anywhere on its span (Roark Table 8.1, case 1e), with shear, moment and deflection diagrams."
category = ["Structural engineering", "Beams"]
tags = ["beam", "simply supported", "point load", "concentrated load", "deflection", "moment", "Roark", "shear diagram", "moment diagram", "Plotly", "Python"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["P", "a", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_max", "θ_A", "θ_B", "y_P", "y_max", "x_m", "M_peak", "y_peak"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 1e (concentrated intermediate load, both ends simply supported), p. 190"

[[tests]]
name = "Roark 8.1 case 1e: 10 kN at 2 m on a 6 m span, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 1e): RA = W(l − a)/l = 6.6667 kN, M = RA·a = 13.333 kN·m, θA = Wa(2l − a)(l − a)/(6EIl) = 0.0011111, ymax = Wa/(3EIl)·((l² − a²)/3)^1.5 = 1.9354 mm. Checked against a stiffness solution (600 elements); the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { P = "10 kN", a = "2 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "6.6666667 kN", R_B = "3.3333333 kN", M_max = "13.333333 kN·m", "θ_A" = "0.0011111111", "θ_B" = "0.00088888889", y_P = "1.7777778 mm", y_max = "1.9353993 mm", x_m = "3.2659863 m", M_peak = "13.333333 kN·m", y_peak = "1.9353993 mm" }
tolerance = "1e-7"
figures = 1
checks = "pass"

[[tests]]
name = "12 kips at 8 ft on a 24 ft span, W-shape with I = 510 in⁴"
source = "By hand: b = 16 ft, RA = 12 × 16/24 = 8 kips, M = 8 × 8 = 64 kip·ft; EI = 29 000 ksi × 510 in⁴; yP = 12 000 × 96² × 192²/(3 EI × 288) = 0.31904 in; ymax = 12 000 × 96 × ((288² − 96²)/3)^1.5/(3 EI × 288) = 0.34733 in at 156.77 in from the far support"
inputs = { P = "12 kips", a = "8 ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "8 kips", R_B = "4 kips", M_max = "64 kip·ft", y_P = "0.31903935 in", y_max = "0.34732605 in", x_m = "156.76734 in", M_peak = "64 kip·ft", y_peak = "0.34732605 in" }
tolerance = "1e-7"
figures = 1
checks = "pass"

[[tests]]
name = "Load past mid-span: the formulas mirror"
source = "By hand: the 6 m beam with the load at 4 m is case 1e turned end for end: RA and RB swap (3.3333, 6.6667 kN), the largest deflection is the same 1.9354 mm"
inputs = { P = "10 kN", a = "4 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "3.3333333 kN", R_B = "6.6666667 kN", M_max = "13.333333 kN·m", "θ_A" = "0.00088888889", "θ_B" = "0.0011111111", y_max = "1.9353993 mm", M_peak = "13.333333 kN·m", y_peak = "1.9353993 mm" }
tolerance = "1e-7"
figures = 1
checks = "pass"
+++

## Simply supported beam, point load

A prismatic beam on a pin and a roller, carrying one concentrated load P at a distance a from the left support A (Roark Table 8.1, case 1e). Linear elastic, small deflections, shear deformation neglected. Loads are positive downward; deflections and slopes are given as magnitudes, deflection downward. For several loads, superpose the results of each (the largest deflections of separate loads don't add, because they occur at different places).

### Beam and load

```calc {description="Concentrated load, downward"}
P = 10 kN_
```

```calc {description="Distance of the load from the left support A"}
a = 2 m_
```

```calc {description="Span between supports"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Reactions and moment

```calc {description="Distance of the load from the right support B"}
b = L - a = ?? m
```

```calc
> Reaction at A
R_A = P b / L = ?? kN
> Reaction at B
R_B = P a / L = ?? kN
> Largest moment, under the load
M_max = P a b / L = ?? kN·m
```

### Slopes and deflection

The largest deflection lies between the load and mid-span, on the side of the longer segment. With c the shorter of a and b, it occurs at x_m = √((L² − c²)/3) from the support farther from the load.

```calc
> End slope at A
θ_A = P a b (L + b) / (6 E I L) = ??
> End slope at B
θ_B = P a b (L + a) / (6 E I L) = ??
> Deflection under the load
y_P = P a^2 b^2 / (3 E I L) = ?? mm
```

```calc {description="Shorter of the two segments"}
c = min(a, b) = ?? m
```

```calc
> Position of the largest deflection, from the support farther from the load
x_m = sqrt((L^2 - c^2) / 3) = ?? m
> Largest deflection
y_max = P c ((L^2 - c^2) / 3)^1.5 / (3 E I L) = ?? mm
```

### Diagrams

The shear steps down by P under the load, the moment is two straight lines meeting at the load, and the deflection is the two cubics of case 1e. Deflection is drawn downward.

```python {description="Shear, moment and deflection along the beam, and their diagrams"}
import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from cruncha import Q_, show

_L = L.to("m").magnitude
_P = P.to("N").magnitude
_a = a.to("m").magnitude
_b = _L - _a
_EI = (E * I).to("N*m^2").magnitude
# Roark Table 8.1's expressions along x from A: the values at A, then the load's terms
_R = _P * _b / _L
_M0 = 0.0
_th = _P * _a * _b * (_L + _b) / (6 * _EI * _L)
# the largest deflection is x_m from the end farther from the load
_keys = np.array([_L - x_m.to("m").magnitude if _a <= _b else x_m.to("m").magnitude])
# x from A, in two pieces that meet under the load; _past is 1 beyond it
_xl = np.unique(np.r_[np.linspace(0, _a, 101), _keys[_keys < _a]])
_xr = np.unique(np.r_[np.linspace(_a, _L, 101), _keys[_keys > _a]])
x_s = np.r_[_xl, _xr]
_past = np.r_[np.zeros(_xl.size), np.ones(_xr.size)]
V_s = Q_(_R - _P * _past, "N").to("kN")
M_s = Q_(_M0 + _R * x_s - _P * _past * (x_s - _a), "N*m").to("kN*m")
y_s = Q_(_th * x_s - (_M0 * x_s**2 / 2 + _R * x_s**3 / 6 - _P * _past * (x_s - _a)**3 / 6) / _EI, "m").to("mm")

fig = make_subplots(rows=3, cols=1, shared_xaxes=True, subplot_titles=("Shear (kN)", "Moment (kN·m)", "Deflection (mm, downward)"))
fig.add_trace(go.Scatter(x=x_s, y=V_s.magnitude, fill="tozeroy", name="V"), row=1, col=1)
fig.add_trace(go.Scatter(x=x_s, y=M_s.magnitude, fill="tozeroy", name="M"), row=2, col=1)
fig.add_trace(go.Scatter(x=x_s, y=y_s.magnitude, fill="tozeroy", name="y"), row=3, col=1)
fig.update_yaxes(autorange="reversed", row=3, col=1)
fig.update_xaxes(title_text="x from A (m)", row=3, col=1)
fig.update_layout(showlegend=False)
show(fig)
```

The plotted peaks should equal the closed-form values:

```calc {description="Largest sagging moment in the plotted diagram"}
M_peak = max(M_s) = ?? kN·m
```

```calc {description="Largest deflection in the plotted diagram"}
y_peak = max(y_s) = ?? mm
```

### Checks

```calc
@test 0 'm' ≤ a ≤ L
```
