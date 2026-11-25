+++
cruncha-template = 1
id = "mechanical.bevelGears.rating"
version = "1.0.0"
title = "Straight bevel gear rating, AGMA 2003 (Shigley)"
summary = "The AGMA rating of a straight (Coniflex) bevel gear pair as Shigley ch. 15 gives ANSI/AGMA 2003-B97: transmitted load at the outer pitch circle, dynamic, size, load-distribution, crowning and size-for-pitting factors, the contact stress and each member's bending stress, their allowables at the life, reliability and blank temperature set, the factors of safety, and the power the pair can carry at the design factors in bending and in wear."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "bevel gear", "straight bevel", "Coniflex", "AGMA", "AGMA 2003", "gear rating", "bending stress", "contact stress", "pitting", "wear", "factor of safety", "rated power", "Shigley"]
from = "lacewing:mechanical.bevelGears"
license = "GPL-3.0-or-later"
inputs = ["m", "N_P", "N_G", "Σ", "F", "H", "n_P", "Q_v", "K_o", "K_mb", "C_xc", "I", "J_P", "J_G", "E_P", "ν_P", "E_G", "ν_G", "s_(at,P)", "s_(at,G)", "s_(ac,P)", "s_(ac,G)", "H_(B,P)", "H_(B,G)", "R", "N_L", "critical", "θ", "n_F", "n_H"]
outputs = ["m_G", "γ", "d_P", "A_0", "F_max", "v_t", "W_t", "K_v", "v_max", "K_s", "C_s", "K_m", "C_p", "K_R", "C_R", "K_T", "C_H", "σ_c", "σ_(t,P)", "σ_(t,G)", "K_(L,P)", "K_(L,G)", "C_(L,P)", "C_(L,G)", "s_(wt,P)", "s_(wt,G)", "s_(wc,P)", "s_(wc,G)", "S_(F,P)", "S_(F,G)", "S_(H,P)", "S_(H,G)", "H_b", "H_w", "H_r"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 15-2 to 15-5: Eqs. 15-1 to 15-25 (stresses, allowables, Kv, Ks, Cs, Km, Cxc, CH, KL, CL, KT, KR, CR, Cp, face width, cone distance); Tables 15-2 (Ko), 15-3 (KR), 15-4 to 15-7 (strengths); Figs. 15-6 and 15-7 (I, J), 15-12 and 15-13 (through-hardened strengths); Examples 15-1 and 15-2"

[[references]]
work = "ANSI/AGMA 2003-B97, Rating the Pitting Resistance and Bending Strength of Generated Straight Bevel, Zerol Bevel and Spiral Bevel Gear Teeth"
edition = "1997"
locator = "as presented in Shigley ch. 15"

[[tests]]
name = "Shigley Ex. 15-1 (a), (b): 25-tooth miters, P 5, 1.10 in face, 600 rev/min, grade 1 steel 180 HB"
source = "Shigley 10th Example 15-1 (also Lacewing Lab bevel_test): Qv 7 gives Kv = 1.299; with I = 0.065 and J = 0.216 (Figs. 15-6, 15-7), neither member straddle-mounted (Kmb 1.25), uncrowned, R 0.99, 10⁷ cycles on the critical curve, sat = 44 HB + 2100 = 10 020 psi and sac = 341 HB + 23 620 = 85 000 psi (Eqs. 15-22, 15-23), SF = SH = 1: 13.2 hp in bending (the book's Wt 552.6 lbf at 785.4 ft/min is 13.15 hp) and 10.9 hp in wear. The 1.10 in face is wider than 0.3 A0 = 1.06 in"
inputs = { m = "0.2 in", N_P = "25", N_G = "25", "Σ" = "90°", F = "1.10 in", H = "1 hp", n_P = "600 rpm", Q_v = "7", K_o = "1", K_mb = "1.25", C_xc = "2", I = "0.065", J_P = "0.216", J_G = "0.216", "s_(at,P)" = "10020 psi", "s_(at,G)" = "10020 psi", "s_(ac,P)" = "85000 psi", "s_(ac,G)" = "85000 psi", "H_(B,P)" = "180", "H_(B,G)" = "180", R = "0.99", N_L = "1e7", critical = "1", n_F = "1", n_H = "1" }
expect = { K_v = "1.299", H_b = "13.15 hp", H_w = "10.9 hp" }
tolerance = "0.3%"
checks = "fail"

[[tests]]
name = "Shigley Ex. 15-1 (c): the same pair at R 0.995, 10⁹ cycles, SF = SH = 1.5"
source = "Shigley 10th Example 15-1(c) (also Lacewing Lab bevel_test): KR = 1.075, KL = 0.8618 on the critical curve; 7.0 hp in bending, 2.58 hp in wear"
inputs = { m = "0.2 in", N_P = "25", N_G = "25", "Σ" = "90°", F = "1.10 in", H = "1 hp", n_P = "600 rpm", Q_v = "7", K_o = "1", K_mb = "1.25", C_xc = "2", I = "0.065", J_P = "0.216", J_G = "0.216", "s_(at,P)" = "10020 psi", "s_(at,G)" = "10020 psi", "s_(ac,P)" = "85000 psi", "s_(ac,G)" = "85000 psi", "H_(B,P)" = "180", "H_(B,G)" = "180", R = "0.995", N_L = "1e9", critical = "1", n_F = "1.5", n_H = "1.5" }
expect = { K_R = "1.075", "K_(L,P)" = "0.8618", H_b = "7.0 hp", H_w = "2.58 hp" }
tolerance = "0.5%"
checks = "fail"

[[tests]]
name = "Shigley Ex. 15-2: 20/60 crowned carburized set, P 8, 1.25 in face, 6.85 hp at 900 rev/min, 300 °F"
source = "Shigley 10th Example 15-2 (also Lacewing Lab bevel_test): γ = 18.43°, Wt = 383.8 lbf, Kv = 1.325 (Qv 6), σt = 8463 psi (pinion) and 10 390 psi (gear) with J 0.248 and 0.202, gear bending strength 11 640 psi at SF = 2 (SF 2 × 11 640/10 390 = 2.241), σc = 107 560 psi with I 0.0825 against 136 120 psi at SH = 1.414 (SH 1.790); carburized grade 1, sat 30 000 and sac 200 000 psi, R 0.995, 10⁹ cycles. KT = (460 + 300)/710 in the book; 759.67/710 here (the Rankine temperature), 0.04 % lower"
inputs = { m = "0.125 in", N_P = "20", N_G = "60", "Σ" = "90°", F = "1.25 in", H = "6.85 hp", n_P = "900 rpm", Q_v = "6", K_o = "1", K_mb = "1.25", C_xc = "1.5", I = "0.0825", J_P = "0.248", J_G = "0.202", "s_(at,P)" = "30000 psi", "s_(at,G)" = "30000 psi", "s_(ac,P)" = "200000 psi", "s_(ac,G)" = "200000 psi", "H_(B,P)" = "600", "H_(B,G)" = "600", R = "0.995", N_L = "1e9", critical = "1", "θ" = "300 °F", n_F = "2", n_H = "1.414" }
expect = { "γ" = "18.43°", W_t = "383.8 lbf", K_v = "1.325", "σ_(t,P)" = "8463 psi", "σ_(t,G)" = "10390 psi", "S_(F,G)" = "2.241", "σ_c" = "107560 psi", "S_(H,G)" = "1.790" }
tolerance = "0.2%"
checks = "fail"

[[tests]]
name = "Hand check: 18/40 through-hardened pair, P 6, 1.0 in face, 5 hp at 1200 rev/min"
source = "By hand: dP = 3 in, v = 942.47780 ft/min, Wt = 175.07044 lbf, Kv (Qv 8) = 1.2549804, Ks = 0.4867 + 0.2132/6 = 0.52223333, Cs = 0.5625, Km = 1.1 + 0.0036 = 1.1036, Cp = 2290.6039 √psi, KR = 1, CH = 1 + (0.00898 × 1.2 − 0.00829)(40/18 − 1) = 1.0030384; with Ko 1.25, I 0.07, J 0.23 and 0.19: σc = 92 300.137 psi, σt = 4129.1387 and 4998.4311 psi; sat = 44 HB + 2100, sac = 341 HB + 23 620 at 300 and 250 HB, 10⁸ pinion cycles on the general curves: SF = 3.6193250 and 2.5966059, SH = 1.5672782 and 1.4261125; at SF = SH = 1, 12.983029 hp in bending and 10.168984 hp in wear"
inputs = { m = "0.16666667 in", N_P = "18", N_G = "40", "Σ" = "90°", F = "1.0 in", H = "5 hp", n_P = "1200 rpm", Q_v = "8", K_o = "1.25", K_mb = "1.1", C_xc = "2", I = "0.07", J_P = "0.23", J_G = "0.19", "s_(at,P)" = "15300 psi", "s_(at,G)" = "13100 psi", "s_(ac,P)" = "125920 psi", "s_(ac,G)" = "108870 psi", "H_(B,P)" = "300", "H_(B,G)" = "250", R = "0.99", N_L = "1e8", critical = "0", "θ" = "100 °F", n_F = "1", n_H = "1" }
expect = { v_t = "942.47780 ft/min", W_t = "175.07044 lbf", K_v = "1.2549804", K_s = "0.52223333", C_s = "0.5625", K_m = "1.1036", C_H = "1.0030384", "σ_c" = "92300.137 psi", "σ_(t,P)" = "4129.1387 psi", "σ_(t,G)" = "4998.4311 psi", "S_(F,P)" = "3.6193250", "S_(F,G)" = "2.5966059", "S_(H,P)" = "1.5672782", "S_(H,G)" = "1.4261125", H_b = "12.983029 hp", H_w = "10.168984 hp" }
tolerance = "2e-6"
checks = "pass"
+++

## Straight bevel gear rating, AGMA 2003

The rating of a straight (Coniflex) bevel gear pair as Shigley Secs. 15-2 to 15-5 present ANSI/AGMA 2003-B97. The contact stress (Eq. 15-1) is the same for both members; the bending stress (Eq. 15-3) is each member's own. The allowables are the strength numbers at 10⁷ cycles and 99 % reliability, moved by the stress-cycle factors K_L and C_L, the reliability factors K_R and C_R and the temperature factor K_T. The factors of safety are strength over stress at the power given; the rated powers are the powers at which the weaker member just reaches the design factors n_F in bending and n_H in wear (the contact stress grows as the square root of the load). Twenty-degree teeth and a 90° shaft angle are what Shigley's charts of I and J are for.

The factors read from charts and tables are inputs: the pitting and bending geometry factors I and J (Figs. 15-6 and 15-7), the overload factor K_o (Table 15-2) and the strengths s_at and s_ac (Tables 15-4 to 15-7; grade 1 through-hardened steel s_at = 44 H_B + 2100 psi and s_ac = 341 H_B + 23 620 psi, Eqs. 15-22 and 15-23). For the geometry and the shaft loads use the bevel gear geometry template.

### Pair and load

```calc {description="Outer transverse module (1/P for a diametral pitch)"}
m = 0.2 in_
```

```calc {description="Pinion teeth"}
N_P = 25_
```

```calc {description="Gear teeth"}
N_G = 25_
```

```calc {description="Shaft angle"}
Σ = 90°_
```

```calc {description="Face width"}
F = 1.0 in_
```

```calc {description="Power transmitted"}
H = 5 hp_
```

```calc {description="Pinion speed"}
n_P = 600 rpm_
```

```calc {description="Transmission accuracy (quality) number Qv"}
Q_v = 7_
```

```calc {description="Overload factor, Shigley Table 15-2"}
K_o = 1_
```

```calc {description="Mounting factor of Eq. 15-11: 1.00 both members straddle-mounted, 1.10 one, 1.25 neither"}
K_mb = 1.25_
```

```calc {description="Crowning factor, Eq. 15-12: 1.5 crowned, 2.0 uncrowned"}
C_xc = 2_
```

```calc
> Gear ratio
m_G = N_G / N_P = ??
> Pinion pitch angle
γ = atan(sin(Σ) / (m_G + cos(Σ))) = ?? °
> Outer pitch diameter of the pinion
d_P = N_P m = ?? in
> Outer cone distance, Eq. 15-25
A_0 = d_P / (2 sin(γ)) = ?? in
> Largest face width, Eq. 15-24
F_max = min(0.3 A_0, 10 m) = ?? in
> Pitch-line velocity at the outer pitch circle, Eq. 15-7
v_t = n_P d_P / 2 = ?? ft/min
> Transmitted load
W_t = H / v_t = ?? lbf
```

### Factors

Shigley's U.S. customary forms: F in inches, P in teeth per inch, v_t in ft/min.

```calc
> Face width in inches and diametral pitch in teeth per inch
f = F / (1 'in') = ??
P = 1 'in' / m = ??
> Dynamic factor, Eqs. 15-5 and 15-6
B = 0.25 (12 - Q_v)^(2 / 3) = ??
A = 50 + 56 (1 - B) = ??
K_v = ((A + sqrt(v_t / (1 'ft/min'))) / A)^B = ??
> Top of the Qv curve, Eq. 15-8
v_max = (A + Q_v - 3)^2 × 1 'ft/min' = ?? ft/min
> Size factor for bending, Eq. 15-10
K_s = {0.5 if P > 16; 0.4867 + 0.2132 / P otherwise} = ??
> Size factor for pitting, Eq. 15-9
C_s = {0.5 if f < 0.5; 0.125 f + 0.4375 if f ≤ 4.5; 1 otherwise} = ??
> Load-distribution factor, Eq. 15-11
K_m = K_mb + 0.0036 f^2 = ??
```

### Geometry factors and materials

```calc {description="Pitting geometry factor, Shigley Fig. 15-6"}
I = 0.065_
```

```calc {description="Bending geometry factor of the pinion, Shigley Fig. 15-7"}
J_P = 0.216_
```

```calc {description="Bending geometry factor of the gear, Shigley Fig. 15-7"}
J_G = 0.216_
```

```calc {description="Young's modulus of the pinion"}
E_P = 30 Mpsi_
```

```calc {description="Poisson's ratio of the pinion"}
ν_P = 0.30_
```

```calc {description="Young's modulus of the gear"}
E_G = 30 Mpsi_
```

```calc {description="Poisson's ratio of the gear"}
ν_G = 0.30_
```

```calc {description="Allowable bending stress number of the pinion, Shigley Tables 15-4, 15-5, Fig. 15-13"}
s_(at,P) = 10020 psi_
```

```calc {description="Allowable bending stress number of the gear, Shigley Tables 15-4, 15-5, Fig. 15-13"}
s_(at,G) = 10020 psi_
```

```calc {description="Allowable contact stress number of the pinion, Shigley Tables 15-6, 15-7, Fig. 15-12"}
s_(ac,P) = 85000 psi_
```

```calc {description="Allowable contact stress number of the gear, Shigley Tables 15-6, 15-7, Fig. 15-12"}
s_(ac,G) = 85000 psi_
```

The hardness ratio factor C_H (Eq. 15-16) raises the gear's pitting allowable when both members are through-hardened and the pinion is at least 1.2 times as hard, up to a ratio of 1.7. For a surface-hardened pinion on a through-hardened gear Eq. 15-17 applies instead, from the pinion's roughness; that is not done here, and equal hardnesses give C_H = 1, which is on the safe side. Do the same for surface-hardened pairs.

```calc {description="Brinell hardness of the pinion (for CH; through-hardened pairs)"}
H_(B,P) = 180_
```

```calc {description="Brinell hardness of the gear (for CH; through-hardened pairs)"}
H_(B,G) = 180_
```

```calc
> Elastic coefficient, Eq. 15-21
C_p = sqrt(1 / (π ((1 - ν_P^2) / E_P + (1 - ν_G^2) / E_G))) = ?? √psi
> Hardness ratio factor of the gear, Eq. 15-16
C_H = 1 + {0 if H_(B,P) / H_(B,G) < 1.2; 0.00898 min(H_(B,P) / H_(B,G), 1.7) - 0.00829 otherwise} (m_G - 1) = ??
```

### Life, reliability and temperature

K_L (Eq. 15-15) has a general curve and a critical (lower) one above 3×10⁶ cycles; C_L is Eq. 15-14. K_R is Eqs. 15-19 and 15-20 (Table 15-3's values at 0.90, 0.99 and 0.999), and C_R = √K_R as Shigley's examples take it. K_T is 1 up to 250 °F and (460 + t)/710 above, t in °F (Eq. 15-18), written here as the blank's absolute (Rankine) temperature over 710 °R.

```calc {description="Reliability"}
R = 0.99_
```

```calc {description="Pinion load cycles over the life"}
N_L = 1e7_
```

```calc {description="Bending life curve above 3×10⁶ cycles: 1 critical (lower), 0 general"}
critical = 1_
```

```calc {description="Gear blank temperature"}
θ = 70 °F_
```

```calc
function KL(N, critical)
    if N < 1000
        return 2.7
    end
    if N < 3e6
        return 6.1514 N^(-0.1192)
    end
    return {1.683 N^(-0.0323) if critical == 1; 1.3558 N^(-0.0178) otherwise}
end
```

```calc
function CL(N)
    return {2 if N < 10000; 3.4822 N^(-0.0602) otherwise}
end
```

```calc
> Stress-cycle factors for bending, Eq. 15-15
K_(L,P) = KL(N_L, critical) = ??
K_(L,G) = KL(N_L / m_G, critical) = ??
> Stress-cycle factors for pitting, Eq. 15-14
C_(L,P) = CL(N_L) = ??
C_(L,G) = CL(N_L / m_G) = ??
> Reliability factor for bending, Eqs. 15-19 and 15-20
K_R = {0.50 - 0.25 log10(1 - R) if R ≥ 0.99; 0.70 - 0.15 log10(1 - R) otherwise} = ??
> Reliability factor for pitting
C_R = sqrt(K_R) = ??
> Temperature factor, Eq. 15-18
K_T = {1 if θ ≤ 250 '°F'; θ / (710 '°R') otherwise} = ??
```

### Stresses and factors of safety

```calc
> Contact stress, Eq. 15-1
σ_c = C_p sqrt(W_t / (F d_P I) K_o K_v K_m C_s C_xc) = ?? psi
> Bending stresses, Eq. 15-3 (Kx = 1 for straight bevels)
σ_(t,P) = W_t / (F m) K_o K_v K_s K_m / J_P = ?? psi
σ_(t,G) = W_t / (F m) K_o K_v K_s K_m / J_G = ?? psi
> Bending strengths at a factor of safety of 1, Eq. 15-4
s_(wt,P) = s_(at,P) K_(L,P) / (K_T K_R) = ?? psi
s_(wt,G) = s_(at,G) K_(L,G) / (K_T K_R) = ?? psi
> Contact strengths at a factor of safety of 1, Eq. 15-2
s_(wc,P) = s_(ac,P) C_(L,P) / (K_T C_R) = ?? psi
s_(wc,G) = s_(ac,G) C_(L,G) C_H / (K_T C_R) = ?? psi
> Bending factors of safety
S_(F,P) = s_(wt,P) / σ_(t,P) = ??
S_(F,G) = s_(wt,G) / σ_(t,G) = ??
> Wear factors of safety
S_(H,P) = s_(wc,P) / σ_c = ??
S_(H,G) = s_(wc,G) / σ_c = ??
```

### Rated power

```calc {description="Bending design factor for the rated power"}
n_F = 1_
```

```calc {description="Wear design factor for the rated power"}
n_H = 1_
```

```calc
> Power at which the weaker member reaches the bending design factor
H_b = H min(S_(F,P), S_(F,G)) / n_F = ?? hp
> Power at which it reaches the wear design factor (σc goes as √Wt)
H_w = H min(S_(H,P), S_(H,G))^2 / n_H^2 = ?? hp
> Rated power
H_r = min(H_b, H_w) = ?? hp
```

### Checks

```calc
@test S_(F,P) ≥ n_F
```

```calc
@test S_(F,G) ≥ n_F
```

```calc
@test S_(H,P) ≥ n_H
```

```calc
@test S_(H,G) ≥ n_H
```

```calc
@test F ≤ F_max
```

```calc
@test v_t ≤ v_max
```
