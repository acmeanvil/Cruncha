+++
cruncha-template = 1
id = "structural.beams.proppedUniform"
version = "1.1.0"
title = "Propped cantilever, uniform load: reactions, moments, slope, deflection and diagrams"
summary = "Reactions, the fixed-end moment, the largest sagging moment, the slope at the pinned end and the largest deflection of a beam pinned at one end and fixed at the other, under a uniform load over its whole span (Roark Table 8.1, case 2c), with shear, moment and deflection diagrams."
category = ["Structural engineering", "Beams"]
tags = ["beam", "propped cantilever", "fixed-pinned", "uniform load", "UDL", "statically indeterminate", "deflection", "Roark", "shear diagram", "moment diagram", "Plotly", "Python"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R_A", "R_B", "M_B", "M_pos", "θ_A", "x_m", "y_max", "M_peak", "M_hog", "y_peak"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2c (distributed load, left end simply supported, right end fixed; uniform over the whole span), p. 192"

[[tests]]
name = "Roark 8.1 case 2c: 10 kN/m on a 6 m span, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test and frame2d_test (Roark 7th Table 8.1 case 2c): RA = 3wl/8 = 22.5 kN, MB = −wl²/8 = −45 kN·m, θA = wl³/(48EI) = 0.00225, M+ = 9wl²/128 = 25.3125 kN·m, ymax: Roark prints 0.0054wl⁴/EI at 0.4215l; the exact 0.0054161 gives 3.5096 mm. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "22.5 kN", R_B = "37.5 kN", M_B = "-45 kN·m", M_pos = "25.3125 kN·m", "θ_A" = "0.00225", x_m = "2.529211 m", y_max = "3.5096468 mm", M_peak = "25.3125 kN·m", M_hog = "-45 kN·m", y_peak = "3.5096468 mm" }
tolerance = "1e-7"
figures = 1
checks = "pass"

[[tests]]
name = "1.2 kip/ft on a 24 ft propped span, I = 510 in⁴"
source = "By hand: RA = 3 × 1.2 × 24/8 = 10.8 kips, RB = 18 kips, MB = −1.2 × 24²/8 = −86.4 kip·ft, M+ = 9 × 1.2 × 576/128 = 48.6 kip·ft; ymax = 0.0054161 × 100 lbf/in × 288⁴/(29e6 × 510) = 0.25194 in"
inputs = { w = "1.2 kip/ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "10.8 kips", R_B = "18 kips", M_B = "-86.4 kip·ft", M_pos = "48.6 kip·ft", y_max = "0.25193597 in", M_peak = "48.6 kip·ft", M_hog = "-86.4 kip·ft", y_peak = "0.25193597 in" }
tolerance = "1e-7"
figures = 1
checks = "pass"
+++

## Propped cantilever, uniform load

A prismatic beam on a pin or roller at its left end A and built in at its right end B, carrying a uniform load w over its whole span (Roark Table 8.1, case 2c). Linear elastic, small deflections, shear deformation neglected. Load the beam's self-weight into w if it matters. Deflections and the slope are magnitudes, deflection downward; moments are positive sagging.

### Beam and load

```calc {description="Uniform load, including self-weight"}
w = 10 kN/m_
```

```calc {description="Span, pinned end to fixed end"}
L = 6 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Second moment of area about the bending axis"}
I = 1e8 mm⁴_
```

### Reactions and moments

```calc
> Reaction at the pinned end
R_A = 3 w L / 8 = ?? kN
> Reaction at the fixed end, and the largest shear
R_B = 5 w L / 8 = ?? kN
> Moment at the fixed end (hogging), the largest moment
M_B = -w L^2 / 8 = ?? kN·m
> Largest sagging moment, at 3L/8 from A
M_pos = 9 w L^2 / 128 = ?? kN·m
```

### Slope and deflection

The deflected shape is y = w x (L³ − 3L x² + 2x³)/(48EI) from A; it peaks at x = L(1 + √33)/16 ≈ 0.4215L, where y ≈ wL⁴/(185EI).

```calc
> Slope at the pinned end
θ_A = w L^3 / (48 E I) = ??
> Position of the largest deflection, from A
x_m = L (1 + sqrt(33)) / 16 = ?? m
> Largest deflection
y_max = w x_m (L^3 - 3 L x_m^2 + 2 x_m^3) / (48 E I) = ?? mm
```

### Diagrams

The shear falls linearly from R_A to −R_B, crossing zero at 3L/8 where the sagging moment peaks; the moment ends at M_B over the fixed end. Deflection is drawn downward.

```python {description="Shear, moment and deflection along the beam, and their diagrams"}
import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from cruncha import Q_, show

_L = L.to("m").magnitude
_w = w.to("N/m").magnitude
_EI = (E * I).to("N*m^2").magnitude
# Roark Table 8.1's expressions along x from A: the values at A, then the load's terms
_R = 3 * _w * _L / 8
_M0 = 0.0
_th = _w * _L**3 / (48 * _EI)
_keys = np.array([3 * _L / 8, x_m.to("m").magnitude])
x_s = np.unique(np.r_[np.linspace(0, _L, 201), _keys])
V_s = Q_(_R - _w * x_s, "N").to("kN")
M_s = Q_(_M0 + _R * x_s - _w * x_s**2 / 2, "N*m").to("kN*m")
y_s = Q_(_th * x_s - (_M0 * x_s**2 / 2 + _R * x_s**3 / 6 - _w * x_s**4 / 24) / _EI, "m").to("mm")

fig = make_subplots(rows=3, cols=1, shared_xaxes=True, subplot_titles=("Shear (kN)", "Moment (kN·m)", "Deflection (mm, downward)"))
fig.add_trace(go.Scatter(x=x_s, y=V_s.magnitude, fill="tozeroy", name="V"), row=1, col=1)
fig.add_trace(go.Scatter(x=x_s, y=M_s.magnitude, fill="tozeroy", name="M"), row=2, col=1)
fig.add_trace(go.Scatter(x=x_s, y=y_s.magnitude, fill="tozeroy", name="y"), row=3, col=1)
fig.update_yaxes(autorange="reversed", row=3, col=1)
fig.update_xaxes(title_text="x from A (m)", row=3, col=1)
fig.update_layout(showlegend=False)
show(fig)
```

The plotted extremes should equal the closed-form values:

```calc {description="Largest sagging moment in the plotted diagram"}
M_peak = max(M_s) = ?? kN·m
```

```calc {description="Largest hogging moment in the plotted diagram"}
M_hog = min(M_s) = ?? kN·m
```

```calc {description="Largest deflection in the plotted diagram"}
y_peak = max(y_s) = ?? mm
```
