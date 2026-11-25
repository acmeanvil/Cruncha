+++
cruncha-template = 1
id = "mechanical.gearRating.juvinall"
version = "1.0.0"
title = "Spur and helical gear capacity (Juvinall)"
summary = "Juvinall and Marshek's capacity estimate for an external spur or helical pair: the Lewis bending stress with the velocity, overload and mounting factors against the endurance limit corrected for the tooth's size, surface, reliability, temperature and one-way bending, and the Hertz stress at the pitch point against the surface fatigue strength, each as a factor of safety on the load, and the power at which the first of the four reaches 1."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "gear rating", "Lewis equation", "bending fatigue", "surface fatigue", "Hertz stress", "pitting", "velocity factor", "endurance limit", "capacity", "Juvinall"]
from = "lacewing:mechanical.gearRating"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N_P", "N_G", "b", "H", "n_P", "curve", "K_o", "K_m", "J_P", "J_G", "S_(n,P)′", "S_(n,G)′", "C_(S,P)", "C_(S,G)", "k_r", "k_t", "k_ms", "E_P", "E_G", "ν", "S_(fe,P)", "S_(fe,G)", "C_(Li,P)", "C_(Li,G)", "C_R"]
outputs = ["m_t", "φ_t", "d_P", "V", "F_t", "K_v", "K_(m,eff)", "C_G", "σ_P", "σ_G", "S_(n,P)", "S_(n,G)", "SF_(b,P)", "SF_(b,G)", "C_p", "I", "CR", "σ_H", "S_(H,P)", "S_(H,G)", "SF_(s,P)", "SF_(s,G)", "H_cap"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 15.7 to 15.10 and 16.4: Eqs. 15.17 to 15.25, 16.12 and 16.13; Tables 15.1 to 15.6; Figs. 15.22 to 15.24 and 15.27; Sample Problems 15.3 and 15.4"

[[tests]]
name = "Juvinall Sample Problem 15.3: 18/36 teeth, P 10, b 1.25 in, 1720 rev/min — 13.8 hp in bending"
source = "Juvinall 5th Sample Problem 15.3 (also Lacewing Lab rating_juvinall_test): steel at 330 and 290 HB, Sn′ 82.5 and 72.5 ksi, CS 0.68 and 0.70, J 0.235 and 0.28 (Fig. 15.23, tip load), curve D, Ko 1.25 (uniform source, moderate shock), Km 1.6 (less rigid mountings, Table 15.2), kr 0.814 (99 %): V = 811 ft/min, Sn = 63.9 and 57.8 ksi, σ = 114Ft and 96Ft psi, the pinion limits bending at Ft = 561 lb, 13.8 hp. At 10 hp the surface checks fail (Sample Problem 15.4)"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "18", N_G = "36", b = "1.25 in", H = "10 hp", n_P = "1720 rpm", curve = "4", K_o = "1.25", K_m = "1.6", J_P = "0.235", J_G = "0.28", "S_(n,P)′" = "82.5 ksi", "S_(n,G)′" = "72.5 ksi", "C_(S,P)" = "0.68", "C_(S,G)" = "0.70", k_r = "0.814" }
expect = { V = "811 ft/min", "S_(n,P)" = "63.9 ksi", "S_(n,G)" = "57.8 ksi", "SF_(b,P)" = "1.378" }
tolerance = "0.2%"
checks = "fail"

