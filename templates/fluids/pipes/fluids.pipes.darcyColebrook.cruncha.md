+++
cruncha-template = 1
id = "fluids.pipes.darcyColebrook"
version = "1.0.0"
title = "Pipe pressure drop: Darcy–Weisbach with the Colebrook friction factor"
summary = "Velocity, Reynolds number, Darcy friction factor (Colebrook, solved numerically; 64/Re when laminar), pressure drop and head loss for incompressible flow in a straight round pipe."
category = ["Fluids", "Pipe flow"]
tags = ["pipe flow", "pressure drop", "head loss", "Darcy-Weisbach", "Colebrook", "friction factor", "Reynolds number", "Moody", "Python"]
from = "lacewing:fluids.pipeFlow"
license = "GPL-3.0-or-later"
inputs = ["Q", "D", "L", "ε", "ρ", "μ"]
outputs = ["v", "Re", "f", "Δp", "h_f"]

[[references]]
work = "Crane Co., Flow of Fluids Through Valves, Fittings, and Pipe, Technical Paper No. 410"
edition = "2009"
locator = "Ch. 1 (Darcy equation, Colebrook equation, Eqs. 1-1 to 1-6); Appendix A-23 to A-26; Example 4-14"

[[references]]
work = "Colebrook, C.F., Turbulent flow in pipes, with particular reference to the transition region between the smooth and rough pipe laws, J. Institution of Civil Engineers 11(4)"
edition = "1939"

[[tests]]
name = "Crane Example 4-14: 1500 L/min of water in 67 m of 5 in. Schedule 40"
source = "Lacewing Lab fluids_test, after Crane TP-410 Example 4-14: v ≈ 1.94 m/s, f ≈ 0.018 read from the chart"
inputs = { Q = "1500 L/min", D = "128.2 mm", L = "67 m", "ε" = "0.045 mm", "ρ" = "999 kg/m^3", "μ" = "1.1 mPa·s" }
expect = { v = "1.94 m/s", f = "0.018" }
tolerance = "1.5%"

[[tests]]
name = "The same flow, to solver precision"
source = "An independent bisection solution of the Colebrook equation: Re 225 494, f 0.0177893, Δp 17.419 kPa"
inputs = { Q = "1500 L/min", D = "128.2 mm", L = "67 m", "ε" = "0.045 mm", "ρ" = "999 kg/m^3", "μ" = "1.1 mPa·s" }
expect = { Re = "225493.96", f = "0.01778928", "Δp" = "17.41922 kPa", h_f = "1.778044 m" }
tolerance = "1e-5"

[[tests]]
name = "Oil in a 50 mm line: laminar, f = 64/Re"
source = "By hand: Re = 880 × 0.2546 × 0.05/0.1 = 112.0, f = 64/112.0 = 0.5712"
inputs = { Q = "0.5 L/s", D = "50 mm", L = "20 m", "ε" = "0.045 mm", "ρ" = "880 kg/m^3", "μ" = "0.1 Pa·s" }
expect = { Re = "112.045", f = "0.571199", "Δp" = "6.51899 kPa" }
tolerance = "1e-5"
+++

## Pipe pressure drop: Darcy–Weisbach with Colebrook

Frictional pressure drop of an incompressible fluid in a straight, round, full pipe at steady flow. Fittings, valves, entrance and exit losses and changes of elevation are separate (Crane gives them as resistance coefficients K).

The Darcy friction factor comes from the Colebrook equation, which is implicit in f, so a short Python cell solves it. Below Re = 2000 the flow is laminar and f = 64/Re. Between 2000 and about 4000 the flow is transitional and neither is reliable.

### Flow

```calc {description="Volumetric flow rate"}
Q = 1500 L/min_
```

```calc {description="Inside diameter"}
D = 128.2 mm_
```

```calc {description="Length of straight pipe"}
L = 67 m_
```

```calc {description="Absolute roughness (commercial steel 0.045 mm)"}
ε = 0.045 mm_
```

```calc {description="Fluid density"}
ρ = 999 kg/m³_
```

```calc {description="Dynamic viscosity"}
μ = 1.1 mPa·s_
```

### Reynolds number

```calc
> Mean velocity
v = Q / (π D^2 / 4) = ?? m/s
> Reynolds number
Re = ρ v D / μ = ??
> Relative roughness
ε_r = ε / D = ??
```

### Friction factor

```python {description="Darcy friction factor: Colebrook (turbulent) or 64/Re (laminar)"}
import math
from scipy.optimize import brentq

Re_ = float(Re)
if Re_ < 2000:
    f = 64 / Re_
else:
    f = brentq(lambda x: 1 / math.sqrt(x) + 2 * math.log10(float(ε_r) / 3.7 + 2.51 / (Re_ * math.sqrt(x))), 1e-4, 1.0, xtol=1e-14)
```

### Pressure drop

```calc {description="Standard gravity"}
g_n = 9.80665 m/s²
```

```calc
> Pressure drop, Darcy–Weisbach
Δp = f (L / D) ρ v^2 / 2 = ?? kPa
> Head loss
h_f = Δp / (ρ g_n) = ?? m
```
