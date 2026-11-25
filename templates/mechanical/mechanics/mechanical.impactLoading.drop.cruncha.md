+++
cruncha-template = 1
id = "mechanical.impactLoading.drop"
version = "1.0.0"
title = "Falling weight on an elastic structure (energy method)"
summary = "Juvinall's energy method for a weight dropped from a height onto an elastic structure of known rate, optionally on a supporting spring in series: the static deflection, the impact factor 1 + √(1 + 2h/δst), the impact deflection, the equivalent static force, the energy absorbed, the structure's own deflection and the natural period that tells an impact from a static load."
category = ["Mechanical engineering", "Mechanics"]
tags = ["impact", "impact factor", "drop", "falling weight", "suddenly applied load", "energy method", "equivalent static force", "shock load", "spring", "Juvinall"]
from = "lacewing:mechanical.impactLoading"
license = "GPL-3.0-or-later"
inputs = ["W", "h", "k_st", "support", "k_sup"]
outputs = ["k", "δ_st", "IF", "δ", "F_e", "U", "δ_struct", "τ_n"]

[[references]]
work = "Juvinall, R.C., Marshek, K.M., Fundamentals of Machine Component Design"
edition = "5th"
locator = "Ch. 7, Secs. 7.1 and 7.2: Table 7.1 and Eq. a (natural period), Eqs. 7.1 and 7.2 (impact factor), Fig. 7.3; Sample Problem 7.3"

[[tests]]
name = "Juvinall Sample Problem 7.3: 100 lb dropped 12 in onto a beam of 1436 lb/in on two 100 lb/in springs"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.3: the beam's rate 48EI/L³ = 1435.6 lb/in (E 10⁶ psi, I 6.46 in⁴, 60 in span); δst = 0.57 in, impact factor 7.6, δ = 4.3 in, Fe = 760 lbf, beam deflection 0.53 in"
inputs = { W = "100 lbf", h = "12 in", k_st = "1435.5556 lbf/in", support = "1", k_sup = "200 lbf/in" }
expect = { "δ_st" = "0.57 in", IF = "7.6", "δ" = "4.3 in", F_e = "760 lbf", "δ_struct" = "0.53 in" }
tolerance = "1%"

[[tests]]
name = "Juvinall Sample Problem 7.3, comment: the same beam without the springs"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.3 (comment 2): with the beam on rigid supports the impact factor is 19.6. By hand: δst = 100/1435.5556 = 0.069659 in, δ = 1.364526 in, Fe = 1 958.853 lbf"
inputs = { W = "100 lbf", h = "12 in", k_st = "1435.5556 lbf/in", support = "0" }
expect = { IF = "19.6", "δ_st" = "0.0696594 in", "δ" = "1.364526 in", F_e = "1958.853 lbf" }
tolerance = "0.1%"

[[tests]]
name = "Suddenly applied load (h = 0): the impact factor is 2"
source = "Lacewing Lab impact_loading_test, after Juvinall Eq. 7.2: 1 + √(1 + 0) = 2"
inputs = { W = "100 N", h = "0 m", k_st = "100000 N/m", support = "0" }
expect = { IF = "2", "δ" = "2 mm", F_e = "200 N" }
tolerance = "1e-9"

[[tests]]
name = "Hand check: 100 N dropped 100 mm onto a 100 kN/m spring"
source = "By hand: δst = 1 mm, IF = 1 + √201 = 15.177447, δ = 15.177447 mm, Fe = 1 517.7447 N, U = 100 × (0.1 + 0.015177447) = 11.517745 J, τ = 2π√((100/9.80665)/1e5) = 0.063448233 s"
inputs = { W = "100 N", h = "100 mm", k_st = "100000 N/m", support = "0" }
expect = { "δ_st" = "1 mm", IF = "15.177447", "δ" = "15.177447 mm", F_e = "1517.7447 N", U = "11.517745 J", "τ_n" = "0.063448233 s" }
tolerance = "1e-7"
+++

## Falling weight on an elastic structure

A weight W falls a height h onto an elastic structure (Juvinall Sec. 7.2, Fig. 7.3). Equating the work of the weight, W(h + δ), to the elastic energy ½kδ² gives the impact deflection as the static deflection times the impact factor 1 + √(1 + 2h/δst) (Eqs. 7.1, 7.2). A load applied suddenly (h = 0) deflects the structure twice as far as the same load applied slowly. The equivalent static force F_e = kδ is the static load that would give the same deflection; the stresses follow from it as for a static load.

The structure's rate k is the force per unit deflection at the point struck: AE/L for a bar in tension or compression, 48EI/L³ for a simply supported beam loaded at mid-span, 3EI/L³ for a cantilever loaded at its end. For a beam with its bending stress, use mechanical.impactLoading.beam.

The structure is taken as massless and elastic, the weight as rigid, and damping is neglected (Juvinall p. 290). The stresses are then underestimated and the energy capacity overestimated, more so for struck parts of large mass.

### Weight and structure

```calc {description="Falling weight (m·g)"}
W = 100 lbf_
```

```calc {description="Height of the fall before contact (0: a suddenly applied load)"}
h = 12 in_
```

```calc {description="Rate of the struck structure at the point of impact (AE/L, 48EI/L³, 3EI/L³)"}
k_st = 1436 lbf/in_
```

A spring under the structure, in series with it, softens the impact. Enter 1 to include it, 0 for a structure on rigid supports.

```calc {description="Supporting spring in series: 1 included, 0 none (rigid supports)"}
support = 1_
```

```calc {description="Rate of the supporting spring (all the supports together)"}
k_sup = 200 lbf/in_
```

```calc {description="Standard gravity"}
g_n = 9.80665 m/s²
```

### Impact

```calc
> Rate of the system (structure and supporting spring in series)
k = {k_st if support == 0; 1 / (1 / k_st + 1 / k_sup) if support == 1} = ?? lbf/in
> Static deflection: where the weight would come to rest
δ_st = W / k = ?? in
> Impact factor, Eqs. 7.1 and 7.2
IF = 1 + sqrt(1 + 2 h / δ_st) = ??
> Impact deflection
δ = IF δ_st = ?? in
> Equivalent static force
F_e = k δ = ?? lbf
> Energy absorbed: the work of the falling weight
U = W (h + δ) = ?? in·lbf
> Deflection of the structure itself (the rest is in the supporting spring)
δ_struct = F_e / k_st = ?? in
```

### Impact or static?

Juvinall's Table 7.1 compares the time the load takes to come on with the natural period of the weight on the structure: in less than half the period it is an impact, in more than three periods it is effectively static, and between the two it is a grey area.

```calc {description="Natural period of the weight on the structure, Eq. a"}
τ_n = 2 π sqrt(W / (g_n k)) = ?? s
```
