+++
cruncha-template = 1
id = "mechanical.impactLoading.moving"
version = "1.0.0"
title = "Moving weight striking an elastic structure (energy method)"
summary = "Juvinall's energy method for a weight moving at a known speed into an elastic structure of known rate, with gravity doing no further work (a horizontal impact): the kinetic energy, the impact deflection √(2U/k), the equivalent static force √(2Uk), the impact factor against the weight resting on it, and the natural period."
category = ["Mechanical engineering", "Mechanics"]
tags = ["impact", "impact factor", "moving weight", "horizontal impact", "bumper", "buffer", "kinetic energy", "energy method", "equivalent static force", "shock load", "Juvinall"]
from = "lacewing:mechanical.impactLoading"
license = "GPL-3.0-or-later"
inputs = ["W", "v", "k"]
outputs = ["U", "δ", "F_e", "δ_st", "IF", "τ_n"]

[[references]]
work = "Juvinall, R.C., Marshek, K.M., Fundamentals of Machine Component Design"
edition = "5th"
locator = "Ch. 7, Sec. 7.2: Eq. g (kinetic energy), Eqs. 7.3a, 7.3b and 7.4b (moving weight); Table 7.1 and Eq. a (natural period)"

[[tests]]
name = "Lacewing Lab: 100 N at 2 m/s into a 100 kN/m spring"
source = "Lacewing Lab impact_loading_test (Eqs. 7.3b, 7.4b): U = ½(100/9.80665) × 2² = 20.394324 J, Fe = √(2Uk) = 2 019.6200 N, τ = 2π√(m/k) = 0.063448233 s. By hand: δ = √(2U/k) = 20.196200 mm, δst = 1 mm, IF = 20.196200"
inputs = { W = "100 N", v = "2 m/s", k = "100000 N/m" }
expect = { U = "20.394324 J", "δ" = "20.196200 mm", F_e = "2019.6200 N", IF = "20.196200", "τ_n" = "0.063448233 s" }
tolerance = "1e-7"

[[tests]]
name = "Hand check: 50 N at 3 m/s into the end of a steel bar, A = 100 mm², L = 1 m"
source = "Lacewing Lab impact_loading_test (Eq. 7.5, σ = √(2UE/V)). By hand: k = AE/L = 1e-4 × 200e9/1 = 2e7 N/m, U = ½(50/9.80665) × 9 = 22.943615 J, Fe = √(2 × 22.943615 × 2e7) = 30 294.300 N, so σ = Fe/A = 302.943 MPa = √(2UE/V)"
inputs = { W = "50 N", v = "3 m/s", k = "20000 kN/m" }
expect = { U = "22.943615 J", F_e = "30294.300 N", "δ" = "1.5147150 mm" }
tolerance = "1e-7"
+++

## Moving weight striking an elastic structure

A weight W moving at speed v strikes an elastic structure, and gravity does no further work: a horizontal impact, or a vertical one where the weight's further fall is negligible (Juvinall Sec. 7.2, Eqs. 7.3b and 7.4b). The kinetic energy ½mv² is taken up as elastic energy ½kδ², which gives the impact deflection and the equivalent static force F_e = kδ, the static load that would give the same deflection; the stresses follow from it as for a static load (F_e/A for a bar). The impact factor compares δ with the deflection under the weight at rest on the structure (Eq. 7.3a).

The structure's rate k is the force per unit deflection at the point struck: AE/L for a bar along its axis, 48EI/L³ for a simply supported beam struck at mid-span, 3EI/L³ for a cantilever struck at its end. For a bar, the stress √(2UE/V) depends on its volume (Eq. 7.5): a longer bar of the same section takes the same energy at a lower stress.

The structure is taken as massless and elastic, the weight as rigid, and damping is neglected (Juvinall p. 290). The stresses are then underestimated and the energy capacity overestimated, more so for struck parts of large mass.

```calc {description="Weight of the moving body (m·g)"}
W = 100 N_
```

```calc {description="Speed of the body at contact"}
v = 2 m/s_
```

```calc {description="Rate of the struck structure at the point of impact (AE/L, 48EI/L³, 3EI/L³)"}
k = 100 kN/m_
```

```calc {description="Standard gravity"}
g_n = 9.80665 m/s²
```

### Impact

```calc
> Kinetic energy at contact, Eq. g
U = W v^2 / (2 g_n) = ?? J
> Impact deflection, Eq. 7.3b
δ = sqrt(2 U / k) = ?? mm
> Equivalent static force, Eq. 7.4b
F_e = sqrt(2 U k) = ?? N
> Static deflection under the weight at rest
δ_st = W / k = ?? mm
> Impact factor, Eq. 7.3a
IF = δ / δ_st = ??
```

### Impact or static?

Juvinall's Table 7.1 compares the time the load takes to come on with the natural period of the weight on the structure: in less than half the period it is an impact, in more than three periods it is effectively static.

```calc {description="Natural period of the weight on the structure, Eq. a"}
τ_n = 2 π sqrt(W / (g_n k)) = ?? s
```
