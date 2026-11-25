+++
cruncha-template = 1
id = "mechanical.pressureVessels.thinCylinder"
version = "1.0.0"
title = "Thin-walled cylinder under pressure (Roark)"
summary = "Membrane stresses of a thin-walled cylinder under internal and external pressure, with capped or open ends: hoop and longitudinal stress on the mid-surface radius, von Mises stress, the factor of safety on yield, and the growth in diameter and change in length, after Roark Table 13.1 cases 1b and 1c."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["pressure vessel", "cylinder", "thin wall", "membrane stress", "hoop stress", "longitudinal stress", "pipe", "tube", "von Mises", "Roark"]
from = "lacewing:mechanical.pressureVessels"
license = "GPL-3.0-or-later"
inputs = ["D_i", "t", "p_i", "p_o", "capped", "l", "E", "ν", "S_y", "n_d"]
outputs = ["q", "R", "R_t", "σ_2", "σ_1", "σ_vm", "τ_max", "n_y", "ΔD", "Δl"]

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 13.1 and Table 13.1, cases 1b (no end load) and 1c (end load, ends capped), pp. 592-593; R/t > 10 for the membrane formulas"

[[tests]]
name = "Hand check: the Lacewing Lab opening cylinder, 600 mm bore, 12.7 mm wall, 2 MPa, capped"
source = "By hand, Roark Table 13.1 case 1c: R = 300 + 6.35 = 306.35 mm, R/t = 24.122, σ2 = 2 × 306.35/12.7 = 48.244094 MPa, σ1 = 24.122047 MPa, σvm = √(σ1² − σ1σ2 + σ2²) = 41.780611 MPa, n = 205/41.780611 = 4.9065821, ΔD = 2qR²(1 − ν/2)/(Et) = 0.13018281 mm (E 193 GPa, ν 0.3), Δl over 1 m = qRl(0.5 − ν)/(Et) = 0.049993880 mm"
inputs = { D_i = "600 mm", t = "12.7 mm", p_i = "2 MPa", p_o = "0 MPa", capped = "1", l = "1 m", E = "193 GPa", "ν" = "0.3", S_y = "205 MPa", n_d = "1.5" }
expect = { R = "306.35 mm", R_t = "24.122047", "σ_2" = "48.244094 MPa", "σ_1" = "24.122047 MPa", "σ_vm" = "41.780611 MPa", n_y = "4.9065821", "ΔD" = "0.13018281 mm", "Δl" = "0.049993880 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: the same cylinder with open ends"
source = "By hand, Roark Table 13.1 case 1b: σ1 = 0, σvm = σ2 = 48.244094 MPa, n = 4.2492247, ΔD = 2qR²/(Et) = 0.15315625 mm, Δl = −qRνl/(Et) = −0.074990820 mm"
inputs = { D_i = "600 mm", t = "12.7 mm", p_i = "2 MPa", p_o = "0 MPa", capped = "0", l = "1 m", E = "193 GPa", "ν" = "0.3", S_y = "205 MPa", n_d = "1.5" }
expect = { "σ_1" = "0 MPa", "σ_vm" = "48.244094 MPa", n_y = "4.2492247", "ΔD" = "0.15315625 mm", "Δl" = "-0.074990820 mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Lacewing Lab: 1 m bore, 5 mm wall, 1 MPa, capped, steel"
source = "Lacewing Lab pressure_vessels_test (thin wall): σ2 = qR/t = 1 × 502.5/5 = 100.5 MPa, σ1 = 50.25 MPa, σvm = (√3/2)σ2 = 87.035553 MPa; by hand ΔD = 0.42926062 mm and Δl over 2 m = 0.201 mm (E 200 GPa, ν 0.3). Against Sy 120 MPa the factor of safety 1.38 is under a design factor of 1.5"
inputs = { D_i = "1000 mm", t = "5 mm", p_i = "1 MPa", p_o = "0 MPa", capped = "1", l = "2 m", E = "200 GPa", "ν" = "0.3", S_y = "120 MPa", n_d = "1.5" }
expect = { "σ_2" = "100.5 MPa", "σ_1" = "50.25 MPa", "σ_vm" = "87.035553 MPa", "ΔD" = "0.42926062 mm", "Δl" = "0.201 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Thin-walled cylinder under pressure

The membrane stresses in a thin-walled cylindrical shell under internal pressure p_i and external pressure p_o, after Roark Table 13.1. The stresses are uniform through the wall and act on the mid-surface radius R; the radial stress is neglected. Roark limits these formulas to R/t > 10; for a thicker wall use the thick-walled cylinder template.

They are the stresses in the shell away from its ends, nozzles and changes of shape. The discontinuity stresses where the shell meets a head are not included, buckling under net external pressure (Roark ch. 15) is not checked, and this is not a design-code (ASME VIII) calculation.

### Geometry and loading

```calc {description="Inside diameter (the bore)"}
D_i = 600 mm_
```

```calc {description="Wall thickness, less any corrosion allowance"}
t = 12.7 mm_
```

```calc {description="Internal (gauge) pressure"}
p_i = 2 MPa_
```

```calc {description="External (gauge) pressure"}
p_o = 0 MPa_
```

Capped ends carry the pressure on the ends into the wall as a longitudinal stress. Open ends, or ends whose load is carried elsewhere (a piston, tie rods), give no longitudinal stress.

```calc {description="End condition: 1 ends capped (end load in the wall, case 1c), 0 open ends (case 1b)"}
capped = 1_
```

```calc {description="Length whose change under pressure is wanted"}
l = 1 m_
```

### Material

E and ν set the growth only; the stresses do not depend on them.

```calc {description="Young's modulus"}
E = 193 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Yield strength (for example ASTM A240 type 304, minimum 205 MPa)"}
S_y = 205 MPa_
```

```calc {description="Design factor the factor of safety on yield must reach"}
n_d = 1.5_
```

### Membrane stresses

```calc
> Net pressure
q = p_i - p_o = ?? MPa
> Mid-surface radius
R = D_i / 2 + t / 2 = ?? mm
> Slenderness; the membrane formulas need R/t > 10
R_t = R / t = ??
> Hoop stress, Table 13.1 cases 1b and 1c
σ_2 = q R / t = ?? MPa
> Longitudinal stress, case 1c (zero with open ends, case 1b)
σ_1 = {q R / (2 t) if capped == 1; 0 'MPa' if capped == 0} = ?? MPa
```

```calc
> Von Mises stress (the radial stress neglected)
σ_vm = sqrt(σ_1^2 - σ_1 σ_2 + σ_2^2) = ?? MPa
> Largest shear stress, half the largest difference of σ1, σ2 and 0
τ_max = (max(σ_1, σ_2, 0 'MPa') - min(σ_1, σ_2, 0 'MPa')) / 2 = ?? MPa
> Factor of safety on yield, distortion energy
n_y = S_y / σ_vm = ??
```

### Growth

The change in the mid-surface diameter and in the length l. A capped cylinder grows in length when ν < 0.5; an open one shortens.

```calc
> Change in mid-surface diameter, Table 13.1 cases 1b and 1c
ΔD = {2 q R^2 / (E t) (1 - ν / 2) if capped == 1; 2 q R^2 / (E t) if capped == 0} = ?? mm
> Change in length l
Δl = {q R l / (E t) (0.5 - ν) if capped == 1; -q R ν l / (E t) if capped == 0} = ?? mm
```

### Checks

```calc
@test R_t > 10
```

```calc
@test n_y ≥ n_d
```
