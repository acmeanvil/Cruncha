+++
cruncha-template = 1
id = "mechanical.conicalSprings.uniformPitch"
version = "1.0.0"
title = "Conical compression spring, uniform pitch (Shigley)"
summary = "A conical compression spring of round wire wound at uniform pitch: its rate before the largest coil bottoms (Castigliano, Shigley Prob. 10-29) beside Juvinall's approximation, the deflection, the Bergsträsser factor and shear stress at the largest coil, the factor of safety on torsional yield and the solid height."
category = ["Mechanical engineering", "Springs"]
tags = ["conical spring", "tapered spring", "compression spring", "coil spring", "spring rate", "Castigliano", "Bergsträsser", "shear stress", "solid height", "Shigley", "Juvinall"]
from = "lacewing:mechanical.conicalSprings"
license = "GPL-3.0-or-later"
inputs = ["d", "D_1", "D_2", "N_a", "G", "F", "S_sy"]
outputs = ["k", "k_J", "y", "C", "K_B", "τ", "n", "L_s"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Problem 10-29 (rate of a conical spring by Castigliano's theorem); Eq. 10-5 (Bergsträsser factor), Eq. 10-7 (coil shear stress); Table 10-6 (torsional yield as a share of tensile strength)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 12.3, Eq. 12.8 and Fig. 12.6 (conical spring at its average diameter)"

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "p. 327 (conical springs: the rate rises as the coils bottom)"

[[tests]]
name = "Equal ends: a cylindrical spring, d⁴G/(8D³Na)"
source = "Lacewing Lab coil_springs_test: d = 2 mm, D1 = D2 = 16 mm, Na = 6, G = 79 GPa gives the helical spring's rate 2⁴ × 79 000 / (8 × 16³ × 6) = 6.429036 N/mm; by hand C = 8, KB = 34/29 = 1.172414, τ = 1.172414 × 8 × 10 × 16 / (π × 8) = 59.7105 MPa"
inputs = { d = "2 mm", D_1 = "16 mm", D_2 = "16 mm", N_a = "6", G = "79 GPa", F = "10 N", S_sy = "500 MPa" }
expect = { k = "6.429036 N/mm", k_J = "6.429036 N/mm", y = "1.555443 mm", C = "8", K_B = "1.172414", "τ" = "59.71054 MPa", n = "8.37373", L_s = "12 mm" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "2:1 cone: Juvinall's approximation 11 % stiff"
source = "Lacewing Lab juvinall_springs_test: d = 1.5 mm, R1 = 5, R2 = 10 mm, Na = 6, G = 79.3 GPa; kJ/k = 2(R1 + R2)(R1² + R2²)/(R1 + R2)³ = 16 × 15 × 125/(8 × 3375) = 1.1111; by hand k = 1.5⁴ × 79 300 / (16 × 6 × 15 × 125) = 2.230313 N/mm, y = 20/2.230313 = 8.96735 mm, C = 13.333, KB = 1.099338, τ = 331.786 MPa"
inputs = { d = "1.5 mm", D_1 = "10 mm", D_2 = "20 mm", N_a = "6", G = "79.3 GPa", F = "20 N", S_sy = "600 MPa" }
expect = { k = "2.230313 N/mm", k_J = "2.478125 N/mm", y = "8.967353 mm", K_B = "1.099338", "τ" = "331.7856 MPa", n = "1.808397" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "2:1 cone overloaded: 100 N"
source = "By hand: five times the load of the previous case, τ = 1658.93 MPa over Ssy = 600 MPa, n = 0.3617"
inputs = { d = "1.5 mm", D_1 = "10 mm", D_2 = "20 mm", N_a = "6", G = "79.3 GPa", F = "100 N", S_sy = "600 MPa" }
expect = { "τ" = "1658.928 MPa", n = "0.3616794" }
tolerance = "1e-5"
checks = "fail"
+++

## Conical compression spring

A compression spring of round wire whose coil diameter changes uniformly from one end to the other, wound at uniform pitch. Until the largest coil bottoms, the strain energy summed over a coil radius that varies linearly from R_1 to R_2 gives the rate d⁴G / [16 N_a (R_1 + R_2)(R_1² + R_2²)] (Castigliano's theorem, Shigley Prob. 10-29). After that the coils bottom one by one from the large end and the rate rises (Machinery's Handbook); that rising part is not followed here, so check that the deflection is less than the travel before the large coil bottoms. The highest stress is at the largest coil, where the moment arm is longest. For a cylindrical spring use the compression spring template.

### Spring

D_1 and D_2 are the mean coil diameters (to the wire's centre) at the two ends. Spring steel has G of about 79 GPa (11.5 Mpsi). The torsional yield strength S_sy is a share of the wire's tensile strength that depends on the material and on whether the set has been removed; Shigley Table 10-6 lists it for common spring wires.

```calc {description="Wire diameter"}
d = 1.5 mm_
```

```calc {description="Mean coil diameter at the small end"}
D_1 = 10 mm_
```

```calc {description="Mean coil diameter at the large end"}
D_2 = 20 mm_
```

```calc {description="Active coils"}
N_a = 6_
```

```calc {description="Shear modulus of the wire"}
G = 79.3 GPa_
```

```calc {description="Torsional yield strength of the wire"}
S_sy = 600 MPa_
```

### Load

```calc {description="Axial load"}
F = 20 N_
```

### Rate and deflection

Juvinall & Marshek approximate the cone as a helical spring at the average mean diameter, d⁴G / (8 D³ N_a) with D = R_1 + R_2. It is stiffer than the exact rate by 2(R_1² + R_2²)/(R_1 + R_2)², about 11 % for a 2:1 cone, so it suits gentle tapers only.

```calc
> Mean coil radii at the two ends
R_1 = D_1 / 2 = ?? mm
R_2 = D_2 / 2 = ?? mm
> Rate before any coil bottoms, Shigley Prob. 10-29
k = d^4 G / (16 N_a (R_1 + R_2) (R_1^2 + R_2^2)) = ?? N/mm
> Juvinall's approximation at the average mean diameter
k_J = d^4 G / (8 (R_1 + R_2)^3 N_a) = ?? N/mm
> Deflection under the load
y = F / k = ?? mm
```

### Stress at the largest coil

```calc
> Largest mean coil diameter
D_max = max(D_1, D_2) = ?? mm
> Spring index at the largest coil
C = D_max / d = ??
> Bergsträsser curvature factor, Shigley Eq. 10-5
K_B = (4 C + 2) / (4 C - 3) = ??
> Shear stress at the largest coil, Shigley Eq. 10-7
τ = K_B 8 F D_max / (π d^3) = ?? MPa
> Factor of safety on torsional yield
n = S_sy / τ = ??
```

### Solid height

If each coil nests inside the next (telescopes), the spring closes to about one wire diameter; if not, it closes to about N_a d, given here.

```calc {description="Solid height of the active coils, coils not telescoping"}
L_s = N_a d = ?? mm
```

### Check

```calc
@test τ ≤ S_sy
```
