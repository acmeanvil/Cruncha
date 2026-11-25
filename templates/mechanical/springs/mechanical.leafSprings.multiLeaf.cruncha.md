+++
cruncha-template = 1
id = "mechanical.leafSprings.multiLeaf"
version = "1.0.0"
title = "Multi-leaf spring: graduated and full-length leaves, nip and fatigue"
summary = "A semi-elliptic or cantilever (quarter-elliptic) laminated spring of graduated and extra full-length leaves: deflection and rate, the stress in each group of leaves, the factor of safety on yield, the nip and clamp force that equalise the stresses, NPTEL's deflection as a cross-check, and the fatigue safety factors to Goodman's line and Gerber's parabola."
category = ["Mechanical engineering", "Springs"]
tags = ["leaf spring", "laminated spring", "semi-elliptic", "quarter-elliptic", "cantilever", "graduated leaves", "full-length leaves", "master leaf", "nip", "nipping", "camber", "fatigue", "Goodman", "Gerber", "vehicle suspension"]
from = "lacewing:mechanical.leafSprings"
license = "GPL-3.0-or-later"
inputs = ["mount", "nip", "L_0", "b", "t", "n_g", "n_f", "E", "W", "S_y", "W_min", "K_f", "S_u", "S_n"]
outputs = ["L", "F", "n", "δ", "k", "σ_g", "σ_f", "σ_max", "n_y", "C", "P_i", "δ_c", "δ_NPTEL", "U", "σ_a", "σ_m", "n_Goodman", "n_Gerber"]

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Table 19, flat springs, cases 2 and 3 (uniform and triangular cantilevers), from which the two-group formulas here are derived"

[[references]]
work = "NPTEL (IIT Kharagpur), Design of Machine Elements, Module 7 Lesson 3: Design of Leaf Springs"
edition = "Version 2 ME"
locator = "Eqs. 7.3.8 (all-graduated stress and deflection) and 7.3.9 (deflection with full-length leaves); Secs. 7.3.3 (blunted ends) and 7.3.7 (master leaf)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 12.10, Fig. 12.22, Eqs. 12.13 to 12.15; Sample Problem 12.4 and Fig. 12.26 (leaf spring fatigue)"

