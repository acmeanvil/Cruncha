+++
cruncha-template = 1
id = "mechanical.impactLoading.energyCapacity"
version = "1.0.0"
title = "Impact energy capacity of a stepped bar with stress raisers"
summary = "The elastic energy a bar of up to three lengths can take in axial impact before its most severely stressed point reaches the strength: the force limit min(S·A/K_i), the energy P²·Σ(L/A)/(2E), the energy of the same volume all stressed to S, the share of that ideal, and the modulus of resilience."
category = ["Mechanical engineering", "Mechanics"]
tags = ["impact", "energy capacity", "energy absorption", "stress concentration", "impact stress factor", "bolt", "stepped bar", "modulus of resilience", "resilience", "shock load", "Juvinall"]
from = "lacewing:mechanical.impactLoading"
license = "GPL-3.0-or-later"
inputs = ["S", "E", "A_1", "L_1", "K_1", "A_2", "L_2", "K_2", "A_3", "L_3", "K_3"]
outputs = ["P", "U", "V", "U_ideal", "η", "u", "u_R"]

[[references]]
work = "Juvinall, R.C., Marshek, K.M., Fundamentals of Machine Component Design"
edition = "5th"
locator = "Ch. 7, Secs. 7.2 and 7.4: Eqs. 7.5 and 7.5a (energy in a bar); Sample Problems 7.1 (stepped rod), 7.2 (modulus of resilience), 7.5 to 7.7 (stress raisers, the bolt redesign)"

[[tests]]
name = "Juvinall Sample Problem 7.7: the original bolt"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.7: S = 1000 MPa, E = 200 GPa; shank 700 mm² with K_i 3.4 at the head fillet, thread root 600 mm² (negligible length) at K_i 3.5. The shank length 100 mm is taken for the absolute values (the book compares the two designs). By hand: P = 1000 × 600/3.5 = 171 428.57 N, U = 171 428.57² × (0.1/700e-6)/(2 × 200e9) = 10.495627 J"
inputs = { S = "1000 MPa", E = "200 GPa", A_1 = "700 mm^2", L_1 = "100 mm", K_1 = "3.4", A_2 = "600 mm^2", L_2 = "0 mm", K_2 = "3.5", A_3 = "700 mm^2", L_3 = "0 mm", K_3 = "1" }
expect = { P = "171428.57 N", U = "10.495627 J" }
tolerance = "1e-7"

[[tests]]
name = "Juvinall Sample Problem 7.7: the redesigned bolt takes 3.18 times the energy"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.7: rolled thread (K_i 3.0) and the shank turned down to 300 mm² with a K_i 1.5 flare; the book's ratio 3.18 to the original. By hand: P = 200 000 N, U = 200 000² × (0.1/300e-6)/(4e11) = 33.333333 J = 3.176 × 10.495627 J"
inputs = { S = "1000 MPa", E = "200 GPa", A_1 = "300 mm^2", L_1 = "100 mm", K_1 = "1.5", A_2 = "600 mm^2", L_2 = "0 mm", K_2 = "3.0", A_3 = "300 mm^2", L_3 = "0 mm", K_3 = "1" }
expect = { P = "200000 N", U = "33.333333 J" }
tolerance = "1e-7"

[[tests]]
name = "Juvinall Sample Problem 7.1: the stepped rod holds 5/8 of the plain rod"
source = "Lacewing Lab impact_loading_test, after Juvinall 5th Sample Problem 7.1: half the length at area A, half at 4A, no stress raisers, takes 5/8 of the energy of a plain rod of area A (S²AL/2E = 22.5 J here) with 2.5 times the volume: a quarter of the ideal. S = 300 MPa, E = 200 GPa, A = 100 mm², L = 1 m"
inputs = { S = "300 MPa", E = "200 GPa", A_1 = "100 mm^2", L_1 = "500 mm", K_1 = "1", A_2 = "400 mm^2", L_2 = "500 mm", K_2 = "1", A_3 = "100 mm^2", L_3 = "0 mm", K_3 = "1" }
expect = { P = "30000 N", U = "14.0625 J", V = "250000 mm^3", U_ideal = "56.25 J", "η" = "0.25", u_R = "225 kJ/m^3" }
tolerance = "1e-9"
+++

## Impact energy capacity of a bar

A bar loaded in axial impact (a bolt, a link, a stepped rod) absorbs the energy as elastic strain energy. The force is the same along the bar, so a length of area A carries a stress P/A and stores P²L/(2AE). Each length's capacity is reached when its most severe point, with its impact stress factor K_i, reaches the strength S: the bar's force limit is the least of S·A/K_i, and every length is stressed to P/A at that force (Juvinall Sec. 7.4, Eq. 7.5a summed over the lengths; the local volumes of the fillets are neglected).

The ideal is the same volume all stressed to S, S²V/(2E). A bar takes the most energy for its weight when it is all at nearly the same stress: turning a bolt's shank down to the thread's root area raises its capacity (Sample Problem 7.7), while enlarging part of a bar lowers it (Sample Problem 7.1).

Stress raisers act under impact at nearly their full theoretical value (Sec. 7.4); 1 means none. Take S as the elastic limit (the yield strength) of a ductile part. For a notch-brittle part, use the strength with its K_i. Give a short critical section, such as a thread root, a length of 0, and an unused length a length of 0, K_i of 1 and an area no smaller than the others'.

### Material

```calc {description="Strength the most severe point may reach (the yield strength as the elastic limit)"}
S = 1000 MPa_
```

```calc {description="Elastic modulus"}
E = 200 GPa_
```

### Lengths of the bar

```calc {description="Area of length 1"}
A_1 = 300 mm²_
```

```calc {description="Length of length 1"}
L_1 = 100 mm_
```

```calc {description="Impact stress factor at length 1's most severe point (1 for none)"}
K_1 = 1.5_
```

```calc {description="Area of length 2"}
A_2 = 600 mm²_
```

```calc {description="Length of length 2 (0 for a short critical section such as a thread root)"}
L_2 = 0 mm_
```

```calc {description="Impact stress factor at length 2's most severe point (1 for none)"}
K_2 = 3.0_
```

```calc {description="Area of length 3 (unused: no smaller than the others)"}
A_3 = 300 mm²_
```

```calc {description="Length of length 3 (0 if unused)"}
L_3 = 0 mm_
```

```calc {description="Impact stress factor at length 3's most severe point (1 for none)"}
K_3 = 1_
```

### Energy capacity

```calc
> Force at which the most severe point reaches S
P = min(S A_1 / K_1, S A_2 / K_2, S A_3 / K_3) = ?? kN
> Energy capacity, Eq. 7.5a summed over the lengths
U = P^2 (L_1 / A_1 + L_2 / A_2 + L_3 / A_3) / (2 E) = ?? J
> Volume of the bar
V = A_1 L_1 + A_2 L_2 + A_3 L_3 = ?? mm³
> Ideal: the same volume all stressed to S
U_ideal = S^2 V / (2 E) = ?? J
> Share of the ideal
η = U / U_ideal = ??
> Energy per unit volume
u = U / V = ?? kJ/m³
> Modulus of resilience of the material, Sample Problem 7.2
u_R = S^2 / (2 E) = ?? kJ/m³
```
