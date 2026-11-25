+++
cruncha-template = 1
id = "mechanical.extensionSprings.fatigue"
version = "1.0.0"
title = "Helical extension spring with hook ends in fatigue (Shigley, Gerber-Zimmerli)"
summary = "A close-wound helical extension spring with full-loop hook ends cycled between F_min and F_max: alternating and mean stresses in the body, in the hook in bending (point A) and in the hook in torsion (point B), and their Gerber fatigue factors of safety with Zimmerli's endurance data, checked against a design factor."
category = ["Mechanical engineering", "Springs"]
tags = ["spring", "extension spring", "tension spring", "helical spring", "coil spring", "hook", "fatigue", "Gerber", "Zimmerli", "Bergsträsser", "Shigley"]
from = "lacewing:mechanical.extensionSprings"
license = "GPL-3.0-or-later"
inputs = ["d", "OD", "r_1", "r_2", "A", "m", "d_A", "S_sa", "S_sm", "F_max", "F_min", "n_d"]
outputs = ["D", "C", "K_B", "S_ut", "S_su", "S_se", "S_e", "F_a", "F_m", "τ_a", "τ_m", "n_(f,body)", "K_hookA", "σ_a", "σ_m", "n_(f,A)", "K_hookB", "τ_(a,B)", "τ_(m,B)", "n_(f,B)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 10-11: Eqs. 10-34 to 10-37 (hook stresses at A and B); Sec. 10-9: Eqs. 10-28 (Zimmerli's unpeened endurance components), 10-30 (Ssu); Table 6-7 (Gerber); Eqs. 10-1, 10-5, 10-14 with Table 10-4; Example 10-7"

[[tests]]
name = "Shigley Ex. 10-7: the Ex. 10-6 spring cycled 1.5 to 5 lbf"
source = "Lacewing Lab coil_springs_test, after Shigley 10th Example 10-7: unpeened Zimmerli data, Gerber: nf = 1.24 in the body, 1.08 at hook A in bending, 1.30 at hook B in torsion"
inputs = { d = "0.035 in", OD = "0.248 in", r_1 = "0.106 in", r_2 = "0.089 in", A = "140 kpsi", m = "0.190", d_A = "1 in", S_sa = "35 kpsi", S_sm = "55 kpsi", F_max = "5 lbf", F_min = "1.5 lbf", n_d = "1" }
expect = { "n_(f,body)" = "1.24", "n_(f,A)" = "1.08", "n_(f,B)" = "1.30" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 10-7 exactly"
source = "By hand: Sut = 264.70071 kpsi, Ssu = 177.34948 kpsi, Sse = 35/(1 − (55/177.34948)²) = 38.724342 kpsi, Se = Sse/0.577 = 67.113245 kpsi; Fa = 1.75, Fm = 3.25 lbf; τa = 27.325238, τm = 50.746870 kpsi, nf body = 1.2390314; σa = 52.301686, σm = 97.131703 kpsi, nf A = 1.0812080 (Gerber on Sut and Se); (τa)B = 26.202714, (τm)B = 48.662183 kpsi, nf B = 1.2921115"
inputs = { d = "0.035 in", OD = "0.248 in", r_1 = "0.106 in", r_2 = "0.089 in", A = "140 kpsi", m = "0.190", d_A = "1 in", S_sa = "35 kpsi", S_sm = "55 kpsi", F_max = "5 lbf", F_min = "1.5 lbf", n_d = "1" }
expect = { S_ut = "264.70071 kpsi", S_su = "177.34948 kpsi", S_se = "38.724342 kpsi", S_e = "67.113245 kpsi", F_a = "1.75 lbf", F_m = "3.25 lbf", "τ_a" = "27.325238 kpsi", "τ_m" = "50.746870 kpsi", "n_(f,body)" = "1.2390314", "σ_a" = "52.301686 kpsi", "σ_m" = "97.131703 kpsi", "n_(f,A)" = "1.0812080", "τ_(a,B)" = "26.202714 kpsi", "τ_(m,B)" = "48.662183 kpsi", "n_(f,B)" = "1.2921115" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 10-7 against a design factor of 1.2"
source = "As Shigley 10th Example 10-7: the hook in bending at A, 1.08, is under 1.2"
inputs = { d = "0.035 in", OD = "0.248 in", r_1 = "0.106 in", r_2 = "0.089 in", A = "140 kpsi", m = "0.190", d_A = "1 in", S_sa = "35 kpsi", S_sm = "55 kpsi", F_max = "5 lbf", F_min = "1.5 lbf", n_d = "1.2" }
expect = { "n_(f,A)" = "1.08" }
tolerance = "0.5%"
checks = "fail"
+++

## Helical extension spring with hook ends in fatigue

A close-wound extension spring with a full loop at each end whose load cycles between F_min and F_max, after Shigley Secs. 10-9 and 10-11. Three places are checked: the body in torsion, the hook at A in bending plus direct tension, and the hook at B in torsion (see the extension spring static template for the geometry of A and B). All three use the Gerber criterion with Zimmerli's endurance data, as Shigley does. For bending at A, the torsional endurance strength is converted to a bending one by the distortion-energy ratio, S_e = S_se/0.577, and S_ut takes the place of S_su. Shigley's example uses the unpeened data. The rate, free length and static checks are in the static template.

### Wire

S_ut = A/d^m (Eq. 10-14) with A and m from Shigley Table 10-4, A for d in the unit d_A. Zimmerli's components (Eq. 10-28) are 35 and 55 kpsi (241 and 379 MPa) unpeened.

```calc {description="Wire diameter"}
d = 0.035 in_
```

```calc {description="Outside diameter of the coil"}
OD = 0.248 in_
```

```calc {description="Table 10-4 constant A of Sut = A/d^m, for d in units of d_A"}
A = 140 kpsi_
```

```calc {description="Table 10-4 exponent m of Sut = A/d^m"}
m = 0.190_
```

```calc {description="Diameter unit Table 10-4's A is for (1 in for kpsi·in^m, 1 mm for MPa·mm^m)"}
d_A = 1 in_
```

```calc {description="Zimmerli's alternating endurance component, Shigley Eq. 10-28 (unpeened 35 kpsi)"}
S_sa = 35 kpsi_
```

```calc {description="Zimmerli's mean endurance component, Shigley Eq. 10-28 (unpeened 55 kpsi)"}
S_sm = 55 kpsi_
```

```calc
> Mean coil diameter
D = OD - d = ?? in
> Spring index, Eq. 10-1
C = D / d = ??
> Bergsträsser factor, Eq. 10-5
K_B = (4 C + 2) / (4 C - 3) = ??
> Minimum tensile strength, Eq. 10-14
S_ut = A / (d / d_A)^m = ?? kpsi
> Torsional ultimate strength, Eq. 10-30
S_su = 0.67 S_ut = ?? kpsi
> Fully reversed torsional endurance strength, Gerber through Zimmerli's point
S_se = S_sa / (1 - (S_sm / S_su)^2) = ?? kpsi
> Fully reversed bending endurance strength, for the hook at A
S_e = S_se / 0.577 = ?? kpsi
```

The Gerber factor of safety on the load line through the origin, for an alternating stress a, a mean stress m, an ultimate strength S_u and an endurance strength S_e (Shigley Table 6-7):

```calc
function gerberFactor(a, m, S_u, S_e)
    return 0.5 (S_u / m)^2 (a / S_e) (-1 + sqrt(1 + (2 m S_e / (S_u a))^2))
end
```

### Loads

```calc {description="Largest load of the cycle"}
F_max = 5 lbf_
```

```calc {description="Smallest load of the cycle (at least the initial tension)"}
F_min = 1.5 lbf_
```

```calc
> Alternating force
F_a = (F_max - F_min) / 2 = ?? lbf
> Mean force
F_m = (F_max + F_min) / 2 = ?? lbf
```

### Body

```calc
> Alternating shear stress
τ_a = K_B 8 F_a D / (π d^3) = ?? kpsi
> Mean shear stress
τ_m = K_B 8 F_m D / (π d^3) = ?? kpsi
> Fatigue factor of safety of the body, Gerber
n_(f,body) = gerberFactor(τ_a, τ_m, S_su, S_se) = ??
```

### Hook at A, bending

```calc {description="Mean radius of the hook bend at A (bending)"}
r_1 = 0.106 in_
```

```calc
> Bending stress-correction factor at A, Eq. 10-35
K_hookA = (4 (2 r_1 / d)^2 - 2 r_1 / d - 1) / (4 (2 r_1 / d) (2 r_1 / d - 1)) = ??
> Alternating stress at A, Eq. 10-34
σ_a = F_a (K_hookA 16 D / (π d^3) + 4 / (π d^2)) = ?? kpsi
> Mean stress at A, Eq. 10-34
σ_m = F_m (K_hookA 16 D / (π d^3) + 4 / (π d^2)) = ?? kpsi
> Fatigue factor of safety at A, Gerber
n_(f,A) = gerberFactor(σ_a, σ_m, S_ut, S_e) = ??
```

### Hook at B, torsion

```calc {description="Mean radius of the hook bend at B (torsion)"}
r_2 = 0.089 in_
```

```calc
> Torsional stress-correction factor at B, Eq. 10-37
K_hookB = (4 (2 r_2 / d) - 1) / (4 (2 r_2 / d) - 4) = ??
> Alternating shear stress at B, Eq. 10-36
τ_(a,B) = K_hookB 8 F_a D / (π d^3) = ?? kpsi
> Mean shear stress at B, Eq. 10-36
τ_(m,B) = K_hookB 8 F_m D / (π d^3) = ?? kpsi
> Fatigue factor of safety at B, Gerber
n_(f,B) = gerberFactor(τ_(a,B), τ_(m,B), S_su, S_se) = ??
```

### Checks

```calc {description="Design factor required against fatigue"}
n_d = 1_
```

```calc
@test n_(f,body) ≥ n_d
```

```calc
@test n_(f,A) ≥ n_d
```

```calc
@test n_(f,B) ≥ n_d
```
