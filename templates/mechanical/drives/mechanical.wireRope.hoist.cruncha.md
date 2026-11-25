+++
cruncha-template = 1
id = "mechanical.wireRope.hoist"
version = "1.0.0"
title = "Wire rope on a hoist: tension, bending and safety factors (Shigley)"
summary = "A hoist on one or more wire ropes over a sheave: the largest rope tension with the rope's own weight and the acceleration, the rope's breaking strength, the equivalent bending load over the sheave, the allowable fatigue tension for the life chosen, the static and fatigue factors of safety, the bearing pressure on the sheave against its allowable, and the wire strength a long life needs."
category = ["Mechanical engineering", "Belts and chains"]
tags = ["wire rope", "rope", "cable", "hoist", "sheave", "mine hoist", "elevator", "crane", "bending load", "fatigue", "bearing pressure", "factor of safety", "Shigley"]
from = "lacewing:mechanical.wireRope"
license = "GPL-3.0-or-later"
inputs = ["d", "D", "W", "m", "l", "a", "c_w", "S_(u,r)", "k_w", "k_A", "E_r", "r_p", "S_u", "p_allow", "r_D", "n_(s,min)"]
outputs = ["w", "F_t", "F_u", "d_w", "A_m", "F_b", "F_f", "n_s", "n_su", "n_f", "p", "S_(u,long)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 17-6: Eqs. 17-41 (bending load), 17-42 (bearing pressure), 17-43 (wire strength for long life), 17-44 (fatigue tension), 17-45, 17-46 (factors of safety), 17-47 (rope tension); Tables 17-24 (rope data), 17-25 (minimum factors of safety), 17-26 (sheave pressures), 17-27 (wire size, metal area, modulus); Fig. 17-21 (p/Su against bends); Example 17-6"

[[tests]]
name = "Shigley Ex. 17-6: 1-ton cage on 531.5 ft of 6 × 19 monitor steel rope over a 72 in sheave, three ropes of 5/8 in"
source = "Lacewing Lab wire_rope_test, after Shigley 10th Example 17-6 and its table of nf by rope size and number of ropes: a = 2 ft/s², p/Su = 0.0014, Su = 240 kpsi; d = 5/8 in with three ropes, nf = 6.099 (the book takes g = 32.2 ft/s²)"
inputs = { d = "0.625 in", D = "72 in", W = "2000 lbf", m = "3", l = "531.5 ft", a = "2 ft/s²", c_w = "1.60 lbf/(ft·in²)", "S_(u,r)" = "106 kpsi", k_w = "0.067", k_A = "0.40", E_r = "12 Mpsi", r_p = "0.0014", S_u = "240 kpsi", p_allow = "480 psi", r_D = "26", "n_(s,min)" = "5" }
expect = { n_f = "6.099" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 17-6: the expressions at d = 1 in, one rope"
source = "Lacewing Lab wire_rope_test, after Shigley 10th Example 17-6: Ft = 2124/m + 903d² = 3027 lbf, Ff = 12 096d = 12 096 lbf, Fb = 4467d³ = 4467 lbf; the book's table gives nf = 2.520"
inputs = { d = "1 in", D = "72 in", W = "2000 lbf", m = "1", l = "531.5 ft", a = "2 ft/s²", c_w = "1.60 lbf/(ft·in²)", "S_(u,r)" = "106 kpsi", k_w = "0.067", k_A = "0.40", E_r = "12 Mpsi", r_p = "0.0014", S_u = "240 kpsi", p_allow = "480 psi", r_D = "26", "n_(s,min)" = "5" }
expect = { F_t = "3027 lbf", F_f = "12096 lbf", F_b = "4467 lbf", n_f = "2.520" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 17-6, three 5/8 in ropes, exactly"
source = "By hand with g = 9.80665 m/s² (32.174049 ft/s²): w = 1.6 × 0.625² = 0.625 lbf/ft, Ft = (2000/3 + 0.625 × 531.5)(1 + 2/32.174049) = 1060.9448 lbf, Fu = 106 000 × π × 0.625²/4 = 32 520.393 lbf, dw = 0.041875 in, Am = 0.15625 in², Fb = 12e6 × 0.041875 × 0.15625/72 = 1090.4948 lbf, Ff = 0.0014 × 240 000 × 72 × 0.625/2 = 7560 lbf, ns = 29.624441, Fu/Ft = 30.652294, nf = 6.0978714, p = 2 × 1060.9448/(0.625 × 72) = 47.153104 psi, Su for long life = 47 153.104 psi"
inputs = { d = "0.625 in", D = "72 in", W = "2000 lbf", m = "3", l = "531.5 ft", a = "2 ft/s²", c_w = "1.60 lbf/(ft·in²)", "S_(u,r)" = "106 kpsi", k_w = "0.067", k_A = "0.40", E_r = "12 Mpsi", r_p = "0.0014", S_u = "240 kpsi", p_allow = "480 psi", r_D = "26", "n_(s,min)" = "5" }
expect = { w = "0.625 lbf/ft", F_t = "1060.9448 lbf", F_u = "32520.393 lbf", d_w = "0.041875 in", A_m = "0.15625 in²", F_b = "1090.4948 lbf", F_f = "7560 lbf", n_s = "29.624441", n_su = "30.652294", n_f = "6.0978714", p = "47.153104 psi", "S_(u,long)" = "47153.104 psi" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "One 1 in rope over a 20 in sheave: far too small a sheave"
source = "By hand: D/d = 20, under Table 17-24's 26 for 6 × 19; Fb = 12e6 × 0.067 × 0.40/20 = 16 080 lbf, Ff = 0.0014 × 240 000 × 20 × 1/2 = 3360 lbf, so nf = (3360 − 16 080)/3027.5863 = −4.2014"
inputs = { d = "1 in", D = "20 in", W = "2000 lbf", m = "1", l = "531.5 ft", a = "2 ft/s²", c_w = "1.60 lbf/(ft·in²)", "S_(u,r)" = "106 kpsi", k_w = "0.067", k_A = "0.40", E_r = "12 Mpsi", r_p = "0.0014", S_u = "240 kpsi", p_allow = "480 psi", r_D = "26", "n_(s,min)" = "5" }
expect = { F_b = "16080 lbf", F_f = "3360 lbf" }
tolerance = "1e-6"
checks = "fail"
+++

## Wire rope on a hoist

A load W (a cage or skip and its contents) hung on m ropes, each passing over a sheave or drum, accelerated at up to a, after Shigley's Sec. 17-6 and the mine hoist of Example 17-6. The largest tension in a rope is at the sheave: its share of the load plus the weight of the longest hanging length l, raised by the acceleration (Eq. 17-47). The rope is checked two ways. Statically, its breaking strength less the equivalent bending load over the sheave against that tension (Eq. 17-46), compared with the minimum factor of safety codes ask for (Table 17-25). In fatigue, the allowable tension for the number of bends wanted, from the ratio of bearing pressure to wire strength p/S_u read off Fig. 17-21, less the bending load, against the tension (Eq. 17-45). Some codes define the static factor as F_u/F_t, which is also shown.

The rope data are inputs from the tables: the weight per foot as a coefficient times d² and the rope's strength on its nominal area (Table 17-24, approximate), the wire diameter and metal area as fractions of d and d² and the rope's modulus (Table 17-27; Ex. 17-6 takes 12 Mpsi for 6 × 19), the smallest sheave as a multiple of d (Table 17-24) and the allowable bearing pressure for the sheave material and lay (Table 17-26). S_u is the strength of the wires, not of the rope: about 240 to 280 kpsi for improved plow (monitor) steel, 210 to 240 for plow steel and 180 to 210 for mild plow steel (p. 912). The rope's weight is turned into a mass with standard gravity, 9.80665 m/s².

### Hoist

```calc {description="Nominal rope diameter"}
d = 0.625 in_
```

```calc {description="Sheave or drum diameter"}
D = 72 in_
```

```calc {description="Load at the end of the ropes (cage and contents)"}
W = 2000 lbf_
```

```calc {description="Number of ropes sharing the load"}
m = 3_
```

```calc {description="Longest suspended length of rope"}
l = 531.5 ft_
```

```calc {description="Largest acceleration or deceleration"}
a = 2 ft/s²_
```

### Rope

```calc {description="Weight per foot over d², Shigley Table 17-24 (1.60 for 6 × 19)"}
c_w = 1.60 lbf/(ft·in²)_
```

```calc {description="Rope strength on the nominal area, Shigley Table 17-24"}
S_(u,r) = 106 kpsi_
```

```calc {description="Wire diameter over rope diameter, Shigley Table 17-27"}
k_w = 0.067_
```

```calc {description="Metal area over d², Shigley Table 17-27"}
k_A = 0.40_
```

```calc {description="Modulus of the rope, Shigley Table 17-27"}
E_r = 12 Mpsi_
```

```calc {description="Smallest sheave diameter over rope diameter, Shigley Table 17-24"}
r_D = 26_
```

### Fatigue and sheave

```calc {description="Pressure-strength ratio p/Su for the bends wanted, Shigley Fig. 17-21"}
r_p = 0.0014_
```

```calc {description="Ultimate strength of the wires"}
S_u = 240 kpsi_
```

```calc {description="Allowable bearing pressure of the rope on the sheave, Shigley Table 17-26"}
p_allow = 480 psi_
```

```calc {description="Minimum static factor of safety, Shigley Table 17-25"}
n_(s,min) = 5_
```

### Tension and strength

```calc
> Rope weight per length, Table 17-24
w = c_w d^2 = ?? lbf/ft
> Largest tension in one rope, Eq. 17-47
F_t = (W / m + w l) (1 + a / (9.80665 'm/s²')) = ?? lbf
> Breaking strength on the nominal area, Table 17-24
F_u = S_(u,r) π d^2 / 4 = ?? lbf
> Wire diameter, Table 17-27
d_w = k_w d = ?? in
> Metal area, Table 17-27
A_m = k_A d^2 = ?? in²
> Equivalent bending load over the sheave, Eq. 17-41
F_b = E_r d_w A_m / D = ?? lbf
> Allowable fatigue tension for the life chosen, Eq. 17-44
F_f = r_p S_u D d / 2 = ?? lbf
```

### Factors of safety and sheave pressure

```calc
> Static factor of safety, Eq. 17-46
n_s = (F_u - F_b) / F_t = ??
> Static factor as some codes define it
n_su = F_u / F_t = ??
> Fatigue factor of safety, Eq. 17-45
n_f = (F_f - F_b) / F_t = ??
> Bearing pressure on the sheave, Eq. 17-42
p = 2 F_t / (d D) = ?? psi
> Wire strength for a long life (p/Su = 0.001), Eq. 17-43
S_(u,long) = 2000 F_t / (d D) = ?? psi
```

### Checks

The rope must reach the life chosen (n_f at least 1), meet the code's static factor, bear on the sheave within its allowable pressure, and run on a sheave no smaller than Table 17-24's.

```calc
@test n_f ≥ 1
```

```calc
@test n_s ≥ n_(s,min)
```

```calc
@test p ≤ p_allow
```

```calc
@test D / d ≥ r_D
```
