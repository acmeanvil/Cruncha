+++
cruncha-template = 1
id = "mechanical.waveSprings.smalley"
version = "1.0.0"
title = "Wave spring, single turn, crest to crest or nested (Smalley)"
summary = "A flat-wire wave spring of one turn, several turns stacked crest to crest (in series) or several layers nested (in parallel): the multiple wave factor, deflection and rate under a load, the operating stress, the work height and the stress as a share of the strip's tensile strength, by Smalley's published formulas."
category = ["Mechanical engineering", "Springs"]
tags = ["wave spring", "flat wire", "crest to crest", "nested", "multi-turn", "spring rate", "deflection", "work height", "Smalley"]
from = "lacewing:mechanical.waveSprings"
license = "GPL-3.0-or-later"
inputs = ["mode", "D_o", "D_i", "t", "N", "Z", "E", "P", "H", "S_ut"]
outputs = ["b", "D_m", "K", "f", "k", "S", "H_w", "r_S"]

[[references]]
work = "Smalley Steel Ring Co., wave spring design formulas (smalley.com)"
edition = "online, retrieved 2025"
locator = "Single Turn, Multiple Turn (crest to crest) and Nested wave spring formulas; multiple wave factor K by waves per turn; operating stress limits"

[[tests]]
name = "Single turn: 50 × 40 mm ring, 0.5 mm strip, 3.5 waves, 100 N"
source = "Lacewing Lab coil_springs_test (wave springs), checked by hand: b = 5 mm, Dm = 45 mm, K = 3.88; f = 100 × 3.88 × 45³ / (200 000 × 5 × 0.5³ × 3.5⁴) × 40/50 = 1.50792 mm; S = 3π × 100 × 45 / (4 × 5 × 0.25 × 12.25) = 692.43 MPa"
inputs = { mode = "1", D_o = "50 mm", D_i = "40 mm", t = "0.5 mm", N = "3.5", Z = "1", E = "200 GPa", P = "100 N", H = "5 mm", S_ut = "1500 MPa" }
expect = { b = "5 mm", D_m = "45 mm", K = "3.88", f = "1.507916 mm", k = "66.3167 N/mm", S = "692.4327 MPa", H_w = "3.492084 mm", r_S = "0.461622" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Four turns crest to crest: four times the deflection at the same stress"
source = "Lacewing Lab coil_springs_test: crest to crest with Z = 4 deflects 4 × the single turn (4 × 1.50792 = 6.03166 mm); the stress is unchanged"
inputs = { mode = "2", D_o = "50 mm", D_i = "40 mm", t = "0.5 mm", N = "3.5", Z = "4", E = "200 GPa", P = "100 N", H = "20 mm", S_ut = "1500 MPa" }
expect = { f = "6.031663 mm", k = "16.57918 N/mm", S = "692.4327 MPa", H_w = "13.96834 mm" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Four layers nested: a quarter of the deflection and of the stress"
source = "Lacewing Lab coil_springs_test: nested with Z = 4 deflects a quarter of the single turn (0.376979 mm); by hand S = 692.43 / 4 = 173.108 MPa"
inputs = { mode = "3", D_o = "50 mm", D_i = "40 mm", t = "0.5 mm", N = "3.5", Z = "4", E = "200 GPa", P = "100 N", H = "5 mm", S_ut = "1500 MPa" }
expect = { f = "0.3769789 mm", k = "265.2668 N/mm", S = "173.1082 MPa", r_S = "0.1154054" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Inch units: 1 lbf on the same ring, 29 Mpsi"
source = "Lacewing Lab coil_springs_test, the formula worked in inches: 1 × 3.88 × 1.7717³ / (29e6 × 0.19685 × 0.019685³ × 3.5⁴) × 1.5748/1.9685 = 0.0026417 in"
inputs = { mode = "1", D_o = "1.9685 in", D_i = "1.5748 in", t = "0.019685 in", N = "3.5", Z = "1", E = "29 Mpsi", P = "1 lbf", H = "0.2 in", S_ut = "220 ksi" }
expect = { f = "0.0026417 in" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "More waves: K steps down with N"
source = "By hand: N = 5 gives K = 2.90; f = 100 × 2.90 × 45³ / (200 000 × 5 × 0.125 × 625) × 0.8 = 0.2706048 mm, S = 3π × 100 × 45 / (4 × 5 × 0.25 × 25) = 339.292 MPa"
inputs = { mode = "1", D_o = "50 mm", D_i = "40 mm", t = "0.5 mm", N = "5", Z = "1", E = "200 GPa", P = "100 N", H = "5 mm", S_ut = "1500 MPa" }
expect = { K = "2.90", f = "0.2706048 mm", S = "339.2920 MPa" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Overloaded single turn: 300 N, stress over the tensile strength"
source = "By hand: three times the first case, S = 2077.3 MPa, S/Sut = 1.385 over Smalley's static limit of 1"
inputs = { mode = "1", D_o = "50 mm", D_i = "40 mm", t = "0.5 mm", N = "3.5", Z = "1", E = "200 GPa", P = "300 N", H = "5 mm", S_ut = "1500 MPa" }
expect = { S = "2077.298 MPa", r_S = "1.384865" }
tolerance = "1e-5"
checks = "fail"
+++

## Wave spring (Smalley)

A wave spring is a ring of flat wire formed into waves round its circumference; under an axial load the waves flatten. It gives a compression spring's force in a much shorter height. The formulas are those Smalley publishes for its wave springs, good while the waves stay well clear of flat: Smalley keeps single-turn and nested springs within 30 to 70 % of their available travel, and multi-turn springs within 20 to 80 %. The solid height (about the number of turns or layers times t) is not checked here, so confirm that the work height stays above it.

### Arrangement

1. Single turn: one wavy ring.
2. Crest to crest: Z turns of one strip, wound so that crests meet troughs; they act in series, so the deflection is Z times a single turn's at the same stress.
3. Nested: Z layers wound together, waves in step; they act in parallel and share the load, so both the deflection and the stress are divided by Z.

```calc {description="Arrangement: 1 single turn, 2 crest to crest, 3 nested"}
mode = 1_
```

```calc
@test mode == round(mode) and 1 ≤ mode ≤ 3
```

### Spring

N is the number of waves round one turn; half waves are made (3.5, 4.5). Z is the number of turns (crest to crest) or layers (nested); it is ignored for a single turn. Spring steel strip has E of about 200 GPa (29 Mpsi).

```calc {description="Outside diameter of the ring"}
D_o = 50 mm_
```

```calc {description="Inside diameter of the ring"}
D_i = 40 mm_
```

```calc {description="Strip thickness"}
t = 0.5 mm_
```

```calc {description="Waves per turn"}
N = 3.5_
```

```calc {description="Turns (crest to crest) or layers (nested)"}
Z = 1_
```

```calc {description="Young's modulus of the strip"}
E = 200 GPa_
```

```calc {description="Free height of the spring"}
H = 5 mm_
```

```calc {description="Minimum tensile strength of the strip"}
S_ut = 1500 MPa_
```

### Load

```calc {description="Axial load"}
P = 100 N_
```

### Geometry and wave factor

Smalley's multiple wave factor K falls as the waves per turn rise: 3.88 to 4 waves, 2.90 to 6.5, 2.30 to 9.5 and 2.13 beyond (the steps are taken half-way between).

```calc
> Radial wall of the strip
b = (D_o - D_i) / 2 = ?? mm
> Mean diameter
D_m = (D_o + D_i) / 2 = ?? mm
> Multiple wave factor
K = { 3.88 if N < 4.25; 2.90 if N < 6.75; 2.30 if N < 9.75; 2.13 otherwise } = ??
```

### Deflection, rate and stress

The deflection of one turn is P K D_m³ / (E b t³ N⁴) × D_i/D_o, which goes as 1/t³ and 1/N⁴; the operating stress is 3π P D_m / (4 b t² N²).

```calc
> Deflection multiplier for the arrangement (Z in series, 1/Z in parallel)
Z_f = { 1 if mode == 1; Z if mode == 2; 1 / Z if mode == 3 } = ??
> Deflection under the load
f = P K D_m^3 / (E b t^3 N^4) (D_i / D_o) Z_f = ?? mm
> Rate
k = P / f = ?? N/mm
> Operating stress (shared by the layers when nested)
S = 3 π P D_m / (4 b t^2 N^2) / { Z if mode == 3; 1 otherwise } = ?? MPa
> Work height under the load
H_w = H - f = ?? mm
> Stress as a share of the tensile strength
r_S = S / S_ut = ??
```

### Checks

Smalley keeps the operating stress below the tensile strength for a static load, and below 0.8 of it for a cyclic one; for a dynamic application test r_S ≤ 0.8 instead. The work height must stay positive.

```calc
@test r_S ≤ 1
```

```calc
@test H_w > 0 'mm'
```