[[tests]]
name = "Juvinall Sample Problem 15.4: the same gears for 5 years of 40-hour weeks — about 3.2 hp in surface fatigue"
source = "Juvinall 5th Sample Problem 15.4 (also Lacewing Lab rating_juvinall_test): I = 0.107, Sfe = 122 ksi at the pinion's 330 HB (Table 15.5: 0.4 HB − 10 ksi), CLi 0.8 at 1.03×10⁹ cycles (Fig. 15.27), CR 1 (99 %): SH = 97.6 ksi, Ft = 129 lb, 3.2 hp at the pinion. The book takes Cp = 2300 √psi (Table 15.4) where Eq. 15.22 gives 2290, so Ft is 130 lb here. The book does not check the gear (106 ksi at 290 HB; here at the same CLi)"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "18", N_G = "36", b = "1.25 in", H = "10 hp", n_P = "1720 rpm", curve = "4", K_o = "1.25", K_m = "1.6", J_P = "0.235", J_G = "0.28", "S_(n,P)′" = "82.5 ksi", "S_(n,G)′" = "72.5 ksi", "C_(S,P)" = "0.68", "C_(S,G)" = "0.70", k_r = "0.814", "S_(fe,P)" = "122 ksi", "S_(fe,G)" = "106 ksi", "C_(Li,P)" = "0.8", "C_(Li,G)" = "0.8", C_R = "1" }
expect = { I = "0.107", C_p = "2290 psi^0.5", "S_(H,P)" = "97.6 ksi", "SF_(s,P)" = "0.321" }
tolerance = "0.3%"
checks = "fail"

[[tests]]
name = "Hand check: Sample Problem 15.4's gears at 2 hp"
source = "By hand: V = π 1.8 × 1720/12 = 810.53090 ft/min, Ft = 66 000/V = 81.428110 lb, Kv = (1200 + V)/1200 = 1.6754424, σP = Ft × 10/(1.25 × 0.235) × Kv × 1.25 × 1.6 = 9288.7224 psi, SnP = 82 500 × 0.68 × 0.814 × 1.4 = 63 931.56 psi, SF = 6.8827076; σH = 2289.8342 √(Ft/(1.25 × 1.8 × 0.10713127) × Kv × 1.25 × 1.6) = 77 040.942 psi, SF surface (97 600/σH)² = 1.6049314 and (84 800/σH)² = 1.2115701; capacity 2 × 1.2115701 = 2.4231401 hp"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "18", N_G = "36", b = "1.25 in", H = "2 hp", n_P = "1720 rpm", curve = "4", K_o = "1.25", K_m = "1.6", J_P = "0.235", J_G = "0.28", "S_(n,P)′" = "82.5 ksi", "S_(n,G)′" = "72.5 ksi", "C_(S,P)" = "0.68", "C_(S,G)" = "0.70", k_r = "0.814", "S_(fe,P)" = "122 ksi", "S_(fe,G)" = "106 ksi", "C_(Li,P)" = "0.8", "C_(Li,G)" = "0.8", C_R = "1" }
expect = { V = "810.53090 ft/min", F_t = "81.428110 lbf", K_v = "1.6754424", "σ_P" = "9288.7224 psi", "S_(n,P)" = "63931.56 psi", "SF_(b,P)" = "6.8827076", "σ_H" = "77040.942 psi", "SF_(s,P)" = "1.6049314", "SF_(s,G)" = "1.2115701", H_cap = "2.4231401 hp" }
tolerance = "1e-6"
checks = "pass"
+++

## Spur and helical gear capacity (Juvinall)

Juvinall and Marshek's estimate of the load an external spur or helical pair of gears carries (Secs. 15.7 to 15.10, helical Sec. 16.4), the counterpart of the AGMA 2001 rating with fewer and coarser factors. Bending is the Lewis equation with the velocity factor K_v, overload factor K_o and mounting factor K_m (Eq. 15.17), against the endurance limit of the tooth (Eq. 15.18). The surface is the Hertz stress at the pitch point (Eq. 15.24) against the surface fatigue strength (Eq. 15.25). Each factor of safety is a multiplier of the load: S_n/σ in bending, (S_H/σ_H)² on the surface, as the contact stress goes as the square root of the load. Juvinall suggests about 1.5 in bending (with 99.9 % reliability) and 1.1 to 1.5 on the surface, as K_o, K_m and k_r are already taken separately. The capacity is the power given times the least of the four, with no further safety factor.

Helical gears take 0.93 K_m (Eq. 16.12) and the contact length b·CR·0.95/cos ψ (Eq. 16.13), with CR the transverse contact ratio of standard full-depth teeth at the standard centre distance; the diametral pitch in the Lewis equation is the transverse one.

### Gears and load

```calc {description="Normal module (1/Pn for a diametral pitch)"}
m_n = 0.1 in_
```

```calc {description="Normal pressure angle"}
φ_n = 20°_
```

