+++
cruncha-template = 1
id = "mechanical.strainGauges.shaftTorque"
version = "1.0.0"
title = "Shaft torque from a ±45° strain gauge bridge"
summary = "Torque on a round shaft or tube from the output of a half or full bridge of gauges at ±45° to its axis (TN-507 cases 3 and 7): the strain, shear strain and shear stress at the surface, the torque, and the output and sensitivity a rated torque should give."
category = ["Mechanical engineering", "Strain measurement"]
tags = ["strain gauge", "torque", "torsion", "shaft", "torque transducer", "half bridge", "full bridge", "±45°", "shear strain", "Wheatstone bridge", "TN-507"]
from = "lacewing:mechanical.strainGauges"
license = "GPL-3.0-or-later"
inputs = ["k", "F", "E", "ν", "D", "d_i", "V_ex", "V_o", "T_r"]
outputs = ["N", "ε", "γ", "G", "τ", "Z_p", "T", "V_r", "S_T"]

[[references]]
work = "Micro-Measurements (Vishay Precision Group) Tech Note TN-507-1, Errors Due to Wheatstone Bridge Nonlinearity"
edition = "Rev. 19-Nov-2010"
locator = "Table 1, case 3 (half bridge, equal and opposite strains) and case 7 (full bridge, shaft in torsion): outputs Fε/2 and Fε, no nonlinearity"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-12: torsion of a round bar, τ = Tr/J, J = π(d⁴ − dᵢ⁴)/32; Sec. 3-6, Mohr's circle for pure shear (principal directions at 45°)"

[[tests]]
name = "Lacewing: a full bridge on a shaft of J/r = 3 cm³, G = 77 GPa, reads 1000 N·m"
source = "Lacewing Lab strain_gauge_test (load and strain are each other's inverse; torsion at ±45° reads half the shear strain): 1000 N·m on J/r = 3e-6 m³ and G = 77 GPa gives ε = T/(2G·J/r) = 2164.502 µε. Here G = 77 GPa from E = 200.2 GPa, ν = 0.3, and J/r = 3e-6 m³ from a solid D = (48e-6/π)^⅓ m = 24.81402 mm; a full bridge at F = 2 then reads Fε = 4.329004 mV/V, 21.64502 mV at 5 V"
inputs = { k = "7", F = "2.0", E = "200.2 GPa", "ν" = "0.3", D = "24.81401964 mm", d_i = "0 mm", V_ex = "5 V", V_o = "21.64502165 mV", T_r = "1000 N·m" }
expect = { Z_p = "3 cm³", G = "77 GPa", "ε" = "2164.502165 µm/m", T = "1000 N·m", V_r = "21.64502165 mV" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "Hand check: a tube, 25 × 19 mm, in a half bridge"
source = "By hand: J/r = π(25⁴ − 19⁴)/(16 × 25) = 2044.4228 mm³; G = 200/2.6 = 76.923 GPa; 1.2 mV at 10 V is 0.12 mV/V, so ε = 2 × 0.12e-3/2.1 = 114.2857 µε (case 3: Eo/E = Fε/2); γ = 2ε, τ = Gγ = 17.58242 MPa; T = τ·J/r = 35.9459 N·m. For 150 N·m: ε = 150/(2G·J/r) = 476.9072 µε, Eo/E = 2.1 × 476.9072e-6/2 = 0.500753 mV/V, 5.00753 mV, 3.33835 µV/V per N·m"
inputs = { k = "3", F = "2.1", E = "200 GPa", "ν" = "0.3", D = "25 mm", d_i = "19 mm", V_ex = "10 V", V_o = "1.2 mV", T_r = "150 N·m" }
expect = { N = "2", Z_p = "2044.422835 mm³", "ε" = "114.2857143 µm/m", "γ" = "228.5714286 µm/m", "τ" = "17.58241758 MPa", T = "35.945896 N·m", V_r = "5.007525754 mV", S_T = "3.338350503 µV/V/N/m" }
tolerance = "1e-8"
checks = "pass"

[[tests]]
name = "A full bridge on a column (case 5) does not read torque"
source = "TN-507-1 Table 1: only the arrangements of equal and opposite strains, cases 3 and 7, read a ±45° pair in torsion (Lacewing Lab strain_gauge_test)"
inputs = { k = "5" }
expect = { G = "76.92307692 GPa" }
tolerance = "1e-8"
checks = "fail"
+++

## Shaft torque from a ±45° strain gauge bridge

A shaft in pure torsion has its principal strains at ±45° to its axis, equal and opposite: ε = γ/2. Gauges laid along those directions, one pair (half bridge) or two pairs on opposite sides (full bridge), read the torque. Their strains are equal and opposite in adjacent arms, so the bridge output is exactly linear (TN-507 Table 1, cases 3 and 7), and an axial load, a bending moment on a full bridge with its pairs 180° apart, and temperature changes cancel.

Assumptions: a round solid shaft or tube, elastic, with the gauges at the outside surface and well away from shoulders, keyways and the load's points of application; equal gauges of one gauge factor, completion resistors (half bridge) equal to them, no lead resistance (or remote sense). The shear modulus comes from E and ν for an isotropic material.

### Bridge

```calc {description="Bridge arrangement: TN-507 case 3 (half bridge, one ±45° pair) or 7 (full bridge, two pairs)"}
k = 7_
```

```calc
@test k == 3 or k == 7
```

```calc {description="Gauge factor of the gauges, from the gauge package"}
F = 2.0_
```

```calc {description="Bridge factor: active arms each reading ±ε"}
N = { 2 if k == 3; 4 otherwise } = ??
```

### Shaft

```calc {description="Young's modulus of the shaft"}
E = 200 GPa_
```

```calc {description="Poisson's ratio of the shaft"}
ν = 0.3_
```

```calc {description="Outside diameter at the gauges"}
D = 25 mm_
```

```calc {description="Bore (0 for a solid shaft)"}
d_i = 0 mm_
```

```calc
> Shear modulus, isotropic material
G = E / (2 (1 + ν)) = ?? GPa
> Polar section modulus J/r at the outside surface
Z_p = π (D^4 - d_i^4) / (16 D) = ?? mm³
```

### Reading

An indicator showing εi at a gauge factor setting Fᵢ has an output Eo/E = Fᵢ·εi/4: enter V_o = Fᵢ·εi·V_ex/4.

```calc {description="Bridge excitation voltage"}
V_ex = 5 V_
```

```calc {description="Bridge output under the torque, taken from the zero output"}
V_o = 2 mV_
```

```calc
> Strain along each gauge, Eo/E = F N ε / 4
ε = 4 V_o / (F N V_ex) = ?? µm/m
> Shear strain at the surface
γ = 2 ε = ?? µm/m
> Shear stress at the surface
τ = G γ = ?? MPa
```

```calc {description="Torque on the shaft, τ J/r"}
T = τ Z_p = ?? N·m
```

### Expected output

The output a rated or expected torque should give, for choosing the excitation and checking a calibration.

```calc {description="Rated or expected torque"}
T_r = 100 N·m_
```

```calc
> Strain along each gauge at the rated torque
ε_r = T_r / (2 G Z_p) = ?? µm/m
> Bridge output at the rated torque
V_r = F N ε_r / 4 × V_ex = ?? mV
> Sensitivity: output per volt of excitation per N·m
S_T = V_r / (V_ex T_r) = ?? µV/V/N/m
```
