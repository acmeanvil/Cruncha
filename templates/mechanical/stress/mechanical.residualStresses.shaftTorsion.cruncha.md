+++
cruncha-template = 1
id = "mechanical.residualStresses.shaftTorsion"
version = "1.0.0"
title = "Residual stresses: round shaft twisted past yield"
summary = "A solid or hollow round shaft of elastic-perfectly plastic material twisted past its yield torque and unloaded: the elastic core and yield depth, the residual shear stresses at the surface and the core edge, the elastic capacity left in each direction, and the twist it keeps."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["residual stress", "yielding", "torsion", "shaft", "plastic torque", "yield torque", "overload", "elastic-plastic", "autofrettage", "presetting"]
from = "lacewing:mechanical.residualStresses"
license = "GPL-3.0-or-later"
inputs = ["d", "d_i", "τ_y", "G", "T", "T_r"]
outputs = ["J", "T_e", "T_p", "ρ", "a_y", "τ_(r,s)", "τ_(r,c)", "T_fwd", "T_rev", "θ_L", "θ_r"]

[[references]]
work = "Juvinall and Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 4.14-4.15 (pp. 166-173): the beam of Fig. 4.44 worked as a shaft, as p. 173 says round bars in torsion are treated"

[[tests]]
name = "Hollow shaft 50/20 mm, τy 120 MPa, 3500 N·m"
source = "Lacewing Lab residual_stress_test (the torque given finds the core radius), checked by hand: Te = τy·J/c = 2870 N·m, Tp = 2πτy(c³ − ci³)/3 = 3676 N·m; T(ρ) = 2πτy((ρ⁴ − ci⁴)/(4ρ) + (c³ − ρ³)/3) = 3500 N·m at ρ = 17.153 mm; τ = T·c/J = 146.35 MPa, residual 120 − 146.35 = −26.35 MPa at the surface and 120 − 3500 × 17.153/J = 19.59 MPa at the core edge; 2Te − T = 2240 N·m"
inputs = { d = "50 mm", d_i = "20 mm", "τ_y" = "120 MPa", G = "80 GPa", T = "3500 N·m", T_r = "2000 N·m" }
expect = { T_e = "2869.84 N·m", T_p = "3675.66 N·m", "ρ" = "17.1529 mm", a_y = "7.8471 mm", "τ_(r,s)" = "-26.349 MPa", "τ_(r,c)" = "19.587 MPa", T_rev = "2239.69 N·m" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Solid 40 mm shaft, τy 150 MPa, 2400 N·m: Tp = (4/3)Te, and the twist kept"
source = "Lacewing Lab residual_stress_test (solid: Tp/Te = 4/3), by hand: Te = 150 MPa × π × 20³/2 mm³ = 1885.0 N·m, Tp = 2513.3 N·m; ρ = 11.298 mm; twist under load τy/(Gρ) = 0.16595 rad/m, kept τy/(Gρ) − T/(GJ) = 0.046588 rad/m"
inputs = { d = "40 mm", d_i = "0 mm", "τ_y" = "150 MPa", G = "80 GPa", T = "2400 N·m", T_r = "1000 N·m" }
expect = { T_e = "1884.96 N·m", T_p = "2513.27 N·m", "ρ" = "11.2983 mm", "τ_(r,s)" = "-40.986 MPa", "θ_L" = "0.165954 rad/m", "θ_r" = "0.0465878 rad/m" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Solid shaft reversed at 1500 N·m yields again"
source = "By hand, as the solid case: after 2400 N·m only 2Te − T = 1369.9 N·m the other way is elastic"
inputs = { d = "40 mm", d_i = "0 mm", "τ_y" = "150 MPa", G = "80 GPa", T = "2400 N·m", T_r = "1500 N·m" }
expect = { T_rev = "1369.91 N·m" }
tolerance = "0.01%"
checks = "fail"
+++

## Residual stresses: round shaft twisted past yield

A round shaft, solid or hollow, twisted past the torque that first yields its surface, then unloaded. The material is elastic-perfectly plastic in shear. Under load the outer ring down to the radius ρ sits at τ_y, and an elastic core carries the rest. Unloading is elastic, so the residual stress is the stress under load less T·r/J. It is opposite in sign to the load stress at the surface. Juvinall works the beam (Sec. 4.15) and notes that round bars in torsion are treated the same way; the equations below are that working.

Plane sections stay plane, and radii stay straight. Strain hardening, the Bauschinger effect and stress concentrations are not included. The shaft does not yield again on unloading, because the stress removed at the surface, T·c/J, stays below 2 τ_y.

### Shaft, material and loads

```calc {description="Outside diameter"}
d = 50 mm_
```

```calc {description="Bore diameter (0 for a solid shaft)"}
d_i = 20 mm_
```

The shear yield strength is about S_y/√3 by the distortion-energy theory (0.577 S_y), or S_y/2 by Tresca.

```calc {description="Shear yield strength"}
τ_y = 120 MPa_
```

```calc {description="Shear modulus, for the twist"}
G = 80 GPa_
```

```calc {description="Torque applied, then removed (between the yield and plastic torques to yield)"}
T = 3500 N·m_
```

```calc {description="Later service torque in the opposite direction"}
T_r = 2000 N·m_
```

### Section

```calc
> Outer radius
c = d / 2 = ?? mm
> Bore radius
c_i = d_i / 2 = ?? mm
> Polar second moment of area
J = π (c^4 - c_i^4) / 2 = ?? mm⁴
> Yield torque: the surface just reaches τ_y
T_e = τ_y J / c = ?? N·m
> Plastic torque: the whole wall yielded
T_p = 2 π τ_y (c^3 - c_i^3) / 3 = ?? N·m
```

### Under load

With an elastic core of radius ρ the torque is T = 2π τ_y ((ρ⁴ − c_i⁴)/(4ρ) + (c³ − ρ³)/3): the core's linear stress plus the yielded ring at τ_y. It falls from T_p to T_e as ρ grows from c_i to c, so this function finds ρ by bisection.

```calc
function coreRadius(T, c, c_i, τ_y)
    lo = c_i
    hi = c
    for k in 1:60
        r = (lo + hi) / 2
        T_k = 2 π τ_y ((r^4 - c_i^4) / (4 r) + (c^3 - r^3) / 3)
        if T_k > T
            lo = r
        else
            hi = r
        end
    end
    return (lo + hi) / 2
end
```

```calc
> Radius of the elastic core (c while the shaft is elastic)
ρ = { c if T ≤ T_e; coreRadius(T, c, c_i, τ_y) otherwise } = ?? mm
> Depth of yielding from the surface
a_y = c - ρ = ?? mm
> The torque at that core, to check the solution
T_ρ = 2 π τ_y ((ρ^4 - c_i^4) / (4 ρ) + (c^3 - ρ^3) / 3) = ?? N·m
```

### Residual stresses after unloading

```calc
> Elastic stress removed at the surface on unloading
τ_e = T c / J = ?? MPa
> Residual shear stress at the surface
τ_(r,s) = { 0 'MPa' if T ≤ T_e; τ_y - τ_e otherwise } = ?? MPa
> Residual shear stress at the edge of the elastic core
τ_(r,c) = { 0 'MPa' if T ≤ T_e; τ_y - T ρ / J otherwise } = ?? MPa
```

### What the overload leaves

```calc
> Elastic capacity afterwards in the direction of the overload
T_fwd = max(T, T_e) = ?? N·m
> Elastic capacity afterwards in the opposite direction
T_rev = { T_e if T ≤ T_e; 2 T_e - T otherwise } = ?? N·m
```

### Twist

```calc
> Twist per length under load: the core's edge is at yield strain
θ_L = { T / (G J) if T ≤ T_e; τ_y / (G ρ) otherwise } = ?? rad/m
> Twist per length the shaft keeps after unloading
θ_r = θ_L - T / (G J) = ?? rad/m
```

### Checks

```calc {description="Not wholly yielded: the wall can still carry the torque"}
@test T < T_p
```

```calc {description="A later reversed torque stays elastic"}
@test T_r ≤ T_rev
```
