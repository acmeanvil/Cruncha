+++
cruncha-template = 1
id = "mechanical.pressureVessels.thinSphere"
version = "1.0.0"
title = "Thin-walled sphere under pressure (Roark)"
summary = "Membrane stress of a thin-walled spherical shell under internal and external pressure: the equal meridional and hoop stress on the mid-surface radius, von Mises stress, the factor of safety on yield and the growth in diameter, after Roark Table 13.1 case 3a."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "sphere", "spherical shell", "thin wall", "membrane stress", "hoop stress", "von Mises", "Roark"]
from = "lacewing:mechanical.pressureVessels"
license = "GPL-3.0-or-later"
inputs = ["D_i", "t", "p_i", "p_o", "E", "ν", "S_y", "n_d"]
outputs = ["q", "R", "R_t", "σ", "σ_vm", "n_y", "ΔD"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 13.1 and Table 13.1, case 3a (sphere under uniform pressure), p. 597; R/t > 10 for the membrane formulas"

[[tests]]
name = "Lacewing Lab: 2 m bore, 10 mm wall, 1 MPa, steel"
source = "Lacewing Lab pressure_vessels_test (thin sphere): σ = qR/2t = 1 × 1005/20 = 50.25 MPa; by hand ΔD = qR²(1 − ν)/(Et) = 0.35350875 mm (E 200 GPa, ν 0.3), n = 250/50.25 = 4.9751244"
inputs = { D_i = "2000 mm", t = "10 mm", p_i = "1 MPa", p_o = "0 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { R = "1005 mm", R_t = "100.5", "σ" = "50.25 MPa", "σ_vm" = "50.25 MPa", n_y = "4.9751244", "ΔD" = "0.35350875 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: 1 m bore, 8 mm wall, 2 MPa; a design factor of 4 not met"
source = "By hand, Roark Table 13.1 case 3a: R = 504 mm, R/t = 63, σ = 2 × 504/16 = 63 MPa, ΔD = 2 × 504²(0.7)/(200 000 × 8) = 0.222264 mm, n = 250/63 = 3.968254, under 4"
inputs = { D_i = "1000 mm", t = "8 mm", p_i = "2 MPa", p_o = "0 MPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "4" }
expect = { R_t = "63", "σ" = "63 MPa", "ΔD" = "0.222264 mm", n_y = "3.968254" }
tolerance = "1e-6"
checks = "fail"
+++

## Thin-walled sphere under pressure

The membrane stress in a thin-walled spherical shell under internal pressure p_i and external pressure p_o, after Roark Table 13.1 case 3a. The meridional and hoop stresses are equal, uniform through the wall and act on the mid-surface radius R; the radial stress is neglected. Roark limits the membrane formulas to R/t > 10; for a thicker wall use the thick-walled sphere template.

Discontinuity stresses at nozzles and supports are not included, buckling under net external pressure (Roark ch. 15) is not checked, and this is not a design-code calculation (for a hemispherical head to ASME VIII-1, see the pressure vessel head templates).

### Geometry and loading

```calc {description="Inside diameter"}
D_i = 2000 mm_
```

```calc {description="Wall thickness, less any corrosion allowance"}
t = 10 mm_
```

```calc {description="Internal (gauge) pressure"}
p_i = 1 MPa_
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

### Membrane stress

```calc
> Net pressure
q = p_i - p_o = ?? MPa
> Mid-surface radius
R = D_i / 2 + t / 2 = ?? mm
> Slenderness; the membrane formulas need R/t > 10
R_t = R / t = ??
> Meridional and hoop stress, Table 13.1 case 3a
σ = q R / (2 t) = ?? MPa
```

With two equal principal stresses and the radial stress neglected, the von Mises stress is the membrane stress itself.

```calc
> Von Mises stress
σ_vm = abs(σ) = ?? MPa
> Factor of safety on yield, distortion energy
n_y = S_y / σ_vm = ??
> Change in mid-surface diameter, Table 13.1 case 3a
ΔD = q R^2 (1 - ν) / (E t) = ?? mm
```

### Checks

```calc
@test R_t > 10
```

```calc
@test n_y ≥ n_d
```
