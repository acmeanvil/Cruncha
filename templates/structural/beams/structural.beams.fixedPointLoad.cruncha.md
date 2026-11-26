+++
cruncha-template = 1
id = "structural.beams.fixedPointLoad"
version = "1.1.0"
title = "Fixed-ended beam, point load: reactions, end moments, deflection and diagrams"
summary = "Reactions, the fixed-end moments, the moment under the load and the deflections of a beam built in at both ends, carrying one concentrated load anywhere on its span (Roark Table 8.1, case 1d), with shear, moment and deflection diagrams."
category = ["Structural engineering", "Beams"]
tags = ["beam", "fixed ends", "fixed-fixed", "built-in", "point load", "concentrated load", "fixed-end moment", "statically indeterminate", "Roark", "shear diagram", "moment diagram", "Plotly", "Python"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["P", "a", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_A", "M_B", "M_P", "y_P", "y_max", "x_m", "M_peak", "M_hog", "y_peak"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 1d (concentrated intermediate load, both ends fixed), p. 190"

[[tests]]
name = "Roark 8.1 case 1d: 10 kN at 2 m on a 6 m span, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 1d): RA = W(l − a)²(l + 2a)/l³ = 7.4074 kN, MA = −Wa(l − a)²/l² = −8.8889 kN·m, MB = −Wa²(l − a)/l² = −4.4444 kN·m, M under the load 2Wa²(l − a)²/l³ = 5.9259 kN·m; deflections by hand and checked against a stiffness solution. The formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { P = "10 kN", a = "2 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "7.4074074 kN", R_B = "2.5925926 kN", M_A = "-8.8888889 kN·m", M_B = "-4.4444444 kN·m", M_P = "5.9259259 kN·m", y_P = "0.39506173 mm", y_max = "0.43537415 mm", x_m = "3.4285714 m", M_peak = "5.9259259 kN·m", M_hog = "-8.8888889 kN·m", y_peak = "0.43537415 mm" }
tolerance = "1e-7"
figures = 1
checks = "pass"

[[tests]]
name = "Load at mid-span: PL/8 at the ends and under the load"
source = "By hand: a = b = L/2 gives MA = MB = −PL/8 = −15 kN·m, M under the load PL/8 = 15 kN·m, y = PL³/(192EI) = 10 000 × 12³/(192 × 2e7) = 4.5 mm"
inputs = { P = "10 kN", a = "6 m", L = "12 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "5 kN", M_A = "-15 kN·m", M_B = "-15 kN·m", M_P = "15 kN·m", y_P = "4.5 mm", y_max = "4.5 mm", x_m = "6 m", M_peak = "15 kN·m", M_hog = "-15 kN·m", y_peak = "4.5 mm" }
tolerance = "1e-9"
figures = 1
checks = "pass"

[[tests]]
name = "12 kips at 8 ft on a 24 ft fixed-ended span, I = 510 in⁴"
source = "By hand: b = 16 ft, MA = −12 × 8 × 16²/24² = −42.667 kip·ft, MB = −12 × 64 × 16/576 = −21.333 kip·ft, RA = 12 × 256 × 40/13 824 = 8.8889 kips"
inputs = { P = "12 kips", a = "8 ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "8.8888889 kips", R_B = "3.1111111 kips", M_A = "-42.666667 kip·ft", M_B = "-21.333333 kip·ft", M_P = "28.444444 kip·ft", y_max = "0.078132086 in", M_peak = "28.444444 kip·ft", M_hog = "-42.666667 kip·ft", y_peak = "0.078132086 in" }
tolerance = "1e-7"
figures = 1
checks = "pass"
+++

## Fixed-ended beam, point load

A prismatic beam built in at both ends A and B, carrying one concentrated load P at a distance a from A (Roark Table 8.1, case 1d). Linear elastic, small deflections, shear deformation neglected; the ends neither rotate nor move. The load is positive downward; deflections are magnitudes, downward. Moments are positive sagging, so the end moments are negative (hogging). The end moments are the fixed-end moments a frame analysis starts from.

### Beam and load

```calc {description="Concentrated load, downward"}
P = 10 kN_
```

```calc {description="Distance of the load from end A"}
a = 2 m_
```

```calc {description="Span between the fixed ends"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Reactions and moments

```calc {description="Distance of the load from end B"}
b = L - a = ?? m
```

```calc
> Reaction at A
R_A = P b^2 (3 a + b) / L^3 = ?? kN
> Reaction at B
R_B = P a^2 (a + 3 b) / L^3 = ?? kN
> Moment at A (hogging)
M_A = -P a b^2 / L^2 = ?? kN·m
> Moment at B (hogging)
M_B = -P a^2 b / L^2 = ?? kN·m
> Moment under the load (sagging)
M_P = 2 P a^2 b^2 / L^3 = ?? kN·m
```

### Deflection

The largest deflection lies between the load and mid-span, on the side of the longer segment. With c₁ the longer and c₂ the shorter of a and b, it occurs at x_m = 2c₁L/(3c₁ + c₂) from the end farther from the load.

```calc
> Deflection under the load
y_P = P a^3 b^3 / (3 E I L^3) = ?? mm
```

```calc
> Longer segment
c_1 = max(a, b) = ?? m
> Shorter segment
c_2 = min(a, b) = ?? m
> Position of the largest deflection, from the end farther from the load
x_m = 2 c_1 L / (3 c_1 + c_2) = ?? m
> Largest deflection
y_max = 2 P c_1^3 c_2^2 / (3 E I (3 c_1 + c_2)^2) = ?? mm
```

### Diagrams

The shear steps down by P under the load; the moment runs in straight lines from M_A through M_P under the load to M_B. Deflection is drawn downward.

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
_R = _P * _b**2 * (3 * _a + _b) / _L**3
_M0 = -_P * _a * _b**2 / _L**2
_th = 0.0
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

The plotted extremes should equal the closed-form values (the hogging one is M_A when the load is nearer A, otherwise M_B):

```calc {description="Largest sagging moment in the plotted diagram"}
M_peak = max(M_s) = ?? kN·m
```

```calc {description="Largest hogging moment in the plotted diagram"}
M_hog = min(M_s) = ?? kN·m
```

```calc {description="Largest deflection in the plotted diagram"}
y_peak = max(y_s) = ?? mm
```

### Checks

```calc
@test 0 'm' < a < L
```
