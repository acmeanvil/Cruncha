+++
cruncha-template = 1
id = "mechanical.threads.strength"
version = "1.0.0"
title = "Thread strength: tension and stripping of a mating thread pair"
summary = "Tensile-stress and cross-shear areas of an external thread, the stripping areas of both threads at least material over the length of engagement, the stresses from axial load, shear, bending and torque, the capacities at an allowable stress and at ultimate, and a utilization checked against 1."
category = ["Mechanical engineering", "Fastening"]
tags = ["thread", "screw thread", "stripping", "thread shear", "length of engagement", "tensile stress area", "bolt", "tapped hole", "nut", "Unified", "ISO metric", "Acme", "buttress", "capacity"]
from = "lacewing:mechanical.threadsAnySize"
license = "GPL-3.0-or-later"
inputs = ["d", "P", "UN", "d_min", "d_2min", "d_3min", "D_1max", "D_2max", "α_1", "α_2", "L_e", "d_i", "S_yb", "S_ub", "S_yn", "S_un", "n_d", "F", "V", "M", "T"]
outputs = ["A_t", "A_x", "A_s", "A_n", "σ", "τ", "σ_vm", "τ_s", "τ_n", "F_t", "F_ss", "F_sn", "V_x", "F_a", "F_u", "U"]

[[references]]
work = "Machinery's Handbook (Oberg et al.)"
edition = "26th"
locator = "Bolts and Nuts: Eq. 2a (tensile stress area), Eqs. 5 and 6 (shear areas of external and internal threads)"

[[references]]
work = "ASME B1.13M Metric Screw Threads: M Profile"
edition = "2005 (R2020)"
locator = "Appendix B-1 (tensile stress area), Appendix B-2 (thread shear areas); limits of size tables"

[[references]]
work = "ASME B1.1 Unified Inch Screw Threads (UN and UNR Thread Form)"
edition = "2019"
locator = "Sec. 5.8 (formulas for limits of size); limits of size tables"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-11 (transverse shear in a round section), Sec. 5-5 (distortion energy)"

