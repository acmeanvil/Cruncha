+++
cruncha-template = 1
id = "mechanical.vdi2230.preloadChecks"
version = "1.0.0"
title = "Bolted joint preload, torque and working checks, VDI 2230 (R2, R5 to R13)"
summary = "For one metric bolt by VDI 2230 Part 1: the clamp load needed for friction grip, sealing and opening, the assembly preload needed against the permissible preload of Eq. 144, the tightening torque, and the checks of the working stress, alternating stress, surface pressure under the head, slipping and shear."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt", "bolted joint", "VDI 2230", "preload", "assembly preload", "permissible preload", "tightening torque", "tightening factor", "working stress", "fatigue", "endurance limit", "surface pressure", "slip", "friction grip", "opening", "R2", "R5", "R7", "R8", "R9", "R10", "R12", "R13"]
from = "lacewing:mechanical.vdi2230"
license = "GPL-3.0-or-later"
inputs = ["d", "P", "R_p02", "ν", "μ_G", "μ_K", "d_W", "d_ha", "α_A", "Φ", "F_Z", "F_Amax", "F_Amin", "F_Q", "M_Y", "r_a", "q_F", "μ_T", "F_KP", "a", "s_sym", "u", "A_D", "I_BT", "N_Z", "c_p", "p_G", "τ_B", "S_Gmin"]
outputs = ["d_2", "d_S", "A_S", "F_KQerf", "F_KA", "F_Kerf", "F_Mmin", "F_Mmax", "σ_Mzul", "F_Mzul", "M_A", "F_Smax", "σ_z", "τ", "σ_redB", "S_F", "σ_a", "σ_AS", "S_D", "A_pmin", "p_Mmax", "p_Bmax", "p_max", "F_KRmin", "S_G", "τ_Q"]

[[references]]
work = "VDI 2230 Part 1, Systematic calculation of highly stressed bolted joints: joints with one cylindrical bolt"
edition = "2015"
locator = "Calculation steps R2 (Eqs. R2/1, R2/3, R2/4), R5 to R7, R8 (Eqs. R8/1 to R8/4), R9 (Eqs. R9/1, R9/5, R9/6), R10 (Eqs. R10/1 to R10/3), R12 (Eq. R12/1), R13 (Eq. R13/1); Eqs. 127, 143 to 145; Tables A1, A5, A8, A9; Examples B1, B2 and B5"

[[references]]
work = "ISO 898-1, Mechanical properties of fasteners made of carbon steel and alloy steel: bolts, screws and studs"
edition = "2013"
locator = "Minimum 0.2 % proof stress by property class"

