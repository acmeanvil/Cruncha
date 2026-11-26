+++
cruncha-template = 1
id = "structural.beams.cantileverUniform"
version = "1.1.0"
title = "Cantilever, uniform load: reaction, fixed-end moment, slope, deflection and diagrams"
summary = "Reaction and moment at the fixed end, and the slope and deflection at the free end, of a cantilever under a uniform load over its whole length (Roark Table 8.1, case 2a), with shear, moment and deflection diagrams."
category = ["Structural engineering", "Beams"]
tags = ["beam", "cantilever", "uniform load", "UDL", "deflection", "moment", "Roark", "shear diagram", "moment diagram", "Plotly", "Python"]
from = "lacewing:structural.beam"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R_A", "M_A", "θ_tip", "y_tip", "M_hog", "y_peak"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Table 8.1, case 2a (distributed load, left end free, right end fixed; uniform over the whole length), p. 191"

[[tests]]
name = "Roark 8.1 case 2a: 10 kN/m on a 6 m cantilever, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test (Roark 7th Table 8.1 case 2a): M = −wl²/2 = −180 kN·m, y = wl⁴/(8EI) = 81 mm; θ = wl³/(6EI) = 0.018 by hand. Checked against a stiffness solution; the formulas were checked in the Roark 7th PDF, Table 8.1"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R_A = "60 kN", M_A = "-180 kN·m", "θ_tip" = "0.018", y_tip = "81 mm", M_hog = "-180 kN·m", y_peak = "81 mm" }
tolerance = "1e-9"
figures = 1
checks = "pass"

[[tests]]
name = "500 lbf/ft on a 12 ft cantilever, I = 510 in⁴"
source = "By hand: R = 6 kips, M = −0.5 × 12²/2 = −36 kip·ft; y = (500/12) × 144⁴/(8 × 29e6 × 510) = 0.15142 in; θ = (500/12) × 144³/(6 × 29e6 × 510) = 0.0014020"
inputs = { w = "500 lbf/ft", L = "12 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R_A = "6 kips", M_A = "-36 kip·ft", "θ_tip" = "0.0014020284", y_tip = "0.15141907 in", M_hog = "-36 kip·ft", y_peak = "0.15141907 in" }
tolerance = "1e-7"
figures = 1
checks = "pass"
+++

## Cantilever, uniform load

A prismatic beam built in at its left end A and free at the other, carrying a uniform load w over its whole length (Roark Table 8.1, case 2a). Linear elastic, small deflections, shear deformation neglected. Load the beam's self-weight into w if it matters. Deflection and slope are magnitudes, deflection downward; the moment at the fixed end is negative (hogging).

### Beam and load

```calc {description="Uniform load, including self-weight"}
w = 10 kN/m_
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
> Reaction at the fixed end, and the largest shear
R_A = w L = ?? kN
> Moment at the fixed end (hogging)
M_A = -w L^2 / 2 = ?? kN·m
> Slope at the free end
θ_tip = w L^3 / (6 E I) = ??
> Deflection at the free end
y_tip = w L^4 / (8 E I) = ?? mm
```

### Diagrams

The shear falls linearly from wL at the fixed end to zero at the tip, and the moment is a parabola from M_A to zero. Deflection is drawn downward.

```python {description="Shear, moment and deflection along the beam, and their diagrams"}
import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from cruncha import Q_, show

_L = L.to("m").magnitude
_w = w.to("N/m").magnitude
_EI = (E * I).to("N*m^2").magnitude
# Roark Table 8.1's expressions along x from A: the values at A, then the load's terms
_R = _w * _L
_M0 = -_w * _L**2 / 2
_th = 0.0
_keys = np.array([])
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

The plotted extremes should equal the closed-form values at the fixed end and the tip:

```calc {description="Largest hogging moment in the plotted diagram"}
M_hog = min(M_s) = ?? kN·m
```

```calc {description="Largest deflection in the plotted diagram"}
y_peak = max(y_s) = ?? mm
```
