+++
cruncha-template = 1
id = "mechanical.strainGauges.bridge"
version = "1.0.0"
title = "Strain gauge bridge: strain from the output, TN-507 arrangements"
summary = "The strain under the main gauge of a quarter, half or full Wheatstone bridge from its output, for the seven arrangements of Micro-Measurements TN-507 Table 1, corrected for bridge nonlinearity; the error a linear reading makes, and the uniaxial stress."
category = ["Mechanical engineering", "Strain measurement"]
tags = ["strain gauge", "Wheatstone bridge", "quarter bridge", "half bridge", "full bridge", "bridge nonlinearity", "gauge factor", "Poisson gauge", "indicated strain", "TN-507", "experimental stress analysis"]
from = "lacewing:mechanical.strainGauges"
license = "GPL-3.0-or-later"
inputs = ["k", "F", "ν", "V_ex", "V_o", "E"]
outputs = ["r", "ε_i", "N", "ε", "ε_lin", "η", "σ"]

[[references]]
work = "Micro-Measurements (Vishay Precision Group) Tech Note TN-507-1, Errors Due to Wheatstone Bridge Nonlinearity"
edition = "Rev. 19-Nov-2010"
locator = "Table 1: the bridge arrangements (cases 1 to 7), their outputs and the correction from indicated strain"