[[tests]]
name = "Juvinall SP 12.4: semi-elliptic, five 7 mm leaves, 2000 to 10 000 N, Kf 1.3"
source = "Juvinall & Marshek 5th, Sample Problem 12.4 (via Lacewing Lab juvinall_springs_test): span 1364 mm, leaves 416/5 mm wide, E = 200 GPa; the book gets k = 30 N/mm, σa = 525 MPa (with b rounded), σa/σm = 0.667, the overload point on the infinite-life curve of Fig. 12.26b (Sn ≈ 700, Su ≈ 1600 MPa read off, Gerber nf ≈ 1.0), and Goodman's σa = 425 MPa (nf = 425/525 = 0.81)"
inputs = { mount = "1", nip = "0", L_0 = "1364 mm", b = "83.2 mm", t = "7 mm", n_g = "5", n_f = "0", E = "200 GPa", W = "10000 N", S_y = "1430 MPa", W_min = "2000 N", K_f = "1.3", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { k = "30 N/mm", "σ_a" = "525 MPa", q = "0.667", n_Gerber = "1.0", n_Goodman = "0.81" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Hand check of SP 12.4"
source = "By hand: L = 682 mm, F = 5000 N; δ = 12 × 5000 × 0.682³ / (200e9 × 0.0832 × 0.007³ × 10) = 333.470 mm, k = 29.9877 N/mm, σg = 12 × 5000 × 0.682 / (0.0832 × 0.007² × 10) = 1003.728 MPa, ny = 1.424688; σa = 1.3 × 1003.728 × 8000/20000 = 521.939 MPa, σm = 782.908 MPa; Goodman 1/(521.939/700 + 782.908/1600) = 0.809753; Gerber σm,lim = 792.464 MPa, nf = 1.012173"
inputs = { mount = "1", nip = "0", L_0 = "1364 mm", b = "83.2 mm", t = "7 mm", n_g = "5", n_f = "0", E = "200 GPa", W = "10000 N", S_y = "1430 MPa", W_min = "2000 N", K_f = "1.3", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { L = "682 mm", F = "5000 N", "δ" = "333.4701 mm", k = "29.98769 N/mm", "σ_g" = "1003.728 MPa", "σ_max" = "1003.728 MPa", n_y = "1.424688", "δ_c" = "1", "δ_NPTEL" = "333.4701 mm", U = "1667.351 J", "σ_a" = "521.9388 MPa", "σ_m" = "782.9082 MPa", n_Goodman = "0.809753", n_Gerber = "1.012173" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Cantilever, six graduated leaves: NPTEL Eq. 7.3.8"
source = "Lacewing Lab coil_springs_test: L = 0.5 m, b = 50 mm, t = 8 mm, E = 207 GPa, W = 2000 N; NPTEL Eq. 7.3.8 gives σ = 6FL/(Nbh²) = 312.5 MPa and δ = 6FL³/(ENbh³) = 47.17693 mm, with the correction factor 1"
inputs = { mount = "2", nip = "0", L_0 = "0.5 m", b = "50 mm", t = "8 mm", n_g = "6", n_f = "0", E = "207 GPa", W = "2000 N", S_y = "1000 MPa", W_min = "2000 N", K_f = "1", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { "σ_g" = "312.5 MPa", "σ_max" = "312.5 MPa", "δ" = "47.17693 mm", "δ_c" = "1", k = "42.3936 N/mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Semi-elliptic, six graduated leaves, 4000 N at the centre of a 1 m span"
source = "Lacewing Lab coil_springs_test: NPTEL Eq. 7.3.8 for a semi-elliptic spring, σ = 3WL/(Nbh²) with L the half span: 3 × 4000 × 0.5 / (6 × 0.05 × 0.008²) = 312.5 MPa"
inputs = { mount = "1", nip = "0", L_0 = "1 m", b = "50 mm", t = "8 mm", n_g = "6", n_f = "0", E = "207 GPa", W = "4000 N", S_y = "1000 MPa", W_min = "4000 N", K_f = "1", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { F = "2000 N", "σ_g" = "312.5 MPa", "δ" = "47.17693 mm", k = "84.7872 N/mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Cantilever, eight graduated and two full-length leaves"
source = "Lacewing Lab coil_springs_test: the full-length leaves carry 1.5 × the graduated stress, the nip C is the difference of the two groups' free deflections, 6FL³/(nEbt³) − 4FL³/(nEbt³) = 9.435386 mm, and NPTEL's deflection agrees within 5 %; by hand σg = 12 × 2000 × 0.5 / (0.05 × 0.008² × 28) = 170.4545 MPa, σf = 255.6818 MPa, δ = 25.73287 mm, Pi = 2 × 8 × 2000 / (10 × 28) = 145.4545 N, m = 0.2, δc = 0.8764747, δ NPTEL = 24.80963 mm; Wmin = 1000 N gives σa = 63.92 MPa, σm = 191.76 MPa, Goodman 4.735616, Gerber 5.750085"
inputs = { mount = "2", nip = "0", L_0 = "0.5 m", b = "50 mm", t = "8 mm", n_g = "8", n_f = "2", E = "207 GPa", W = "2000 N", S_y = "1000 MPa", W_min = "1000 N", K_f = "1", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { "σ_g" = "170.4545 MPa", "σ_f" = "255.6818 MPa", "σ_max" = "255.6818 MPa", "δ" = "25.73287 mm", C = "9.435386 mm", P_i = "145.4545 N", "δ_c" = "0.8764747", "δ_NPTEL" = "24.80963 mm", n_y = "3.911111", n_Goodman = "4.735616", n_Gerber = "5.750085" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same spring nipped: every leaf equally stressed"
source = "Lacewing Lab coil_springs_test: nipped, σ = 6FL/(nbt²) = 6 × 2000 × 0.5 / (10 × 0.05 × 64e-6) = 187.5 MPa in every leaf; by hand δ = 6FL³/(nEbt³) = 28.30616 mm"
inputs = { mount = "2", nip = "1", L_0 = "0.5 m", b = "50 mm", t = "8 mm", n_g = "8", n_f = "2", E = "207 GPa", W = "2000 N", S_y = "1000 MPa", W_min = "1000 N", K_f = "1", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { "σ_g" = "187.5 MPa", "σ_f" = "187.5 MPa", "σ_max" = "187.5 MPa", "δ" = "28.30616 mm", k = "70.656 N/mm" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "SP 12.4 spring at 13 000 N: short of infinite life"
source = "By hand: as SP 12.4 with W = 13 000 N, σg = 1304.85 MPa (ny = 1.096 against 1430 MPa), σa = 717.67, σm = 978.64 MPa, Gerber nf = 0.763 under 1"
inputs = { mount = "1", nip = "0", L_0 = "1364 mm", b = "83.2 mm", t = "7 mm", n_g = "5", n_f = "0", E = "200 GPa", W = "13000 N", S_y = "1430 MPa", W_min = "2000 N", K_f = "1.3", S_u = "1600 MPa", S_n = "700 MPa" }
expect = { n_y = "1.095914", n_Gerber = "0.7629673" }
tolerance = "1e-5"
checks = "fail"
+++

## Multi-leaf spring

A laminated spring of leaves all of one width b and thickness t: n_g graduated leaves of stepped lengths (the master leaf counted among them) and n_f extra full-length leaves beside the master. A semi-elliptic spring rests on its two eyes and carries its load W at the centre clamp; each half is a cantilever of half the span carrying W/2. A cantilever (quarter-elliptic) spring is clamped at one end and loaded at the other.

The graduated leaves act together as a triangular plate of uniform strength cut into strips, and the full-length leaves as a uniform cantilever (Machinery's Handbook Table 19, cases 3 and 2). Sharing the end load F between the two groups so that they deflect alike gives the formulas below; they are derived from Table 19, not quoted from it. With all leaves graduated they reduce to NPTEL's Eq. 7.3.8. NPTEL's Eq. 7.3.9, a correction factor on the deflection for full-length leaves, is shown as a cross-check; it agrees within a few per cent.

Left out: friction between the leaves, the shackle forces and twist on the master leaf (often made thinner or of a stronger steel, NPTEL Sec. 7.3.7), and blunted leaf ends, which change the deflection a little and the stress hardly at all (NPTEL Sec. 7.3.3).

### Mounting

```calc {description="Mounting: 1 semi-elliptic (load at the centre), 2 cantilever (load at the end)"}
mount = 1_
```

```calc
@test mount == 1 or mount == 2
```

Without nipping, the full-length leaves carry half as much stress again as the graduated ones. Nipping pre-bends the full-length leaves to a smaller radius, leaving a gap C between them and the graduated leaves that the centre clamp closes; with the right C every leaf ends equally stressed at the full load. Set nip = 1 to work the spring as built that way.

```calc {description="Nipped to equal stress at the full load: 1 yes, 0 no"}
nip = 0_
```

```calc
@test nip == 0 or nip == 1
```

### Leaves

```calc {description="Span between the eyes (semi-elliptic) or length from the clamp to the load (cantilever)"}
L_0 = 1364 mm_
```

```calc {description="Leaf width"}
b = 83.2 mm_
```

```calc {description="Leaf thickness"}
t = 7 mm_
```

```calc {description="Graduated leaves, the master leaf included (at least 1)"}
n_g = 5_
```

```calc {description="Extra full-length leaves beside the master (0 or more)"}
n_f = 0_
```

```calc
@test n_g ≥ 1 and n_f ≥ 0
```

```calc {description="Young's modulus of the leaves"}
E = 200 GPa_
```

```calc {description="Yield strength of the leaf steel"}
S_y = 1430 MPa_
```

### Load

```calc {description="Largest load: at the centre (semi-elliptic) or the end (cantilever)"}
W = 10000 N_
```

### Deflection, rate and stresses

```calc
> Cantilever length: half the span of a semi-elliptic spring
L = { L_0 / 2 if mount == 1; L_0 otherwise } = ?? mm
> End load on each cantilever
F = { W / 2 if mount == 1; W otherwise } = ?? N
> Total number of leaves
n = n_g + n_f = ??
> Deflection: the centre relative to the eyes, or the free end
δ = { 6 F L^3 / (n E b t^3) if nip == 1; 12 F L^3 / (E b t^3 (3 n_f + 2 n_g)) otherwise } = ?? mm
> Rate
k = W / δ = ?? N/mm
> Bending stress in the graduated leaves
σ_g = { 6 F L / (n b t^2) if nip == 1; 12 F L / (b t^2 (3 n_f + 2 n_g)) otherwise } = ?? MPa
> Bending stress in the full-length leaves
σ_f = { σ_g if nip == 1; 18 F L / (b t^2 (3 n_f + 2 n_g)) otherwise } = ?? MPa
> Stress in the most stressed leaves
σ_max = { σ_g if n_f == 0; σ_f otherwise } = ?? MPa
> Factor of safety on yield
n_y = S_y / σ_max = ??
> Energy stored
U = W δ / 2 = ?? J
```

### Nip

The gap between the full-length and graduated leaves, before clamping, that makes the stresses equal at this load is the difference of the two groups' free deflections; the clamp must press the groups together with P_i to close it (on both halves of a semi-elliptic spring). Both are zero without full-length leaves.

```calc
> Nip that equalises the stresses at W
C = 2 F L^3 / (n E b t^3) = ?? mm
> Clamp force that closes the nip
P_i = { 2 if mount == 1; 1 otherwise } n_f n_g F / (n (3 n_f + 2 n_g)) = ?? N
```

### NPTEL cross-check

```calc
> Share of full-length leaves
m = n_f / n = ??
> NPTEL's deflection correction factor, Eq. 7.3.9
δ_c = { 1 if n_f == 0; (1 - 4 m + 2 m^2 (1.5 - ln(m))) / (1 - m)^3 otherwise } = ??
> NPTEL's deflection, Eq. 7.3.9
δ_NPTEL = δ_c 6 F L^3 / (E n b t^3) = ?? mm
```

### Fatigue

The load cycles between W_min and W. The fatigue stress concentration K_f of the centre bolt hole goes on both the alternating and the mean stress, as Juvinall & Marshek do in Sample Problem 12.4 (1 for none). S_n is the fully reversed bending endurance limit of the leaves as made (size, surface and shot peening allowed for). The safety factors are taken along the load line, σ_a/σ_m held constant, to Goodman's line through S_n and S_u and to Gerber's parabola σ_a = S_n [1 − (σ_m/S_u)²]. Juvinall's infinite-life curve for leaf springs (Fig. 12.26b) lies close to Gerber's, which is checked here; Goodman's is the conservative choice.

```calc {description="Smallest load of the cycle"}
W_min = 2000 N_
```

```calc {description="Fatigue stress concentration factor of the centre bolt"}
K_f = 1.3_
```

```calc {description="Ultimate tensile strength of the leaf steel"}
S_u = 1600 MPa_
```

```calc {description="Corrected fully reversed bending endurance limit of the leaves"}
S_n = 700 MPa_
```

```calc
> Alternating stress in the most stressed leaves
σ_a = K_f σ_max (W - W_min) / (2 W) = ?? MPa
> Mean stress
σ_m = K_f σ_max (W + W_min) / (2 W) = ?? MPa
> Slope of the load line
q = σ_a / σ_m = ??
> Fatigue safety factor to Goodman's line
n_Goodman = 1 / (σ_a / S_n + σ_m / S_u) = ??
> Mean stress where the load line meets Gerber's parabola
σ_(m,lim) = (-q + sqrt(q^2 + 4 S_n^2 / S_u^2)) / (2 S_n / S_u^2) = ?? MPa
> Fatigue safety factor to Gerber's parabola
n_Gerber = σ_(m,lim) / σ_m = ??
```

### Checks

```calc
@test σ_max ≤ S_y
```

```calc
@test n_Gerber ≥ 1
```