[[tests]]
name = "M12 × 1.75 6g/6H, Le 12 mm, class 8.8 in a steel nut, combined loads"
source = "Lacewing Lab thread_standards_test: limits of B1.13M Tables 14 and 15 (d min 11.701, d2 min 10.679, d3 min 9.601, D1 max 10.441, D2 max 11.063 mm); stripping areas by the App. B-2 equations; At = 84.27 mm²; tension governs (F at yield = At·640 MPa)"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", d_min = "11.701 mm", d_2min = "10.679 mm", d_3min = "9.601 mm", D_1max = "10.441 mm", D_2max = "11.063 mm", L_e = "12 mm", S_yb = "640 MPa", S_ub = "800 MPa", S_yn = "640 MPa", S_un = "800 MPa", n_d = "1.5", F = "20 kN", V = "1 kN", M = "5 N·m", T = "10 N·m" }
expect = { A_t = "84.27 mm²", A_s = "227.71 mm²", A_n = "313.41 mm²", F_a = "35.954 kN" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: the combined-load case exactly"
source = "By hand: ridge widths 0.875 + 0.57735(10.679 − 10.441) = 1.012409 mm and 0.875 + 0.57735(11.701 − 11.063) = 1.243349 mm; As = π·12·10.441·1.012409/1.75 = 227.7148 mm², An = π·12·11.701·1.243349/1.75 = 313.4074 mm²; Z = π·9.601³/32 = 86.8859 mm³; σ = 20000/84.26637 + 5000/86.8859 = 294.889 MPa; τ = 4·1000/(3·72.39738) + 10000/173.7718 = 75.9636 MPa; σvm = 322.910 MPa; U = 322.910/(640/1.5) = 0.756821"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", d_min = "11.701 mm", d_2min = "10.679 mm", d_3min = "9.601 mm", D_1max = "10.441 mm", D_2max = "11.063 mm", L_e = "12 mm", S_yb = "640 MPa", S_ub = "800 MPa", S_yn = "640 MPa", S_un = "800 MPa", n_d = "1.5", F = "20 kN", V = "1 kN", M = "5 N·m", T = "10 N·m" }
expect = { A_s = "227.714832 mm²", A_n = "313.407412 mm²", A_t = "84.2663675 mm²", A_x = "72.3973752 mm²", "σ" = "294.889351 MPa", "τ" = "75.9636106 MPa", "σ_vm" = "322.910421 MPa", "τ_s" = "87.8291493 MPa", "τ_n" = "63.8147 MPa", F_ss = "56.0603555 kN", F_sn = "77.1567261 kN", V_x = "17.8232685 kN", F_u = "67.413094 kN", U = "0.756821299" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "M12 class 8.8 in an aluminium tapped hole: the internal thread strips first"
source = "Lacewing Lab thread_standards_test (aluminium Sy 240, Su 290 MPa, design factor 2); by hand F_sn = 313.4074·0.577·240/2 = 21.700 kN, under F_t = 84.26637·640/2 = 26.965 kN"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", d_min = "11.701 mm", d_2min = "10.679 mm", d_3min = "9.601 mm", D_1max = "10.441 mm", D_2max = "11.063 mm", L_e = "12 mm", S_yb = "640 MPa", S_ub = "800 MPa", S_yn = "240 MPa", S_un = "290 MPa", n_d = "2", F = "20 kN", V = "0 kN", M = "0 N·m", T = "0 N·m" }
expect = { F_t = "26.9652376 kN", F_sn = "21.7003292 kN", F_a = "21.7003292 kN", U = "0.921645003" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "M12 cut on a tube with a 6 mm bore"
source = "Lacewing Lab thread_standards_test: At = 0.7854(12 − 0.9382·1.75)² − π/4·6² = 55.992 mm²"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", d_min = "11.701 mm", d_2min = "10.679 mm", d_3min = "9.601 mm", D_1max = "10.441 mm", D_2max = "11.063 mm", L_e = "12 mm", d_i = "6 mm", S_yb = "640 MPa", S_ub = "800 MPa", S_yn = "640 MPa", S_un = "800 MPa", n_d = "1.5", F = "10 kN", V = "0 kN", M = "0 N·m", T = "0 N·m" }
expect = { A_t = "55.9920336 mm²", A_x = "44.1230413 mm²" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "1/2-28 UNEF 2A/2B, Le 1/2 in"
source = "Lacewing Lab thread_standards_test: limits of ASME B1.1 Tables 18A and 18B (d min 0.4924, d2 min 0.4720, D1 max 0.470, D2 max 0.4816 in); At = 0.7854(0.5 − 0.9743/28)² = 0.16997 in² (B1.1 lists 0.1700); by hand As = π·0.5·0.470·0.019012/(1/28) = 0.39301 in², An = π·0.5·0.4924·0.024093/(1/28) = 0.52177 in². d3 min is taken as the basic 0.4536 in, which only the cross-shear area uses"
inputs = { d = "0.5 in", P = "0.0357142857 in", UN = "1", d_min = "0.4924 in", d_2min = "0.4720 in", d_3min = "0.4536 in", D_1max = "0.470 in", D_2max = "0.4816 in", L_e = "0.5 in", S_yb = "92 ksi", S_ub = "120 ksi", S_yn = "92 ksi", S_un = "120 ksi", n_d = "1.5", F = "2 kips", V = "0 kips", M = "0 lbf·in", T = "0 lbf·in" }
expect = { A_t = "0.1700 in²", A_s = "0.393006736 in²", A_n = "0.521769014 in²" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "M12 at 40 kN: over the allowable"
source = "By hand: σ = 40000/84.26637 = 474.69 MPa against 640/1.5 = 426.67 MPa, U = 1.1125"
inputs = { d = "12 mm", P = "1.75 mm", UN = "0", d_min = "11.701 mm", d_2min = "10.679 mm", d_3min = "9.601 mm", D_1max = "10.441 mm", D_2max = "11.063 mm", L_e = "12 mm", S_yb = "640 MPa", S_ub = "800 MPa", S_yn = "640 MPa", S_un = "800 MPa", n_d = "1.5", F = "40 kN", V = "0 kN", M = "0 N·m", T = "0 N·m" }
expect = { U = "1.11254351" }
tolerance = "1e-6"
checks = "fail"
+++

## Thread strength: tension and stripping of a mating thread pair

What an external thread (a bolt, screw or stud) and the internal thread it engages (a nut or tapped hole) can carry: the external thread in tension, cross shear, bending and torsion, and either thread's ridges sheared off along the length of engagement (stripping).

The stripping areas are the geometric minimums at least material: the external thread's ridges sheared at the internal thread's largest minor diameter, and the internal thread's at the external thread's smallest major diameter. Machinery's Handbook and ASME B1.13M Appendix B-2 give them for 60° threads; here they are written for any pair of flank angles, so they also serve Acme (14.5°/14.5°), trapezoidal (15°/15°) and buttress (7°/45°, load on the 7° flank only) threads. Real stripping strength is somewhat less: the nut dilates, the threads bend, and the load is not shared evenly along the engagement.

Bending, torsion and cross shear act on the external thread's least minor diameter (conservative), and the axial load on the tensile-stress area. Shear strength is taken as 0.577 S_y at the allowable and 0.5 S_u at ultimate.

### Thread size and limits

The limits of size depend on the thread's tolerance class. Look them up in the thread's standard and enter them here: ASME B1.1 (Unified inch) or ASME B1.13M / ISO 965-1 (ISO metric) for 60° threads; ASME B1.5 (Acme), DIN 103 / ISO 2904 (trapezoidal), ASME B1.9 or DIN 513 (buttress). The defaults are M12 × 1.75, 6g external and 6H internal.

```calc {description="Basic major (nominal) diameter"}
d = 12 mm_
```

```calc {description="Pitch (1/n for n threads per inch)"}
P = 1.75 mm_
```

```calc {description="Tensile-stress area formula: 1 for Unified inch (UN), 0 for ISO metric (M)"}
UN = 0_
```

```calc {description="External thread: minimum major diameter"}
d_min = 11.701 mm_
```

```calc {description="External thread: minimum pitch diameter"}
d_2min = 10.679 mm_
```

```calc {description="External thread: minimum minor (root) diameter"}
d_3min = 9.601 mm_
```

```calc {description="Internal thread: maximum minor diameter"}
D_1max = 10.441 mm_
```

```calc {description="Internal thread: maximum pitch diameter"}
D_2max = 11.063 mm_
```

```calc {description="Flank half-angle of the load-bearing flank (30° for UN and M)"}
α_1 = 30°_
```

```calc {description="Flank half-angle of the other flank (30° for UN and M)"}
α_2 = 30°_
```

```calc {description="Length of engagement: full threads in the nut or tapped hole, less chamfers"}
L_e = 12 mm_
```

```calc {description="Bore of a tube the external thread is cut on (0 for a solid bar)"}
d_i = 0 mm_
```

### Materials

```calc {description="External thread: yield strength"}
S_yb = 640 MPa_
```

```calc {description="External thread: tensile strength"}
S_ub = 800 MPa_
```

```calc {description="Internal thread (nut or tapped part): yield strength"}
S_yn = 640 MPa_
```

```calc {description="Internal thread (nut or tapped part): tensile strength"}
S_un = 800 MPa_
```

```calc {description="Design factor on yield"}
n_d = 1.5_
```

### Loads

```calc {description="Axial load, tension positive"}
F = 20 kN_
```

```calc {description="Transverse shear across the thread, resultant of both axes"}
V = 1 kN_
```

```calc {description="Bending moment at the thread, resultant of both axes"}
M = 5 N·m_
```

```calc {description="Torque about the thread axis"}
T = 10 N·m_
```

### Areas

The tensile-stress area is ASME B1.1's 0.7854(d − 0.9743P)² for Unified threads and B1.13M's 0.7854(d − 0.9382P)² for metric ones. For Unified threads of steel over 100 ksi tensile strength, Machinery's Handbook gives a smaller area from the minimum pitch diameter (its Eq. 2b); for an Acme, trapezoidal or buttress thread, use the area its standard gives (about π/4 of the square of the mean of the minimum pitch and minor diameters) and adjust this cell.

```calc {description="Tensile-stress area, less the bore"}
A_t = π / 4 ((d - { 0.9743 if UN == 1; 0.9382 otherwise } P)^2 - d_i^2) = ?? mm²
```

```calc {description="Cross-shear area on the least minor diameter, less the bore"}
A_x = π / 4 (d_3min^2 - d_i^2) = ?? mm²
```

A ridge widens with depth by the sum of its flanks' tangents. Its width along the axis is P/2 at the pitch diameter.

```calc
> Width of the external thread's ridge at the internal minor diameter
w_s = P / 2 + (d_2min - D_1max) / 2 (tan(α_1) + tan(α_2)) = ?? mm
> Width of the internal thread's ridge at the external major diameter
w_n = P / 2 + (d_min - D_2max) / 2 (tan(α_1) + tan(α_2)) = ?? mm
> External thread stripping area, Machinery's Eq. 5
A_s = π L_e D_1max w_s / P = ?? mm²
> Internal thread stripping area, Machinery's Eq. 6
A_n = π L_e d_min w_n / P = ?? mm²
```

### Stresses

```calc
> Section modulus at the least minor diameter
Z = π / 32 (d_3min^4 - d_i^4) / d_3min = ?? mm³
> Normal stress: axial on the tensile area plus bending
σ = F / A_t + M / Z = ?? MPa
> Peak transverse shear in the round (hollow) section plus torsion, added as at one point
τ = 4 V / (3 A_x) (d_3min^2 + d_3min d_i + d_i^2) / (d_3min^2 + d_i^2) + abs(T) / (2 Z) = ?? MPa
> Von Mises equivalent stress, Shigley Sec. 5-5
σ_vm = sqrt(σ^2 + 3 τ^2) = ?? MPa
> Mean shear on the external thread's ridges
τ_s = abs(F) / A_s = ?? MPa
> Mean shear on the internal thread's ridges
τ_n = abs(F) / A_n = ?? MPa
```

### Capacities at the allowable stress

Yield over the design factor; the shear allowable is 0.577 S_y over it.

```calc
> External thread in tension
F_t = A_t S_yb / n_d = ?? kN
> External thread stripping
F_ss = A_s 0.577 S_yb / n_d = ?? kN
> Internal thread stripping
F_sn = A_n 0.577 S_yn / n_d = ?? kN
> Cross shear of the external thread on its minor diameter
V_x = A_x 0.577 S_yb / n_d = ?? kN
> Axial capacity: the least of tension and both strippings
F_a = min(F_t, F_ss, F_sn) = ?? kN
```

A well-proportioned joint is governed by the external thread in tension. When F_sn is the least, the nut or tapped hole strips first: lengthen the engagement or use a stronger nut.

```calc {description="Axial capacity at ultimate: tension at S_u, stripping at 0.5 S_u"}
F_u = min(A_t S_ub, A_s 0.5 S_ub, A_n 0.5 S_un) = ?? kN
```

### Check

```calc {description="Utilization: the largest of σvm and each stripping shear over its allowable"}
U = max(σ_vm / (S_yb / n_d), τ_s / (0.577 S_yb / n_d), τ_n / (0.577 S_yn / n_d)) = ??
```

```calc
@test U ≤ 1
```
