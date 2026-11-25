+++
cruncha-template = 1
id = "mechanical.impactLoading.beam"
version = "1.0.0"
title = "Weight dropped on a beam: impact stress and deflection"
summary = "Juvinall's energy method for a weight dropped onto a simply supported beam at mid-span or a cantilever at its end, optionally on supporting springs: the beam's rate, the impact factor, the deflections, the equivalent static force, the bending moment and stress, checked against a strength."
category = ["Mechanical engineering", "Mechanics"]
tags = ["impact", "impact factor", "drop", "falling weight", "beam", "cantilever", "simply supported", "bending stress", "impact stress", "energy method", "equivalent static force", "shock load", "Juvinall"]
from = "lacewing:mechanical.impactLoading"
license = "GPL-3.0-or-later"
inputs = ["W", "h", "beam", "L", "E", "I", "Z", "support", "k_sup", "S"]
outputs = ["k_b", "k", "δ_st", "IF", "δ", "F_e", "δ_b", "M", "σ", "U"]

[[references]]
work = "Juvinall, R.C., Marshek, K.M., Fundamentals of Machine Component Design"
edition = "5th"
locator = "Ch. 7, Sec. 7.2: Eqs. 7.1 and 7.2 (impact factor); Sample Problem 7.3 and its comments"

[[tests]]
name = "Juvinall Sample Problem 7.3: 100 lb dropped 12 in onto a 2 × 4 pine beam on two 100 lb/in springs"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.3: 60 in span, I 6.46 in⁴, Z 3.56 in³, E 10⁶ psi; impact factor 7.6, δ = 4.3 in, beam deflection 0.53 in, Fe = 760 lbf, stress 3 200 psi. The strength S = 5 000 psi is chosen for the check, not from the book"
inputs = { W = "100 lbf", h = "12 in", beam = "1", L = "60 in", E = "1e6 psi", I = "6.46 in^4", Z = "3.56 in^3", support = "1", k_sup = "200 lbf/in", S = "5000 psi" }
expect = { IF = "7.6", "δ" = "4.3 in", "δ_b" = "0.53 in", F_e = "760 lbf", "σ" = "3200 psi" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 7.3, comment: the beam on rigid supports"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.3 (comment 2): without the springs the impact factor is 19.6 and the stress 8 250 psi, over the 5 000 psi chosen for the check"
inputs = { W = "100 lbf", h = "12 in", beam = "1", L = "60 in", E = "1e6 psi", I = "6.46 in^4", Z = "3.56 in^3", support = "0", k_sup = "200 lbf/in", S = "5000 psi" }
expect = { IF = "19.6", "σ" = "8250 psi" }
tolerance = "0.1%"
checks = "fail"

[[tests]]
name = "Hand check: 200 N dropped 50 mm onto the end of a steel cantilever"
source = "By hand: k = 3 × 200e9 × 1e-6/1³ = 600 kN/m, δst = 0.33333 mm, IF = 1 + √(1 + 0.1/3.3333e-4) = 18.349352, δ = 6.1164505 mm, Fe = 3 669.8703 N, M = Fe·L = 3 669.8703 N·m, σ = M/Z = 183.49352 MPa, U = 200 × 0.0561165 = 11.223290 J; S = 250 MPa chosen for the check"
inputs = { W = "200 N", h = "50 mm", beam = "2", L = "1 m", E = "200 GPa", I = "1e-6 m^4", Z = "2e-5 m^3", support = "0", k_sup = "1 N/m", S = "250 MPa" }
expect = { k_b = "600 kN/m", IF = "18.349352", "δ" = "6.1164505 mm", F_e = "3669.8703 N", M = "3669.8703 N·m", "σ" = "183.49352 MPa", U = "11.223290 J" }
tolerance = "1e-7"
checks = "pass"
+++

## Weight dropped on a beam

A weight W falls a height h onto a beam: a simply supported beam struck at mid-span, or a cantilever struck at its free end, optionally resting on springs (Juvinall Sec. 7.2, Sample Problem 7.3). Equating the work of the weight, W(h + δ), to the elastic energy ½kδ² gives the impact deflection as the static deflection times the impact factor 1 + √(1 + 2h/δst) (Eqs. 7.1, 7.2). The equivalent static force F_e = kδ is the load that would give the same deflection, and the bending stress follows from it as for a static load.

Springs under the beam, in series with it, take up much of the energy at a far lower force: in Sample Problem 7.3 they cut the stress from 8 250 to 3 200 psi.

The beam is taken as massless and elastic, the weight as rigid, and damping is neglected (Juvinall p. 290). The stresses are then underestimated, more so for a heavy beam. No stress concentration is included.

### Weight

```calc {description="Falling weight (m·g)"}
W = 100 lbf_
```

```calc {description="Height of the fall before contact (0: a suddenly applied load)"}
h = 12 in_
```

### Beam

Enter the beam as a number: 1 simply supported, struck at mid-span; 2 cantilever, struck at its free end.

```calc {description="Beam: 1 simply supported, struck at mid-span; 2 cantilever, struck at its end"}
beam = 1_
```

```calc {description="Span of the simply supported beam, or length of the cantilever"}
L = 60 in_
```

```calc {description="Elastic modulus of the beam"}
E = 1e6 psi_
```

```calc {description="Second moment of area of the beam's section about its bending axis"}
I = 6.46 in⁴_
```

```calc {description="Section modulus of the beam about its bending axis"}
Z = 3.56 in³_
```

```calc {description="Strength the peak bending stress is compared with"}
S = 5000 psi_
```

### Supporting springs

```calc {description="Supporting springs in series: 1 included, 0 none (rigid supports)"}
support = 1_
```

```calc {description="Rate of the supporting springs, all together"}
k_sup = 200 lbf/in_
```

### Impact

```calc
> Rate of the beam at the point struck
k_b = {48 E I / L^3 if beam == 1; 3 E I / L^3 if beam == 2} = ?? lbf/in
> Rate of the system (beam and supporting springs in series)
k = {k_b if support == 0; 1 / (1 / k_b + 1 / k_sup) if support == 1} = ?? lbf/in
> Static deflection
δ_st = W / k = ?? in
> Impact factor, Eqs. 7.1 and 7.2
IF = 1 + sqrt(1 + 2 h / δ_st) = ??
> Impact deflection
δ = IF δ_st = ?? in
> Equivalent static force
F_e = k δ = ?? lbf
> Deflection of the beam itself
δ_b = F_e / k_b = ?? in
> Energy absorbed: the work of the falling weight
U = W (h + δ) = ?? in·lbf
```

### Stress

```calc
> Largest bending moment: under the load, or at the cantilever's root
M = {F_e L / 4 if beam == 1; F_e L if beam == 2} = ?? in·lbf
> Peak bending stress
σ = M / Z = ?? psi
```

```calc
@test σ ≤ S
```
