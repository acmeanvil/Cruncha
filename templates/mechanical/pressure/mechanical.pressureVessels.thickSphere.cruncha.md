+++
cruncha-template = 1
id = "mechanical.pressureVessels.thickSphere"
version = "1.0.0"
title = "Thick-walled sphere under pressure (Lamé, Roark)"
summary = "Lamé's thick-wall stresses in a hollow sphere under internal and external pressure: tangential and radial stress at the bore and the outside, von Mises and largest shear stress at the bore, the factor of safety on yield, the internal pressure at which the bore starts to yield, and the changes in bore and outside diameter, after Roark Table 13.5 cases 2a and 2b."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "sphere", "spherical shell", "thick wall", "Lamé", "hoop stress", "radial stress", "von Mises", "Roark"]
from = "lacewing:mechanical.pressureVessels"
license = "GPL-3.0-or-later"
inputs = ["D_i", "t", "p_i", "p_o", "E", "ν", "S_y", "n_d"]
outputs = ["b", "a", "σ_(2,b)", "σ_(3,b)", "σ_(2,a)", "σ_(3,a)", "σ_vm", "τ_max", "n_y", "q_y", "ΔD_i", "ΔD_o"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 13.5 and Table 13.5, cases 2a (internal pressure, with the yield pressure q_y) and 2b (external pressure), p. 685"

[[tests]]
name = "Roark case 2a: 200 mm bore, 300 mm outside, 10 MPa inside"
source = "Lacewing Lab pressure_vessels_test, against Roark 7th Table 13.5 case 2a's own formulas (b = 100, a = 150 mm, E 200 GPa, ν 0.3): σ2 at b = q(a³ + 2b³)/(2(a³ − b³)) = 11.315789 MPa, τmax = 3qa³/(4(a³ − b³)) = 10.657895 MPa, Δa = qa·3(1 − ν)b³/(2E(a³ − b³)) (ΔDo = 0.0066315789 mm), Δb = qb[(1 − ν)(a³ + 2b³)/(2(a³ − b³)) + ν]/E (ΔDi = 0.010921053 mm), qy = (2/3)250(1 − b³/a³) = 117.28395 MPa"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "10 MPa", p_o = "0 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "σ_(2,b)" = "11.315789 MPa", "σ_(3,b)" = "-10 MPa", "σ_(2,a)" = "6.3157895 MPa", "τ_max" = "10.657895 MPa", "σ_vm" = "21.315789 MPa", n_y = "11.728395", q_y = "117.28395 MPa", "ΔD_i" = "0.010921053 mm", "ΔD_o" = "0.0066315789 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Roark case 2b: 10 MPa outside"
source = "Lacewing Lab pressure_vessels_test, against Roark 7th Table 13.5 case 2b: σ2 at b = −3qa³/(2(a³ − b³)) = −21.315789 MPa, Δa = −qa[(1 − ν)(b³ + 2a³)/(2(a³ − b³)) − ν]/E (ΔDo = −0.012631579 mm), Δb = −qb·3(1 − ν)a³/(2E(a³ − b³)) (ΔDi = −0.014921053 mm)"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "0 MPa", p_o = "10 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "σ_(2,b)" = "-21.315789 MPa", "σ_(2,a)" = "-16.315789 MPa", "σ_(3,a)" = "-10 MPa", "ΔD_i" = "-0.014921053 mm", "ΔD_o" = "-0.012631579 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Case 2a against a design factor of 12"
source = "By hand: n = 250/21.315789 = 11.728395, under 12"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "10 MPa", p_o = "0 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "12" }
expect = { n_y = "11.728395" }
tolerance = "1e-6"
checks = "fail"
+++

## Thick-walled sphere under pressure

The stresses in a hollow sphere under internal pressure p_i and external pressure p_o, by Lamé's solution: Roark Table 13.5 cases 2a and 2b, superposed. The two tangential stresses are equal; they and the radial stress are largest in magnitude at the bore, where the factor of safety on yield is taken. Use it for any wall; for R/t > 10 the thin-walled sphere template gives nearly the same stress.

### Geometry and loading

```calc {description="Inside diameter"}
D_i = 200 mm_
```

```calc {description="Wall thickness"}
t = 50 mm_
```

```calc {description="Internal (gauge) pressure"}
p_i = 10 MPa_
```

```calc {description="External (gauge) pressure"}
p_o = 0 MPa_
```

### Material

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Yield strength"}
S_y = 250 MPa_
```

```calc {description="Design factor the factor of safety on yield must reach"}
n_d = 1.5_
```

### Lamé stresses

With A = (p_i b³ − p_o a³)/(a³ − b³) and B = (p_i − p_o) a³ b³/(a³ − b³), the tangential stress at radius r is A + B/(2r³) and the radial stress A − B/r³.

```calc
> Inner radius
b = D_i / 2 = ?? mm
> Outer radius
a = b + t = ?? mm
> Lamé's constant term
A_L = (p_i b^3 - p_o a^3) / (a^3 - b^3) = ?? MPa
> Lamé's 1/r³ term
B_L = (p_i - p_o) a^3 b^3 / (a^3 - b^3) = ?? MPa·mm³
> Tangential stress at the bore
σ_(2,b) = A_L + B_L / (2 b^3) = ?? MPa
> Radial stress at the bore (−p_i)
σ_(3,b) = A_L - B_L / b^3 = ?? MPa
> Tangential stress at the outside
σ_(2,a) = A_L + B_L / (2 a^3) = ?? MPa
> Radial stress at the outside (−p_o)
σ_(3,a) = A_L - B_L / a^3 = ?? MPa
```

### Yield at the bore

With two equal tangential stresses, the von Mises stress is their difference from the radial stress, and the largest shear stress half of it. Roark's q_y is the internal pressure alone (no external pressure) at which the bore starts to yield.

```calc
> Von Mises stress at the bore
σ_vm = abs(σ_(2,b) - σ_(3,b)) = ?? MPa
> Largest shear stress at the bore
τ_max = σ_vm / 2 = ?? MPa
> Factor of safety on yield, distortion energy
n_y = S_y / σ_vm = ??
> Internal pressure at which the bore starts to yield, case 2a
q_y = 2 S_y / 3 (1 - b^3 / a^3) = ?? MPa
```

### Deformation

Hooke's law at each surface: Δr = r[(1 − ν)σ2 − νσ3]/E.

```calc
> Change in the bore
ΔD_i = 2 b / E ((1 - ν) σ_(2,b) - ν σ_(3,b)) = ?? mm
> Change in the outside diameter
ΔD_o = 2 a / E ((1 - ν) σ_(2,a) - ν σ_(3,a)) = ?? mm
```

### Check

```calc
@test n_y ≥ n_d
```