[[tests]]
name = "TN-507 case 1, quarter bridge: 5000 µε indicated"
source = "Micro-Measurements TN-507-1 Table 1, case 1, as in Lacewing Lab strain_gauge_test (F = 2.0): ε = 2εi/(2 − Fεi) = 0.01/1.99 = 5025.126 µε; the linear reading is 0.5 % low. The output is Fεi/4 = 2.5 mV/V, 12.5 mV at 5 V"
inputs = { k = "1", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV" }
expect = { r = "2.5 mV/V", "ε_i" = "5000 µm/m", "ε" = "5025.125628 µm/m", "η" = "-0.005" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "TN-507 case 2, half bridge with a Poisson gauge: 5000 µε indicated"
source = "Micro-Measurements TN-507-1 Table 1, case 2, as in Lacewing Lab strain_gauge_test (F = 2.0, ν = 0.3): ε = 2εi/(2(1 + ν) − Fεi(1 − ν)) = 0.01/2.593 = 3856.537 µε"
inputs = { k = "2", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV" }
expect = { "ε" = "3856.53683 µm/m", N = "1.3" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "TN-507 case 3, half bridge of equal and opposite strains: 5000 µε indicated"
source = "Micro-Measurements TN-507-1 Table 1, case 3, as in Lacewing Lab strain_gauge_test: ε = εi/2 = 2500 µε, with no nonlinearity"
inputs = { k = "3", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV" }
expect = { "ε" = "2500 µm/m", "η" = "0" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "TN-507 case 4, half bridge of equal strains in opposite arms: 5000 µε indicated"
source = "Micro-Measurements TN-507-1 Table 1, case 4, as in Lacewing Lab strain_gauge_test (F = 2.0): ε = 2εi/(4 − Fεi) = 0.01/3.99 = 2506.266 µε"
inputs = { k = "4", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV" }
expect = { "ε" = "2506.265664 µm/m", "η" = "-0.0025" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "TN-507 case 5, full bridge on a column: −3000 µε indicated"
source = "Micro-Measurements TN-507-1 Table 1, case 5, as in Lacewing Lab strain_gauge_test (F = 2.0, ν = 0.3): ε = 2εi/(4(1 + ν) − Fεi(1 − ν)) = −0.006/5.2042 = −1152.915 µε. The output is Fεi/4 = −1.5 mV/V, −7.5 mV at 5 V"
inputs = { k = "5", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "-7.5 mV" }
expect = { "ε" = "-1152.914953 µm/m", "η" = "0.00080769231" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "TN-507 case 6, full bridge on a beam: 5000 µε indicated"
source = "Micro-Measurements TN-507-1 Table 1, case 6, as in Lacewing Lab strain_gauge_test (ν = 0.3): ε = εi/2(1 + ν) = 1923.077 µε"
inputs = { k = "6", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV" }
expect = { "ε" = "1923.076923 µm/m" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "TN-507 case 7, full bridge of equal and opposite pairs: 5000 µε indicated, on steel"
source = "Micro-Measurements TN-507-1 Table 1, case 7, as in Lacewing Lab strain_gauge_test: ε = εi/4 = 1250 µε; by hand σ = 200 GPa × 1250 µε = 250 MPa"
inputs = { k = "7", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV", E = "200 GPa" }
expect = { "ε" = "1250 µm/m", "σ" = "250 MPa" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "TN-507: one active gauge at 10 000 µε reads about 1 % low"
source = "Micro-Measurements TN-507-1, as in Lacewing Lab strain_gauge_test: the nonlinearity of a quarter bridge is about the strain in per cent. By hand: at ε = 0.01, F = 2, Eo/E = Fε/(4 + 2Fε) = 0.02/4.04 = 4.950495 mV/V (24.752475 mV at 5 V), εi = 9900.990 µε, 0.990 % low (η = −1/101)"
inputs = { k = "1", F = "2.0", V_ex = "5 V", V_o = "24.75247525 mV" }
expect = { "ε" = "10000 µm/m", "η" = "-0.00990099" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "No such arrangement: case 8"
source = "TN-507-1 Table 1 has cases 1 to 7 only"
inputs = { k = "8", F = "2.0", "ν" = "0.3", V_ex = "5 V", V_o = "12.5 mV" }
expect = { "ε_i" = "5000 µm/m" }
tolerance = "1e-9"
checks = "fail"
+++

## Strain gauge bridge: strain from the output

The strain under the main gauge of a Wheatstone bridge of strain gauges, from the bridge output, for the seven arrangements of Micro-Measurements Tech Note TN-507 Table 1. Every bridge is slightly nonlinear unless its active arms change by equal and opposite amounts; the correction here is TN-507's exact one, so a large strain (a yielding part, a plastic or elastomer specimen) comes out right.

The arms are a and b in series on one side of the bridge, d and c on the other, with the output taken between the two midpoints: Eo/E = a/(a + b) − d/(d + c). Strain in a or c raises the output; strain in b or d lowers it. The arrangements, by TN-507 case:

1. Quarter bridge: one active gauge (a).
2. Half bridge: an axial gauge (a) and a Poisson gauge across it (b), reading −νε.
3. Half bridge: equal and opposite strains in adjacent arms (a, b): the top and bottom of a beam in bending, or a ±45° pair on a shaft in torsion.
4. Half bridge: equal strains in opposite arms (a, c): two sides of a column, so that bending cancels.
5. Full bridge: two axial gauges (a, c) and two Poisson gauges (b, d) on a column.
6. Full bridge: two axial gauges (a, b) top and bottom of a beam in bending, and two Poisson gauges (c, d).
7. Full bridge: two pairs of equal and opposite strains (all four arms): a beam in bending, or a shaft in torsion.

Assumptions: equal gauge resistances, completion resistors equal to them, no lead resistance and no thermal output (the quarter bridge template takes both), and gauges of one gauge factor. ε is the strain along the main gauge's grid (arm a).

### Bridge

```calc {description="Bridge arrangement: the case number of TN-507 Table 1, 1 to 7"}
k = 1_
```

```calc
@test k == round(k) and 1 ≤ k ≤ 7
```

```calc {description="Gauge factor of the gauges, from the gauge package"}
F = 2.0_
```

```calc {description="Poisson's ratio of the part, for the Poisson gauges of cases 2, 5 and 6"}
ν = 0.3_
```

### Reading

```calc {description="Bridge excitation voltage"}
V_ex = 5 V_
```

```calc {description="Bridge output under load, taken from the zero (unloaded) output"}
V_o = 12.5 mV_
```

A strain indicator shows εi, the strain a linear quarter bridge would give at its gauge factor setting: Eo/E = F·εi/4. With the indicator set to the gauges' F, enter V_o = F·εi·V_ex/4.

```calc
> Bridge output per volt of excitation
r = V_o / V_ex = ?? mV/V
> Indicated strain, the strain an indicator set to F shows
ε_i = 4 r / F = ?? µm/m
```

### Strain under the main gauge

```calc {description="Strain along the main gauge, TN-507 Table 1's correction for the arrangement"}
ε = { 2 ε_i / (2 - F ε_i) if k == 1; 2 ε_i / (2 (1 + ν) - F ε_i (1 - ν)) if k == 2; ε_i / 2 if k == 3; 2 ε_i / (4 - F ε_i) if k == 4; 2 ε_i / (4 (1 + ν) - F ε_i (1 - ν)) if k == 5; ε_i / (2 (1 + ν)) if k == 6; ε_i / 4 otherwise } = ?? µm/m
```

The bridge factor N is the output of the arrangement over a quarter bridge's at small strain. Dividing the indicated strain by it is the linear reading; η is that reading's error. Cases 3, 6 and 7 are linear; for the others the error is roughly proportional to the strain.

```calc {description="Bridge factor: small-strain output relative to one active gauge"}
N = { 1 if k == 1; 1 + ν if k == 2; 2 if k == 3; 2 if k == 4; 2 (1 + ν) if k == 5; 2 (1 + ν) if k == 6; 4 otherwise } = ??
```

```calc
> Linear reading, the indicated strain over the bridge factor
ε_lin = ε_i / N = ?? µm/m
> Error of the linear reading (negative: it reads low)
η = (ε_lin - ε) / ε = ??
```

### Stress

Where the main gauge lies along a uniaxial stress (an axial member, or the extreme fibre of a beam in bending) the stress there is E·ε. In a biaxial state use a rosette; for torque on a shaft, the shaft torque template.

```calc {description="Young's modulus of the part"}
E = 200 GPa_
```

```calc {description="Uniaxial stress along the main gauge"}
σ = E ε = ?? MPa
```