```calc {description="Helix angle (0 for spur gears)"}
ψ = 0°_
```

```calc {description="Pinion teeth"}
N_P = 18_
```

```calc {description="Gear teeth"}
N_G = 36_
```

```calc {description="Face width"}
b = 1.25 in_
```

```calc {description="Power transmitted"}
H = 2 hp_
```

```calc {description="Pinion speed"}
n_P = 1720 rpm_
```

```calc
> Transverse module
m_t = m_n / cos(ψ) = ?? in
> Transverse pressure angle
φ_t = atan(tan(φ_n) / cos(ψ)) = ?? °
> Pitch diameters
d_P = N_P m_t = ?? in
d_G = N_G m_t = ?? in
> Pitch-line velocity, Eq. 15.13
V = n_P d_P / 2 = ?? ft/min
> Transmitted tangential load, Eq. 15.14
F_t = H / V = ?? lbf
```

### Velocity, overload and mounting factors

The velocity factor curves of Fig. 15.24 go with how the gears are made: 1 (A) highest precision, shaved and ground; 2 (B) high precision, shaved and ground; 3 (C) precision; 4 (D) hobbed or shaped; 5 (E) form cut. Juvinall takes helical gears on A or B. K_o is Table 15.1's overload factor (the same table as AGMA's), K_m Table 15.2's mounting factor at the face width (1.6 for less rigid mountings and face widths to 2 in, 1.3 for accurate mountings).

```calc {description="Velocity factor curve, Juvinall Fig. 15.24: 1 A, 2 B, 3 C, 4 D, 5 E"}
curve = 4_
```

```calc {description="Overload factor, Juvinall Table 15.1"}
K_o = 1.25_
```

```calc {description="Mounting factor, Juvinall Table 15.2"}
K_m = 1.6_
```

```calc
> V in ft/min
v = V / (1 'ft/min') = ??
> Velocity factor, Fig. 15.24
K_v = {sqrt((78 + sqrt(v)) / 78) if curve == 1; (78 + sqrt(v)) / 78 if curve == 2; (50 + sqrt(v)) / 50 if curve == 3; (1200 + v) / 1200 if curve == 4; (600 + v) / 600 if curve == 5} = ??
> Mounting factor as used (0.93 K_m for helical gears, Eq. 16.12)
K_(m,eff) = {K_m if ψ == 0; 0.93 K_m otherwise} = ??
```

### Bending

The endurance limit S_n = S_n′·C_L·C_G·C_S·k_r·k_t·k_ms (Eq. 15.18) starts from the R. R. Moore limit S_n′ (about 250 H_B psi for steel), with C_L = 1 for bending, the gradient factor C_G of 0.85 for coarse teeth (P ≤ 5) and 1 otherwise, the fillet's surface factor C_S (Fig. 8.13), the reliability factor k_r of Table 15.3 (0.814 at 99 %), the temperature factor k_t (1 below 160 °F; 620/(460 + T) above, Eq. 15.19) and k_ms = 1.4 for one-way bending or 1 for idlers and planets. J is the AGMA bending geometry factor (Fig. 15.23).

```calc {description="Bending geometry factor of the pinion, Juvinall Fig. 15.23"}
J_P = 0.235_
```

```calc {description="Bending geometry factor of the gear, Juvinall Fig. 15.23"}
J_G = 0.28_
```

```calc {description="R. R. Moore endurance limit of the pinion material"}
S_(n,P)′ = 82.5 ksi_
```

```calc {description="R. R. Moore endurance limit of the gear material"}
S_(n,G)′ = 72.5 ksi_
```

```calc {description="Surface factor of the pinion's fillet, Juvinall Fig. 8.13"}
C_(S,P) = 0.68_
```

```calc {description="Surface factor of the gear's fillet, Juvinall Fig. 8.13"}
C_(S,G) = 0.70_
```

```calc {description="Reliability factor, Juvinall Table 15.3"}
k_r = 0.814_
```

```calc {description="Temperature factor, Juvinall Eq. 15.19 (1 below 160 °F)"}
k_t = 1_
```

```calc {description="Mean stress factor: 1.4 one-way bending, 1.0 two-way (idlers)"}
k_ms = 1.4_
```

