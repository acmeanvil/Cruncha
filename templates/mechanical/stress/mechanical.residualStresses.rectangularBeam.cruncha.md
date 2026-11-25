+++
cruncha-template = 1
id = "mechanical.residualStresses.rectangularBeam"
version = "1.0.0"
title = "Residual stresses: rectangular beam bent past yield"
summary = "A rectangular beam of elastic-perfectly plastic material bent past its yield moment and unloaded: the yield depth, the residual stresses at the surface and the core edge, the moment that holds it straight, the elastic capacity left in each direction, and the curvature it keeps."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["residual stress", "yielding", "plastic bending", "overload", "elastic-plastic", "plastic moment", "yield moment", "springback", "autofrettage"]
from = "lacewing:mechanical.residualStresses"
license = "GPL-3.0-or-later"
inputs = ["b", "h", "S_y", "E", "M", "M_r"]
outputs = ["M_e", "M_p", "y_e", "a_y", "σ_(r,s)", "σ_(r,c)", "M_s", "M_fwd", "M_rev", "κ_L", "κ_r"]

[[references]]
work = "Juvinall and Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 4.14-4.15 (pp. 166-173), Figs. 4.42e and 4.44"

[[tests]]
name = "Juvinall Fig. 4.44: 25 × 50 mm steel beam, Sy 300 MPa, yielded 10 mm deep"
source = "Juvinall 5th Sec. 4.15, Fig. 4.44 (pp. 171-173), Lacewing Lab residual_stress_test: M = 4125 N·m; residual −96 MPa at the surface, 62 MPa at the core edge; 1083 N·m holds it straight; 2125 N·m the other way stays elastic"
inputs = { b = "25 mm", h = "50 mm", S_y = "300 MPa", E = "200 GPa", M = "4125 N·m", M_r = "2000 N·m" }
expect = { M_e = "3125 N·m", M_p = "4687.5 N·m", a_y = "10 mm", "σ_(r,s)" = "-96 MPa", "σ_(r,c)" = "62.4 MPa", M_s = "1083.33 N·m", "M_rev" = "2125 N·m" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "The same beam: the curvature it keeps"
source = "By hand, plane sections: under load κ = Sy/(E·ye) = 300/(200 000 × 15 mm) = 0.1 /m; after unloading 0.1 − M/(EI) = 0.1 − 4125/(200 GPa × 2.604e-7 m⁴) = 0.0208 /m (radius 48.1 m)"
inputs = { b = "25 mm", h = "50 mm", S_y = "300 MPa", E = "200 GPa", M = "4125 N·m", M_r = "2000 N·m" }
expect = { "κ_L" = "0.1 m^-1", "κ_r" = "0.0208 m^-1" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Below the yield moment: nothing is left behind"
source = "By hand: Me = 250 MPa × 20 × 40²/6 mm³ = 1333 N·m > 1000 N·m, so the beam stays elastic"
inputs = { b = "20 mm", h = "40 mm", S_y = "250 MPa", E = "200 GPa", M = "1000 N·m", M_r = "1000 N·m" }
expect = { a_y = "0 mm", "σ_(r,s)" = "0 MPa", "κ_r" = "0 m^-1" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Juvinall's beam reversed at 2500 N·m yields again"
source = "Juvinall 5th Fig. 4.44e (p. 173): after the overload only 2125 N·m the other way is elastic"
inputs = { b = "25 mm", h = "50 mm", S_y = "300 MPa", E = "200 GPa", M = "4125 N·m", M_r = "2500 N·m" }
expect = { "M_rev" = "2125 N·m" }
tolerance = "0.01%"
checks = "fail"
+++

## Residual stresses: rectangular beam bent past yield

A rectangular beam bent about the axis parallel to its width b, past the moment that first yields its surfaces, then unloaded. The material is idealized as elastic-perfectly plastic, with the same yield strength in tension and compression (Juvinall Fig. 4.42e). Under load the outer fibres to a depth a_y sit at S_y and an elastic core of half-depth y_e carries the rest. Unloading is elastic, so the residual stress is the stress under load less M·y/I. It is opposite in sign to the load stress at the surface.

Plane sections stay plane. Strain hardening, the Bauschinger effect and stress concentrations are not included. The beam does not yield again on unloading, because the stress removed at the surface, M·c/I, stays below 1.5 S_y.

### Beam, material and loads

```calc {description="Width of the section (parallel to the bending axis)"}
b = 25 mm_
```

```calc {description="Depth of the section"}
h = 50 mm_
```

```calc {description="Yield strength (flat-top stress-strain curve)"}
S_y = 300 MPa_
```

```calc {description="Modulus of elasticity, for the curvature"}
E = 200 GPa_
```

```calc {description="Bending moment applied, then removed (between the yield and plastic moments to yield)"}
M = 4125 N·m_
```

```calc {description="Later service moment in the opposite direction"}
M_r = 2000 N·m_
```

### Section

```calc
> Distance from the neutral axis to the surface
c = h / 2 = ?? mm
> Second moment of area
I = b h^3 / 12 = ?? mm⁴
> Yield moment: the surfaces just reach S_y
M_e = S_y b h^2 / 6 = ?? N·m
> Plastic moment: the whole section yielded
M_p = S_y b h^2 / 4 = ?? N·m
```

### Under load

The moment of the stress block, a rectangle of S_y over each yielded layer and a triangle over the core, is M = S_y b (c² − y_e²/3) (Juvinall p. 171). Solved for the core:

```calc
> Half-depth of the elastic core (c while the beam is elastic)
y_e = { c if M ≤ M_e; sqrt(3 (c^2 - M / (S_y b))) otherwise } = ?? mm
> Depth of yielding from each surface
a_y = c - y_e = ?? mm
```

### Residual stresses after unloading

```calc
> Elastic stress removed at the surface on unloading
σ_e = M c / I = ?? MPa
> Residual stress at the surface, Juvinall Fig. 4.44b
σ_(r,s) = { 0 'MPa' if M ≤ M_e; S_y - σ_e otherwise } = ?? MPa
> Residual stress at the edge of the elastic core, Fig. 4.44b
σ_(r,c) = { 0 'MPa' if M ≤ M_e; S_y - M y_e / I otherwise } = ?? MPa
```

### What the overload leaves

```calc
> Moment, opposite to M, that frees the core of stress and holds the beam straight, Fig. 4.44c
M_s = { 0 'N·m' if M ≤ M_e; S_y I / y_e - M otherwise } = ?? N·m
> Elastic capacity afterwards in the direction of the overload, Fig. 4.44d
M_fwd = max(M, M_e) = ?? N·m
> Elastic capacity afterwards in the opposite direction, Fig. 4.44e
M_rev = { M_e if M ≤ M_e; 2 M_e - M otherwise } = ?? N·m
```

The overload raises the elastic capacity in its own direction and lowers it in the other by the same amount.

### Curvature

```calc
> Curvature under load: the core's edge is at yield strain
κ_L = { M / (E I) if M ≤ M_e; S_y / (E y_e) otherwise } = ?? m^-1
> Curvature the beam keeps after unloading
κ_r = κ_L - M / (E I) = ?? m^-1
```

### Checks

```calc {description="Not wholly yielded: the section can still carry the moment"}
@test M < M_p
```

```calc {description="A later reversed moment stays elastic"}
@test M_r ≤ M_rev
```
