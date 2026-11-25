+++
cruncha-template = 1
id = "mechanical.compressionSprings.static"
version = "1.0.0"
title = "Helical compression spring, static design (Shigley)"
summary = "A round-wire helical compression spring under a static working load: wire strength from Sut = A/d^m, spring index and Bergsträsser factor, rate, stress and safety factor at the working load, solid length, solid force and safety factor at solid, the solid-safe free length, pitch, the buckling limit of Eq. 10-12 and the surge frequency, with Shigley's recommended design ranges checked."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "compression spring", "helical spring", "coil spring", "spring rate", "Bergsträsser", "Wahl", "solid length", "free length", "buckling", "surge", "natural frequency", "spring index", "music wire", "Shigley"]
from = "lacewing:mechanical.compressionSprings"
license = "GPL-3.0-or-later"
inputs = ["d", "OD", "N_a", "ends", "L_0", "A", "m", "d_A", "f_sy", "G", "E", "α", "ρ", "F"]
outputs = ["D", "C", "K_B", "S_ut", "S_sy", "k", "y", "τ", "n", "N_t", "L_s", "F_s", "τ_s", "n_s", "ξ", "F_sy", "L_(0,safe)", "p", "L_(0,cr)", "m_a", "f_n"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 10-2 to 10-8: Eqs. 10-1 (index), 10-5 and 10-7 (Bergsträsser factor and stress), 10-9 (rate), 10-12 and Table 10-2 (buckling), 10-14 and Table 10-4 (Sut = A/d^m), Table 10-5 (G, E), Table 10-6 (static allowables), Table 10-1 (ends), 10-17 to 10-21 (design ranges), 10-25 and 10-27 (surge); Examples 10-1 and 10-4"

[[tests]]
name = "Shigley Ex. 10-1: no. 16 music wire, OD 7/16 in, squared ends, 12½ turns"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-1: Sut = 324 kpsi, C = 10.8, KB = 1.124, F at Ssy = 6.46 lbf, k = 4.13 lbf/in (the book rounds D to 0.400 in), Ls = 0.500 in, solid-safe L0 = 2.06 in, p = 0.186 in. Closed solid it just reaches Ssy (ns = 1, under 1.2), and with this wire's E and G (29.0, 11.85 Mpsi) the buckling limit is 2.03 in, just under 2.06 in"
inputs = { d = "0.037 in", OD = "0.4375 in", N_a = "10.5", ends = "3", L_0 = "2.0657 in", A = "201 kpsi", m = "0.145", d_A = "1 in", f_sy = "0.45", G = "11.85 Mpsi", E = "29.0 Mpsi", F = "6.46 lbf" }
expect = { S_ut = "324 kpsi", C = "10.8", K_B = "1.124", F_sy = "6.46 lbf", k = "4.13 lbf/in", L_s = "0.500 in", "L_(0,safe)" = "2.06 in", p = "0.186 in", n_s = "1.0" }
tolerance = "0.5%"
checks = "fail"

[[tests]]
name = "Shigley Ex. 10-4: music wire 0.092 in, OD 9/16 in, 21 active coils, squared and ground, 35 lbf"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-4: Sut = 284.1 kpsi, KB = 1.287 (the book rounds C to 5.11), k = 48.1 lbf/in, weight of the active coils 0.0586 lbf, surge frequency 281 Hz. At L0 = 4.375 in it is far too long to close solid safely (ns = 0.59) and would need guiding"
inputs = { d = "0.092 in", OD = "0.5625 in", N_a = "21", ends = "4", L_0 = "4.375 in", A = "201 kpsi", m = "0.145", d_A = "1 in", f_sy = "0.45", G = "11.75 Mpsi", E = "28.5 Mpsi", "ρ" = "0.284 lb/in³", F = "35 lbf" }
expect = { S_ut = "284.1 kpsi", K_B = "1.287", k = "48.1 lbf/in", m_a = "0.0586 lb", f_n = "281 Hz" }
tolerance = "0.4%"
checks = "fail"

[[tests]]
name = "Hand check: Ex. 10-1 exactly"
source = "By hand: D = 0.4005 in, C = 10.824324, KB = (4C + 2)/(4C − 3) = 1.1240778, Sut = 201/0.037^0.145 = 324.19537 kpsi, Ssy = 145.88792 kpsi, k = 0.037⁴ × 11.85e6/(8 × 0.4005³ × 10.5) = 4.1156497 lbf/in, F_sy = Ssy πd³/(8 KB D) = 6.4459286 lbf, y = 1.5661995 in, Nt = 12.5, Ls = 0.037 × 13.5 = 0.4995 in, L0 safe = 2.0656995 in"
inputs = { d = "0.037 in", OD = "0.4375 in", N_a = "10.5", ends = "3", L_0 = "2.0657 in", A = "201 kpsi", m = "0.145", d_A = "1 in", f_sy = "0.45", G = "11.85 Mpsi", E = "29.0 Mpsi", F = "6.46 lbf" }
expect = { D = "0.4005 in", C = "10.824324", K_B = "1.1240778", S_ut = "324.19537 kpsi", S_sy = "145.88792 kpsi", k = "4.1156497 lbf/in", F_sy = "6.4459286 lbf", N_t = "12.5", L_s = "0.4995 in", "L_(0,safe)" = "2.0656995 in" }
tolerance = "1e-6"
checks = "fail"
[[tests]]
name = "Hand check: a music-wire spring inside every recommended range"
source = "By hand: d = 0.105 in music wire (Sut = 201/0.105^0.145 = 278.69143 kpsi, Ssy = 125.41114 kpsi), OD 0.9 in, D = 0.795 in, C = 7.5714286, KB = 1.1832461, 8 active coils squared and ground, L0 = 2.0 in, F = 20 lbf: k = 0.105⁴ × 11.75e6/(8 × 0.795³ × 8) = 44.413367 lbf/in, Ls = 1.05 in, Fs = 42.192699 lbf, τs = 87.307535 kpsi, ns = 1.4364298, ξ = 1.1096350, τ = 41.385139 kpsi, n = 3.0303424, p = 0.22375 in, L0cr = 4.0092924 in, mass 0.049135322 lb, fn = 295.37479 Hz"
inputs = { d = "0.105 in", OD = "0.9 in", N_a = "8", ends = "4", L_0 = "2.0 in", A = "201 kpsi", m = "0.145", d_A = "1 in", f_sy = "0.45", G = "11.75 Mpsi", E = "28.5 Mpsi", "ρ" = "0.284 lb/in³", "α" = "0.5", F = "20 lbf" }
expect = { S_ut = "278.69143 kpsi", C = "7.5714286", K_B = "1.1832461", k = "44.413367 lbf/in", L_s = "1.05 in", F_s = "42.192699 lbf", "τ_s" = "87.307535 kpsi", n_s = "1.4364298", "ξ" = "1.1096350", "τ" = "41.385139 kpsi", n = "3.0303424", p = "0.22375 in", "L_(0,cr)" = "4.0092924 in", m_a = "0.049135322 lb", f_n = "295.37479 Hz" }
tolerance = "1e-6"
checks = "pass"
+++

## Helical compression spring, static design

A helical compression spring of round wire under a static (or rarely repeated) working load F, after Shigley Secs. 10-2 to 10-8. The stress is the torsion of the wire with Bergsträsser's factor K_B for direct shear and curvature, compared with Shigley's static allowable S_sy, a fraction of the wire's tensile strength. The checks are Shigley's recommended ranges for a static design: the spring index, the number of active coils, the overrun to closure and the safety factor when closed solid. For a load that cycles, use the compression spring fatigue template; for a set-removed spring, Table 10-6's after-set fraction goes with the direct-shear factor K_s = (2C + 1)/(2C) in place of K_B, which this template does not do.

### Wire

Shigley fits the minimum tensile strength of spring wire as S_ut = A/d^m (Eq. 10-14), with A and m for each wire in Table 10-4; A is for d in the unit d_A (kpsi with d in inches, or MPa with d in millimetres). The moduli G and E are in Table 10-5. The static allowable S_sy is a fraction of S_ut from Table 10-6 (before set removal): 0.45 for music wire and cold-drawn carbon steel, 0.50 for hardened and tempered carbon and low-alloy steels, 0.35 for austenitic stainless and nonferrous wire.

```calc {description="Wire diameter"}
d = 0.105 in_
```

```calc {description="Table 10-4 constant A of Sut = A/d^m, for d in units of d_A"}
A = 201 kpsi_
```

```calc {description="Table 10-4 exponent m of Sut = A/d^m"}
m = 0.145_
```

```calc {description="Diameter unit Table 10-4's A is for (1 in for kpsi·in^m, 1 mm for MPa·mm^m)"}
d_A = 1 in_
```

```calc {description="Static torsional allowable as a fraction of Sut, Shigley Table 10-6"}
f_sy = 0.45_
```

```calc {description="Shear modulus of the wire, Shigley Table 10-5"}
G = 11.75 Mpsi_
```

```calc {description="Young's modulus of the wire, Shigley Table 10-5"}
E = 28.5 Mpsi_
```

```calc {description="Density of the wire"}
ρ = 0.284 lb/in³_
```

```calc
> Minimum tensile strength, Eq. 10-14
S_ut = A / (d / d_A)^m = ?? kpsi
> Static torsional allowable, Table 10-6
S_sy = f_sy S_ut = ?? kpsi
```

### Coils and ends

The ends set the inactive coils, the solid length and the pitch (Shigley Table 10-1). Enter the type as a number: 1 plain, 2 plain and ground, 3 squared (closed), 4 squared and ground. Squared and ground ends are the usual choice for a spring that must stand square.

```calc {description="Outside diameter of the coil"}
OD = 0.9 in_
```

```calc {description="Active coils"}
N_a = 8_
```

```calc {description="End type, Shigley Table 10-1: 1 plain, 2 plain and ground, 3 squared, 4 squared and ground"}
ends = 4_
```

```calc {description="Free length"}
L_0 = 2.0 in_
```

```calc
> Mean coil diameter
D = OD - d = ?? in
> Spring index, Eq. 10-1
C = D / d = ??
> Bergsträsser factor, Eq. 10-5
K_B = (4 C + 2) / (4 C - 3) = ??
> Spring rate, Eq. 10-9
k = d^4 G / (8 D^3 N_a) = ?? lbf/in
```

```calc
> End coils, Table 10-1
N_e = {0 if ends == 1; 1 if ends == 2; 2 if ends == 3; 2 if ends == 4} = ??
> Total coils
N_t = N_a + N_e = ??
> Solid length, Table 10-1
L_s = {d (N_t + 1) if ends == 1; d N_t if ends == 2; d (N_t + 1) if ends == 3; d N_t if ends == 4} = ?? in
> Pitch of the body coils, Table 10-1
p = {(L_0 - d) / N_a if ends == 1; L_0 / (N_a + 1) if ends == 2; (L_0 - 3 d) / N_a if ends == 3; (L_0 - 2 d) / N_a if ends == 4} = ?? in
```

### Working load

```calc {description="Working (largest) load"}
F = 20 lbf_
```

```calc
> Deflection at the working load
y = F / k = ?? in
> Shear stress at the working load, Eq. 10-7
τ = K_B 8 F D / (π d^3) = ?? kpsi
> Static safety factor at the working load
n = S_sy / τ = ??
```

### Closed solid

A spring that can be closed solid in service, or in assembly, should not take a set when it is: Shigley asks n_s ≥ 1.2 at solid (Eq. 10-21). The solid-safe free length is the longest for which closing the spring solid just reaches S_sy (Ex. 10-1(f)).

```calc
> Force to close the spring solid
F_s = k (L_0 - L_s) = ?? lbf
> Shear stress closed solid
τ_s = K_B 8 F_s D / (π d^3) = ?? kpsi
> Safety factor closed solid, Eq. 10-21
n_s = S_sy / τ_s = ??
> Fractional overrun to closure, Eq. 10-17
ξ = F_s / F - 1 = ??
> Load that reaches the allowable S_sy
F_sy = S_sy π d^3 / (8 K_B D) = ?? lbf
> Solid-safe free length, Ex. 10-1(f)
L_(0,safe) = L_s + F_sy / k = ?? in
```

### Buckling

A spring longer than L_(0,cr) can buckle (Eq. 10-12); it must then be guided on a rod or in a hole. The end-condition constant α is from Shigley Table 10-2: 0.5 between flat parallel plates, 0.707 one end fixed and one pivoted, 1 both ends pivoted, 2 one end clamped and the other free.

```calc {description="End-condition constant for buckling, Shigley Table 10-2"}
α = 0.5_
```

```calc {description="Free length below which the spring cannot buckle, Eq. 10-12"}
L_(0,cr) = π D / α sqrt(2 (E - G) / (2 G + E)) = ?? in
```

### Surge

The first natural frequency of a spring between two flat plates is half the square root of its rate over the mass of its active coils (Eqs. 10-25, 10-27). Shigley recommends keeping it 15 to 20 times the forcing frequency.

```calc
> Mass of the active coils, Eq. 10-27
m_a = π^2 d^2 D N_a ρ / 4 = ?? lb
> Surge (first natural) frequency, both ends on plates, Eq. 10-25
f_n = 1 / 2 sqrt(k / m_a) = ?? Hz
```

### Checks

Shigley's recommended ranges for a static design (Eqs. 10-18 to 10-21), and stability.

```calc
@test 4 ≤ C ≤ 12
```

```calc
@test 3 ≤ N_a ≤ 15
```

```calc
@test ξ ≥ 0.15
```

```calc
@test n_s ≥ 1.2
```

```calc
@test L_0 < L_(0,cr)
```