```calc
> Gradient factor by the normal diametral pitch, Sec. 15.8
C_G = {0.85 if m_n ≥ 0.2 'in'; 1 otherwise} = ??
> Bending stresses, Eq. 15.17 (16.12 helical)
σ_P = F_t / (m_t b J_P) K_v K_o K_(m,eff) = ?? psi
σ_G = F_t / (m_t b J_G) K_v K_o K_(m,eff) = ?? psi
> Endurance limits, Eq. 15.18
S_(n,P) = S_(n,P)′ C_G C_(S,P) k_r k_t k_ms = ?? ksi
S_(n,G) = S_(n,G)′ C_G C_(S,G) k_r k_t k_ms = ?? ksi
> Bending factors of safety (ratios of load)
SF_(b,P) = S_(n,P) / σ_P = ??
SF_(b,G) = S_(n,G) / σ_G = ??
```

### Surface

The elastic coefficient is Eq. 15.22 (Table 15.4 rounds it: 2300 √psi for steel on steel). The surface fatigue strength S_fe at 10⁷ cycles and 99 % reliability is Table 15.5's (0.4 H_B − 10 ksi for steel), the life factor C_Li Fig. 15.27's at each member's cycles (the gear sees N/m_G), and C_R Table 15.6's reliability factor (1 at 99 %).

```calc {description="Young's modulus of the pinion, Juvinall Table 15.4"}
E_P = 30 Mpsi_
```

```calc {description="Young's modulus of the gear, Juvinall Table 15.4"}
E_G = 30 Mpsi_
```

```calc {description="Poisson's ratio of both members"}
ν = 0.30_
```

```calc {description="Surface fatigue strength of the pinion, Juvinall Table 15.5"}
S_(fe,P) = 122 ksi_
```

```calc {description="Surface fatigue strength of the gear, Juvinall Table 15.5"}
S_(fe,G) = 106 ksi_
```

```calc {description="Life factor of the pinion, Juvinall Fig. 15.27"}
C_(Li,P) = 0.8_
```

```calc {description="Life factor of the gear, Juvinall Fig. 15.27"}
C_(Li,G) = 0.8_
```

```calc {description="Reliability factor, Juvinall Table 15.6"}
C_R = 1_
```

```calc
> Elastic coefficient, Eq. 15.22
C_p = 0.564 sqrt(1 / ((1 - ν^2) / E_P + (1 - ν^2) / E_G)) = ?? √psi
> Ratio of pitch diameters
R = N_G / N_P = ??
> Geometry factor, Eq. 15.23
I = sin(φ_t) cos(φ_t) / 2 R / (R + 1) = ??
> Transverse contact ratio of standard full-depth teeth, Eq. 15.9
CR = (sqrt((d_P / 2 + m_n)^2 - (d_P cos(φ_t) / 2)^2) + sqrt((d_G / 2 + m_n)^2 - (d_G cos(φ_t) / 2)^2) - (d_P + d_G) / 2 sin(φ_t)) / (π m_t cos(φ_t)) = ??
> Hertz stress at the pitch point, Eq. 15.24 (16.13 helical)
σ_H = C_p sqrt(F_t / (b d_P I) {1 if ψ == 0; cos(ψ) / (0.95 CR) otherwise} K_v K_o K_(m,eff)) = ?? psi
> Surface fatigue strengths, Eq. 15.25
S_(H,P) = S_(fe,P) C_(Li,P) C_R = ?? ksi
S_(H,G) = S_(fe,G) C_(Li,G) C_R = ?? ksi
> Surface factors of safety (ratios of load)
SF_(s,P) = (S_(H,P) / σ_H)^2 = ??
SF_(s,G) = (S_(H,G) / σ_H)^2 = ??
```

### Capacity

```calc {description="Power at which the first factor of safety reaches 1"}
H_cap = H min(SF_(b,P), SF_(b,G), SF_(s,P), SF_(s,G)) = ?? hp
```

```calc
@test SF_(b,P) ≥ 1
```

```calc
@test SF_(b,G) ≥ 1
```

```calc
@test SF_(s,P) ≥ 1
```

```calc
@test SF_(s,G) ≥ 1
```
