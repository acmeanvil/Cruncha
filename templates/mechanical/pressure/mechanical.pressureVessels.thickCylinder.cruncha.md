+++
cruncha-template = 1
id = "mechanical.pressureVessels.thickCylinder"
version = "1.0.0"
title = "Thick-walled cylinder under pressure (Lamé, Roark)"
summary = "Lamé's thick-wall stresses in a cylinder under internal and external pressure, with capped or free ends: hoop, radial and longitudinal stress at the bore and the outside, von Mises and largest shear stress at the bore, the factor of safety on yield, and the changes in bore, outside diameter and length, after Roark Table 13.5 cases 1a to 1d."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "cylinder", "thick wall", "Lamé", "hoop stress", "radial stress", "tube", "hydraulic cylinder", "gun barrel", "von Mises", "Roark"]
from = "lacewing:mechanical.pressureVessels"
license = "GPL-3.0-or-later"
inputs = ["D_i", "t", "p_i", "p_o", "capped", "l", "E", "ν", "S_y", "n_d"]
outputs = ["b", "a", "σ_1", "σ_(2,b)", "σ_(3,b)", "σ_(2,a)", "σ_(3,a)", "σ_vm", "τ_max", "n_y", "ΔD_i", "ΔD_o", "Δl"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 13.5 and Table 13.5, cases 1a to 1d (thick-walled cylinder, internal and external pressure, ends free or capped), p. 683"

[[tests]]
name = "Roark case 1b: 200 mm bore, 300 mm outside, 10 MPa inside, ends capped"
source = "Lacewing Lab pressure_vessels_test, against Roark 7th Table 13.5 case 1b's own formulas (b = 100, a = 150 mm, E 200 GPa, ν 0.3): σ1 = qb²/(a² − b²) = 8 MPa, σ2 at b = q(a² + b²)/(a² − b²) = 26 MPa, τmax = qa²/(a² − b²) = 18 MPa, Δa = qa·b²(2 − ν)/(E(a² − b²)) (ΔDo = 0.0204 mm), Δb = qb[a²(1 + ν) + b²(1 − 2ν)]/(E(a² − b²)) (ΔDi = 0.0266 mm), Δl over 1 m = qb²(1 − 2ν)/(E(a² − b²)) = 0.016 mm; by hand σvm = 31.176915 MPa"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "10 MPa", p_o = "0 MPa", capped = "1", l = "1 m", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "σ_1" = "8 MPa", "σ_(2,b)" = "26 MPa", "σ_(3,b)" = "-10 MPa", "σ_(2,a)" = "16 MPa", "τ_max" = "18 MPa", "σ_vm" = "31.176915 MPa", "ΔD_i" = "0.0266 mm", "ΔD_o" = "0.0204 mm", "Δl" = "0.016 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Roark case 1a: the same cylinder, ends free"
source = "Lacewing Lab pressure_vessels_test, against Roark 7th Table 13.5 case 1a: σ1 = 0, Δa = 2qab²/(E(a² − b²)) (ΔDo = 0.024 mm), Δb = qb[(a² + b²)/(a² − b²) + ν]/E (ΔDi = 0.029 mm), Δl = −2qνb²l/(E(a² − b²)) = −0.024 mm; by hand σvm = 32.186954 MPa, n = 7.7671221"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "10 MPa", p_o = "0 MPa", capped = "0", l = "1 m", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "σ_1" = "0 MPa", "σ_vm" = "32.186954 MPa", n_y = "7.7671221", "ΔD_i" = "0.029 mm", "ΔD_o" = "0.024 mm", "Δl" = "-0.024 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Roark case 1d: 10 MPa outside, ends capped"
source = "Lacewing Lab pressure_vessels_test, against Roark 7th Table 13.5 case 1d: σ1 = −qa²/(a² − b²) = −18 MPa, σ2 at b = −2qa²/(a² − b²) = −36 MPa, Δa = −qa[a²(1 − 2ν) + b²(1 + ν)]/(E(a² − b²)) (ΔDo = −0.0264 mm), Δb = −qba²(2 − ν)/(E(a² − b²)) (ΔDi = −0.0306 mm)"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "0 MPa", p_o = "10 MPa", capped = "1", l = "1 m", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "σ_1" = "-18 MPa", "σ_(2,b)" = "-36 MPa", "σ_(3,a)" = "-10 MPa", "ΔD_i" = "-0.0306 mm", "ΔD_o" = "-0.0264 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Case 1b against a design factor of 10"
source = "By hand: n = 250/31.176915 = 8.0187537, under 10"
inputs = { D_i = "200 mm", t = "50 mm", p_i = "10 MPa", p_o = "0 MPa", capped = "1", l = "1 m", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "10" }
expect = { n_y = "8.0187537" }
tolerance = "1e-6"
checks = "fail"
+++

## Thick-walled cylinder under pressure

The stresses in a thick-walled cylinder under internal pressure p_i and external pressure p_o, by Lamé's solution: Roark Table 13.5 cases 1a and 1c (ends free) or 1b and 1d (ends capped), superposed. The hoop and radial stresses vary through the wall and are largest in magnitude at the bore, where the factor of safety on yield is taken; the longitudinal stress of capped ends is uniform. Use it for any wall; for R/t > 10 the thin-walled template gives nearly the same hoop stress.

Discontinuity stresses at heads, ports and changes of section are not included, and neither is buckling under external pressure.

### Geometry and loading

```calc {description="Inside diameter (the bore)"}
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

```calc {description="End condition: 1 ends capped (cases 1b, 1d), 0 ends free (cases 1a, 1c)"}
capped = 1_
```

```calc {description="Length whose change under pressure is wanted"}
l = 1 m_
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

With A = (p_i b² − p_o a²)/(a² − b²) and B = (p_i − p_o) a² b²/(a² − b²), the hoop stress at radius r is A + B/r² and the radial stress A − B/r². Capped ends carry A as a uniform longitudinal stress.

```calc
> Inner radius
b = D_i / 2 = ?? mm
> Outer radius
a = b + t = ?? mm
> Lamé's constant term
A_L = (p_i b^2 - p_o a^2) / (a^2 - b^2) = ?? MPa
> Lamé's 1/r² term
B_L = (p_i - p_o) a^2 b^2 / (a^2 - b^2) = ?? MPa·mm²
> Longitudinal stress (zero with free ends)
σ_1 = {A_L if capped == 1; 0 'MPa' if capped == 0} = ?? MPa
> Hoop stress at the bore
σ_(2,b) = A_L + B_L / b^2 = ?? MPa
> Radial stress at the bore (−p_i)
σ_(3,b) = A_L - B_L / b^2 = ?? MPa
> Hoop stress at the outside
σ_(2,a) = A_L + B_L / a^2 = ?? MPa
> Radial stress at the outside (−p_o)
σ_(3,a) = A_L - B_L / a^2 = ?? MPa
```

### Yield at the bore

```calc
> Von Mises stress at the bore
σ_vm = sqrt(((σ_1 - σ_(2,b))^2 + (σ_(2,b) - σ_(3,b))^2 + (σ_(3,b) - σ_1)^2) / 2) = ?? MPa
> Largest shear stress at the bore
τ_max = (max(σ_1, σ_(2,b), σ_(3,b)) - min(σ_1, σ_(2,b), σ_(3,b))) / 2 = ?? MPa
> Factor of safety on yield, distortion energy
n_y = S_y / σ_vm = ??
```

### Deformation

Hooke's law at each surface: Δr = r[σ2 − ν(σ3 + σ1)]/E, and the length changes by l[σ1 − ν(σ2 + σ3)]/E, where σ2 + σ3 = 2A is the same at every radius.

```calc
> Change in the bore
ΔD_i = 2 b / E (σ_(2,b) - ν (σ_(3,b) + σ_1)) = ?? mm
> Change in the outside diameter
ΔD_o = 2 a / E (σ_(2,a) - ν (σ_(3,a) + σ_1)) = ?? mm
> Change in length l
Δl = l / E (σ_1 - ν (σ_(2,b) + σ_(3,b))) = ?? mm
```

### Check

```calc
@test n_y ≥ n_d
```
