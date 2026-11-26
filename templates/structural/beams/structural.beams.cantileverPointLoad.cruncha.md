+++
cruncha-template = 1
id = "structural.beams.cantileverPointLoad"
version = "1.1.0"
title = "Cantilever, point load: reaction, fixed-end moment, slope, deflection and diagrams"
summary = "Reaction and moment at the fixed end, and the slope and deflection at the load and at the free end, of a cantilever carrying one concentrated load anywhere along it (Roark Table 8.1, case 1a), with shear, moment and deflection diagrams."
category = ["Structural engineering", "Beams"]
tags = ["beam", "cantilever", "point load", "concentrated load", "deflection", "moment", "Roark", "shear diagram", "moment diagram", "Plotly", "Python"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["P", "a", "L", "E", "I"]
outputs = ["R_A", "M_A", "θ_tip", "y_P", "y_tip", "M_hog", "y_peak"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 1a (concentrated intermediate load, left end free, right end fixed), p. 189"

[[tests]]
name = "Roark 8.1 case 1a: 10 kN 2 m from the free end of a 6 m cantilever, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 1a, a measured from the free end; here a = 4 m from the fixed end): MB = −W(l − aR) = −40 kN·m, y free end = W/(6EI)·(2l³ − 3l²aR + aR³) = 18.667 mm, slope W(l − aR)²/(2EI) = 0.004. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { P = "10 kN", a = "4 m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "10 kN", M_A = "-40 kN·m", "θ_tip" = "0.004", y_P = "10.666667 mm", y_tip = "18.666667 mm", M_hog = "-40 kN·m", y_peak = "18.666667 mm" }
tolerance = "1e-7"
figures = 1
checks = "pass"

[[tests]]
name = "2 kips at the tip of a 10 ft cantilever, I = 510 in⁴"
source = "By hand: M = −2 × 10 = −20 kip·ft; y = PL³/(3EI) = 2000 × 120³/(3 × 29e6 × 510) = 0.077890 in; θ = PL²/(2EI) = 0.00097363"
inputs = { P = "2 kips", a = "10 ft", L = "10 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "2 kips", M_A = "-20 kip·ft", "θ_tip" = "0.00097363083", y_P = "0.077890467 in", y_tip = "0.077890467 in", M_hog = "-20 kip·ft", y_peak = "0.077890467 in" }
tolerance = "1e-7"
figures = 1
checks = "pass"
+++

## Cantilever, point load

A prismatic beam built in at its left end A and free at the other, carrying one concentrated load P at a distance a from the fixed end (Roark Table 8.1, case 1a; Roark draws the fixed end on the right and measures a from the free end, so his a is L − a here). Linear elastic, small deflections, shear deformation neglected. The load is positive downward; deflections and slopes are magnitudes, deflection downward. The moment is negative (hogging: tension on top).

Beyond the load the beam carries no moment, so it stays straight: the tip deflects by the deflection at the load plus the slope there times the remaining length.

### Beam and load

```calc {description="Concentrated load, downward"}
P = 10 kN_
```

```calc {description="Distance of the load from the fixed end A"}
a = 4 m_
```

```calc {description="Length of the cantilever"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Results

```calc
> Reaction at the fixed end
R_A = P = ?? kN
> Moment at the fixed end (hogging)
M_A = -P a = ?? kN·m
> Slope at the load and beyond it, to the free end
θ_tip = P a^2 / (2 E I) = ??
> Deflection under the load
y_P = P a^3 / (3 E I) = ?? mm
> Deflection at the free end
y_tip = P a^2 (3 L - a) / (6 E I) = ?? mm
```

### Diagrams

The shear is P from the fixed end to the load and zero beyond it; the moment rises linearly from M_A to zero at the load; beyond the load the beam stays straight. Deflection is drawn downward.

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
_R = _P
_M0 = -_P * _a
_th = 0.0
_keys = np.array([])
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

The plotted extremes should equal the closed-form values at the fixed end and the tip:

```calc {description="Largest hogging moment in the plotted diagram"}
M_hog = min(M_s) = ?? kN·m
```

```calc {description="Largest deflection in the plotted diagram"}
y_peak = max(y_s) = ?? mm
```

### Checks

```calc
@test 0 'm' < a ≤ L
```