[[tests]]
name = "VDI 2230 Example B1: M12 10.9 piston rod joint, 24.9 kN dynamic, concentric"
source = "Lacewing Lab vdi2230_test, after VDI 2230 Part 1:2015 Example B1: FKerf 1 kN (sealing), Φ 0.056, FZ 2415 N, αA 1.8: FMmin 26 921 N, FMmax 48 458 N; FMzul 64.8 kN (Table A1); R8 σz 785.2, τ 269.3, σred,B 819 N/mm²; R9 σa 8.3, σASV 48.9 N/mm²; R10 64.8 kN on Ap,min 90 mm² (dW 17.23 mm, dha 13.5 mm) is 720 N/mm². pG 900 MPa assumed."
inputs = { d = "12 mm", P = "1.75 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "17.23 mm", d_ha = "13.5 mm", "α_A" = "1.8", "Φ" = "0.056", F_Z = "2415 N", F_Amax = "24.9 kN", F_Amin = "0 kN", F_Q = "0 kN", F_KP = "1 kN", p_G = "900 MPa" }
expect = { F_Kerf = "1000 N", F_Mmin = "26921 N", F_Mmax = "48458 N", F_Mzul = "64.8 kN", "σ_z" = "785.2 MPa", "τ" = "269.3 MPa", "σ_redB" = "819 MPa", "σ_a" = "8.3 MPa", "σ_AS" = "48.9 MPa", p_Mmax = "720 MPa" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "VDI 2230 Example B2: M16 10.9 flange coupling, 8.4 kN transverse"
source = "Lacewing Lab vdi2230_test, after VDI 2230 Part 1:2015 Example B2: FKQerf = 8400/0.15 = 56 000 N, FZ 3630 N, αA 1.6: FMmax 95 408 N (no axial load, so Φ has no effect); FMzul 118.8 kN (Table A1, μG 0.12); R12 FKRmin = 118 800/1.6 − 3630 = 70 620 N, SG 1.26, τQ 41.8 N/mm² against τB 620 N/mm²; R10 118.8 kN on the example's Ap,min of 151.5 mm² (here dha 17.7 mm gives that area) is 784 N/mm². pG 900 MPa assumed."
inputs = { d = "16 mm", P = "2 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.12", "μ_K" = "0.12", d_W = "22.5 mm", d_ha = "17.7 mm", "α_A" = "1.6", "Φ" = "0.1", F_Z = "3630 N", F_Amax = "0 kN", F_Amin = "0 kN", F_Q = "8.4 kN", q_F = "1", "μ_T" = "0.15", F_KP = "0 kN", p_G = "900 MPa", "τ_B" = "620 MPa" }
expect = { F_KQerf = "56000 N", F_Mmax = "95408 N", F_Mzul = "118.8 kN", F_KRmin = "70620 N", S_G = "1.26", "τ_Q" = "41.8 MPa", p_Mmax = "784 MPa" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "VDI 2230 Example B5: M20 10.9 cylinder cap, eccentric, opening at one side"
source = "Lacewing Lab vdi2230_test, after VDI 2230 Part 1:2015 Example B5: FA 20.53 kN at a 24 mm, ssym −1.7 mm, u 13.8 mm, AD 843 mm², IBT 109 875 mm⁴: FKA 68.13 kN (R2/3); FMzul 190 kN (Table A1, μG 0.1); R8 with Φ 0.057: σz 780.3, τ 257.3, σred,B 811.5 N/mm²; σASV 44.6 N/mm². αA 1.8 and pG 900 MPa assumed; the example's alternating stress includes bending (Eq. 188), which this template leaves out, so σa is not compared."
inputs = { d = "20 mm", P = "2.5 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "28.87 mm", d_ha = "22 mm", "α_A" = "1.8", "Φ" = "0.057", F_Z = "5703 N", F_Amax = "20.53 kN", F_Amin = "6.16 kN", F_Q = "0 kN", F_KP = "0 kN", a = "24 mm", s_sym = "-1.7 mm", u = "13.8 mm", A_D = "843 mm²", I_BT = "109875 mm^4", p_G = "900 MPa" }
expect = { F_KA = "68.13 kN", F_Kerf = "68.13 kN", F_Mzul = "190 kN", "σ_z" = "780.3 MPa", "τ" = "257.3 MPa", "σ_redB" = "811.5 MPa", "σ_AS" = "44.6 MPa" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Table A1: M12 10.9 at μ = 0.10, 64.8 kN and 108 N·m"
source = "Lacewing Lab vdi2230_test, VDI 2230 Part 1:2015 Table A1 by Eq. 144 and Eq. 127 with DKm = (dW + dh)/2 = (17.23 + 13.5)/2 mm: FMzul 64.8 kN, MA 108 N·m (Table A1 rounds its torques)"
inputs = { d = "12 mm", P = "1.75 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "17.23 mm", d_ha = "13.5 mm" }
expect = { F_Mzul = "64.8 kN", M_A = "108 N·m" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Table A1: M16 10.9 at μ = 0.12, 118.8 kN and 302 N·m"
source = "Lacewing Lab vdi2230_test, VDI 2230 Part 1:2015 Table A1 by Eq. 144 and Eq. 127 with DKm = (22.5 + 17.5)/2 mm: FMzul 118.8 kN, MA 302 N·m"
inputs = { d = "16 mm", P = "2 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.12", "μ_K" = "0.12", d_W = "22.5 mm", d_ha = "17.5 mm" }
expect = { F_Mzul = "118.8 kN", M_A = "302 N·m" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Table A1: M12 12.9 at μ = 0.10, 75.9 kN"
source = "Lacewing Lab vdi2230_test, VDI 2230 Part 1:2015 Table A1 by Eq. 144: Rp0.2min 1100 MPa (ISO 898-1, 12.9), FMzul 75.9 kN"
inputs = { d = "12 mm", P = "1.75 mm", R_p02 = "1100 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "17.23 mm", d_ha = "13.5 mm" }
expect = { F_Mzul = "75.9 kN" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Example B1 exactly"
source = "By hand: d2 = 10.863342, dS = 12 − 0.938194 × 1.75 = 10.358161 mm, AS = 84.26654 mm²; k = 1.5 × 10.863342/10.358161 × (1.75/(π × 10.863342) + 0.1155) = 0.2652; σMzul = 0.9 × 940/√(1 + 3k²) = 770.203 MPa, FMzul = 64 902.3 N; FSmax = 64 902.3 + 0.056 × 24 900 = 66 296.7 N; σa = 0.056 × 24 900/(2 × 84.26654) = 8.27375 MPa; Ap,min = π/4 (17.23² − 13.5²) = 90.0246 mm², pM = 720.940 MPa, pB = (64 902.3 − 2415 + 1394.4)/90.0246 = 709.603 MPa; FKRmin = 64 902.3/1.8 − 0.944 × 24 900 − 2415 = 10 136.3 N"
inputs = { d = "12 mm", P = "1.75 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "17.23 mm", d_ha = "13.5 mm", "α_A" = "1.8", "Φ" = "0.056", F_Z = "2415 N", F_Amax = "24.9 kN", F_Amin = "0 kN", F_Q = "0 kN", F_KP = "1 kN", p_G = "900 MPa" }
expect = { d_S = "10.358161 mm", A_S = "84.26654 mm²", "σ_Mzul" = "770.2028 MPa", F_Mzul = "64902.32 N", F_Smax = "66296.72 N", "σ_a" = "8.273747 MPa", A_pmin = "90.02462 mm²", p_Mmax = "720.9397 MPa", p_Bmax = "709.6028 MPa", F_KRmin = "10136.25 N" }
tolerance = "2e-6"
checks = "pass"

[[tests]]
name = "Example B1 at 2 × 10⁵ cycles: finite-life fatigue strength"
source = "By hand: σASV = 0.85 (150/12 + 45) = 48.875 MPa, raised for N = 2·10⁵ < 2·10⁶ by (2·10⁶/2·10⁵)^(1/3) = 2.15443 (Eq. R9/6, rolled before heat treatment): σAS = 105.298 MPa"
inputs = { d = "12 mm", P = "1.75 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "17.23 mm", d_ha = "13.5 mm", "α_A" = "1.8", "Φ" = "0.056", F_Z = "2415 N", F_Amax = "24.9 kN", F_Amin = "0 kN", F_Q = "0 kN", F_KP = "1 kN", N_Z = "200000", p_G = "900 MPa" }
expect = { "σ_AS" = "105.298 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Example B1 tightened by impact wrench: the bolt is too small"
source = "Example B1 with αA 2.5 (impact wrench, VDI 2230 Table A8's upper range): FMmax = 2.5 × 26 921 = 67 302 N exceeds FMzul 64.9 kN (R7)"
inputs = { d = "12 mm", P = "1.75 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.10", "μ_K" = "0.10", d_W = "17.23 mm", d_ha = "13.5 mm", "α_A" = "2.5", "Φ" = "0.056", F_Z = "2415 N", F_Amax = "24.9 kN", F_Amin = "0 kN", F_Q = "0 kN", F_KP = "1 kN", p_G = "900 MPa" }
expect = { F_Mmax = "67301.5 N" }
tolerance = "1e-5"
checks = "fail"

[[tests]]
name = "Example B2 with a smaller safety against slipping: under the 1.8 asked for"
source = "Example B2 with a required slipping safety of 1.8 (the guideline's value for a dynamic transverse load): SG 1.26 < 1.8 (R12)"
inputs = { d = "16 mm", P = "2 mm", R_p02 = "940 MPa", "ν" = "0.9", "μ_G" = "0.12", "μ_K" = "0.12", d_W = "22.5 mm", d_ha = "17.7 mm", "α_A" = "1.6", "Φ" = "0.1", F_Z = "3630 N", F_Amax = "0 kN", F_Amin = "0 kN", F_Q = "8.4 kN", q_F = "1", "μ_T" = "0.15", F_KP = "0 kN", p_G = "900 MPa", "τ_B" = "620 MPa", S_Gmin = "1.8" }
expect = { S_G = "1.26" }
tolerance = "0.4%"
checks = "fail"
+++

## Bolted joint preload, torque and working checks (VDI 2230)

One cylindrical bolt with an ISO metric thread by VDI 2230 Part 1:2015, from the clamp load the joint needs to the checks of the bolt in service: calculation steps R2 and R5 to R13. The load factor Φ and the embedding loss F_Z come from step R3 and R4; the VDI 2230 resilience and load factor template works them out, and its outputs bind to the inputs of the same name here.

The bolt has a plain shank at the nominal diameter (no waisted or hollow shank), and its smallest section is the thread's stress section. Not checked here: the size estimate of Table A7 (R0), thermal changes of preload, the bending stress of an eccentrically loaded bolt (Eq. 188, which raises the alternating stress of R9), the length of thread engagement (R11), and joints of several bolts (VDI 2230 Part 2).

### Bolt and tightening

The property class gives the minimum 0.2 % proof stress (ISO 898-1: 640 MPa for 8.8 up to M16 and 660 MPa above, 940 MPa for 10.9, 1100 MPa for 12.9). The permissible preload uses the share ν of it, 0.9 in VDI 2230, under the tension and the torsion of tightening together. Use the smallest friction expected (VDI 2230 Table A5) for the permissible preload and the torque.

The tightening factor α_A = F_Mmax/F_Mmin is the scatter of the preload for the tightening method; VDI 2230 Table A8 gives guide ranges, and the upper value is the safe one for dimensioning. For yield-point or angle-controlled tightening take α_A = 1 and c_p = 1.4 below.

```calc {description="Nominal thread diameter"}
d = 12 mm_
```

```calc {description="Thread pitch"}
P = 1.75 mm_
```

```calc {description="Minimum 0.2 % proof stress of the bolt (ISO 898-1 by property class)"}
R_p02 = 940 MPa_
```

```calc {description="Share of the proof stress the assembly preload may use (0.9 in VDI 2230)"}
ν = 0.9_
```

```calc {description="Thread friction coefficient, smallest expected (Table A5)"}
μ_G = 0.10_
```

```calc {description="Head friction coefficient, smallest expected (Table A5)"}
μ_K = 0.10_
```

```calc {description="Outside diameter of the head's bearing face"}
d_W = 17.23 mm_
```

```calc {description="Inner diameter of the bearing face: the hole, or its chamfer if larger"}
d_ha = 13.5 mm_
```

```calc {description="Tightening factor α_A = F_Mmax / F_Mmin (Table A8)"}
α_A = 1.8_
```

### Joint and loads

```calc {description="Load factor, from step R3"}
Φ = 0.056_
```

```calc {description="Preload lost to embedding, from step R4"}
F_Z = 2415 N_
```

```calc {description="Axial working load on the joint, largest"}
F_Amax = 24.9 kN_
```

```calc {description="Axial working load on the joint, smallest"}
F_Amin = 0 kN_
```

```calc {description="Transverse load across the interfaces, largest"}
F_Q = 0 kN_
```

```calc {description="Torque about the bolt axis carried by friction"}
M_Y = 0 N·m_
```

```calc {description="Friction radius of the interface for M_Y"}
r_a = 20 mm_
```

```calc {description="Number of interfaces that transmit the transverse load by friction"}
q_F = 1_
```

```calc {description="Friction coefficient at the interfaces, smallest expected"}
μ_T = 0.15_
```

```calc {description="Residual clamp load required, for sealing or otherwise"}
F_KP = 1 kN_
```

### Opening at one side (eccentric loading)

For an axial load off the symmetry axis of the interface, the joint must not open at its edge. a is the load's distance from the symmetry axis, s_sym the bolt's, and u the distance to the edge at risk of opening, with the signs of VDI 2230 Table 4; A_D is the interface (sealing) area and I_BT its second moment of area about the symmetry axis. With a = s_sym = 0 (concentric) no clamp load is needed against opening.

```calc {description="Line of action of the axial load from the symmetry axis"}
a = 0 mm_
```

```calc {description="Bolt axis from the symmetry axis (Table 4 sign)"}
s_sym = 0 mm_
```

```calc {description="Edge at risk of opening from the symmetry axis (Table 4 sign)"}
u = 20 mm_
```

```calc {description="Interface (sealing) area"}
A_D = 1000 mm²_
```

```calc {description="Second moment of area of the interface about the symmetry axis"}
I_BT = 100000 mm^4_
```

### Required clamp load (R2)

```calc
> Clamp load for friction grip against transverse load and torque, Eq. R2/1
F_KQerf = F_Q / (q_F μ_T) + M_Y / (q_F r_a μ_T) = ?? N
> Clamp load against opening at one side, Eq. R2/3
F_KA = max(0 'N', F_Amax A_D (a u - s_sym u) / (I_BT + s_sym u A_D)) = ?? N
> Required clamp load, Eq. R2/4
F_Kerf = max(F_KQerf, F_KP + F_KA) = ?? N
```

### Assembly preload (R5 to R7)

The least preload must hold the required clamp load after the bolt has taken its share of the axial load and the joint has settled; the tightening scatter then sets the largest preload that the assembly may give.

```calc
> Minimum assembly preload, Eq. R5/1 (no thermal change)
F_Mmin = F_Kerf + (1 - Φ) F_Amax + F_Z = ?? N
> Maximum assembly preload, Eq. R6/1
F_Mmax = α_A F_Mmin = ?? N
```

The permissible preload is the tension at which tension and the torsion from thread friction during tightening reach ν R_p0.2 together (von Mises), with the plastic reserve in torsion (the 3/2 of Eq. 144), on the stress section of the thread.

```calc
> Pitch diameter (DIN 13)
d_2 = d - 0.649519 P = ?? mm
> Stress diameter (d₂ + d₃)/2 (DIN 13)
d_S = d - 0.938194 P = ?? mm
> Tensile stress area
A_S = π d_S^2 / 4 = ?? mm²
> Permissible assembly stress, Eq. 144
σ_Mzul = ν R_p02 / sqrt(1 + 3 (3 / 2 × d_2 / d_S (P / (π d_2) + 1.155 μ_G))^2) = ?? MPa
> Permissible assembly preload, Eq. 143
F_Mzul = σ_Mzul A_S = ?? kN
```

```calc
@test F_Mzul ≥ F_Mmax
```

If this fails, choose a larger diameter or property class, or a tightening method with a smaller α_A (R7).

### Tightening torque (R13)

```calc {description="Tightening torque for F_Mzul at the smallest friction, Eq. 127 (R13/1)"}
M_A = F_Mzul (0.16 P + 0.58 d_2 μ_G + (d_W + d_ha) / 4 × μ_K) = ?? N·m
```

### Working stress (R8)

In service the bolt carries the permissible preload plus its share of the axial load. Of the torsion from tightening, only half is taken to remain (k_τ = 0.5).

```calc
> Largest bolt force, Eq. R8/1
F_Smax = F_Mzul + Φ F_Amax = ?? N
> Tensile stress, Eq. R8/2
σ_z = F_Smax / A_S = ?? MPa
> Thread moment of tightening
M_G = F_Mzul d_2 / 2 × (P / (π d_2) + 1.155 μ_G) = ?? N·m
> Torsional stress, Eq. R8/3
τ = M_G / (π d_S^3 / 16) = ?? MPa
> Reduced (von Mises) working stress with k_τ = 0.5, Eq. R8/4
σ_redB = sqrt(σ_z^2 + 3 (0.5 τ)^2) = ?? MPa
> Safety against exceeding the proof stress
S_F = R_p02 / σ_redB = ??
```

```calc
@test S_F ≥ 1
```

### Alternating stress (R9)

For a concentric joint the stress amplitude in the thread is the bolt's share of the load range over twice the stress area. The endurance limit is for threads rolled before heat treatment (the usual case); below 2·10⁶ cycles it is raised for finite life (Eq. R9/6, valid from 10⁴ cycles). The guideline asks for a safety of at least 1.2. With no load range (F_Amax = F_Amin) there is nothing to check, and S_D says so.

```calc {description="Number of alternating cycles (2·10⁶ or more: endurance limit)"}
N_Z = 2000000_
```

```calc
> Stress amplitude in the thread, concentric, Eq. R9/1
σ_a = Φ abs(F_Amax - F_Amin) / (2 A_S) = ?? MPa
> Endurance limit of threads rolled before heat treatment, Eq. R9/5 (d in mm)
σ_ASV = 0.85 (150 'mm' / d + 45) × 1 'MPa' = ?? MPa
> Fatigue strength at N_Z cycles, Eq. R9/6
σ_AS = { σ_ASV (2000000 / N_Z)^(1 / 3) if N_Z < 2000000; σ_ASV otherwise } = ?? MPa
> Safety against fatigue failure
S_D = { σ_AS / σ_a if σ_a > 0 'MPa'; "static: no alternating load" otherwise } = ??
```

```calc
@test 1.2 σ_a ≤ σ_AS
```

```calc
@test N_Z ≥ 10000
```

### Surface pressure under the head (R10)

The bearing face is the annulus from the hole (or its chamfer) to d_W. Compare the larger of the assembly and working pressures with the limiting surface pressure of the clamped part's material (VDI 2230 Table A9, or the material's own data). If it is over, use a washer or a larger bearing face.

```calc {description="Assembly pressure factor: 1, or 1.4 for yield-point or angle-controlled tightening (R10/3)"}
c_p = 1_
```

```calc {description="Limiting surface pressure of the clamped part (Table A9)"}
p_G = 900 MPa_
```

```calc
> Smallest bearing area under the head
A_pmin = π / 4 × (d_W^2 - d_ha^2) = ?? mm²
> Surface pressure at assembly, Eqs. R10/1 and R10/3
p_Mmax = c_p F_Mzul / A_pmin = ?? MPa
> Surface pressure in service, Eq. R10/2
p_Bmax = (F_Mzul - F_Z + Φ F_Amax) / A_pmin = ?? MPa
> Governing surface pressure
p_max = max(p_Mmax, p_Bmax) = ?? MPa
```

```calc
@test p_max ≤ p_G
```

### Slipping and shear (R12)

The least residual clamp load in service must carry the transverse load by friction with a margin: VDI 2230 recommends at least 1.2 for static and 1.8 for alternating transverse loads. If the joint slips, the bolt is sheared; the bolt's shear strength is (τ_B/R_m)·R_m, the ratio by property class from VDI 2230 Table 7 (620 MPa for 10.9). With no transverse load or torque, S_G says so.

```calc {description="Required safety against slipping (1.2 static, 1.8 alternating)"}
S_Gmin = 1.2_
```

```calc {description="Shear strength of the bolt"}
τ_B = 620 MPa_
```

```calc
> Least residual clamp load at the interface, Eq. R12/1
F_KRmin = F_Mzul / α_A - (1 - Φ) F_Amax - F_Z = ?? N
> Safety against slipping
S_G = { F_KRmin / F_KQerf if F_KQerf > 0 'N'; "no transverse load" otherwise } = ??
> Shear stress in the bolt if the joint slips
τ_Q = F_Q / (π d^2 / 4) = ?? MPa
```

```calc
@test F_KRmin ≥ S_Gmin F_KQerf
```

```calc
@test τ_Q ≤ τ_B
```
