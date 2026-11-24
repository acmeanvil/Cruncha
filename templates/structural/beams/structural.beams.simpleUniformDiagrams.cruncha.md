+++
cruncha-template = 1
id = "structural.beams.simpleUniformDiagrams"
version = "1.0.0"
title = "Simply supported beam, uniform load: reactions, moment, deflection and diagrams"
summary = "Reactions, maximum shear and moment, end slope and mid-span deflection of a simply supported beam under a uniform load over its whole span, with shear and moment diagrams."
category = ["Structural engineering", "Beams"]
tags = ["beam", "simply supported", "uniform load", "UDL", "shear diagram", "moment diagram", "deflection", "Roark", "Plotly", "Python"]
from = "lacewing:beams.roark"
license = "GPL-3.0-or-later"
inputs = ["w", "L", "E", "I"]
outputs = ["R", "M_max", "θ_A", "y_max", "M_peak"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas, Sadegh)"
edition = "8th"
locator = "Table 8.1, case 2e (uniformly distributed load, both ends simply supported)"

[[tests]]
name = "Roark 8.1 case 2e: 10 kN/m over 6 m, EI = 2 × 10⁷ N·m²"
source = "Lacewing Lab beam_test: M = wl²/8 = 45 kN·m, y = 5wl⁴/384EI = 8.4375 mm, θ = wl³/24EI = 0.0045 rad"
inputs = { w = "10 kN/m", L = "6 m", E = "200 GPa", I = "1e8 mm^4" }
expect = { R = "30 kN", M_max = "45 kN·m", "θ_A" = "0.0045", y_max = "8.4375 mm", M_peak = "45 kN·m" }
tolerance = "1e-9"
figures = 1

[[tests]]
name = "1.2 kip/ft over 24 ft"
source = "By hand: R = 1.2 × 24/2 = 14.4 kips, M = 1.2 × 24²/8 = 86.4 kip·ft"
inputs = { w = "1.2 kip/ft", L = "24 ft", E = "29000 ksi", I = "510 in^4" }
expect = { R = "14.4 kips", M_max = "86.4 kip·ft", M_peak = "86.4 kip·ft" }
tolerance = "1e-9"
figures = 1
+++

## Simply supported beam, uniform load

A prismatic beam on a pin and a roller, carrying a uniform load w over its whole span L. Small deflections, linear elastic material, shear deformation neglected (Roark Table 8.1, case 2e). Load the beam's self-weight into w if it matters.

### Beam and load

```calc {description="Uniform load, including self-weight"}
w = 10 kN/m_
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

### Closed form

```calc
> Each reaction, and the maximum shear
R = w L / 2 = ?? kN
> Maximum moment, at mid-span
M_max = w L^2 / 8 = ?? kN·m
> End slope
θ_A = w L^3 / (24 E I) = ??
> Maximum deflection, at mid-span
y_max = 5 w L^4 / (384 E I) = ?? mm
```

### Diagrams

The shear falls linearly from R to −R, and the moment is a parabola with its peak at mid-span.

```python {description="Shear and moment along the span, and their diagrams"}
import numpy as np
import plotly.graph_objects as go
from plotly.subplots import make_subplots
from cruncha import Q_, show

span = L.to("m").magnitude
load = w.to("kN/m").magnitude
x_s = np.linspace(0, span, 101)
V_s = Q_(load * (span / 2 - x_s), "kN")
M_s = Q_(load * x_s * (span - x_s) / 2, "kN*m")

fig = make_subplots(rows=2, cols=1, shared_xaxes=True, subplot_titles=("Shear (kN)", "Moment (kN·m)"))
fig.add_trace(go.Scatter(x=x_s, y=V_s.magnitude, fill="tozeroy", name="V"), row=1, col=1)
fig.add_trace(go.Scatter(x=x_s, y=M_s.magnitude, fill="tozeroy", name="M"), row=2, col=1)
fig.update_xaxes(title_text="x (m)", row=2, col=1)
fig.update_layout(showlegend=False)
show(fig)
```

The plotted moment should peak at the closed-form value:

```calc {description="Largest moment in the plotted diagram"}
M_peak = max(M_s) = ?? kN·m
```
