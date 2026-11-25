+++
cruncha-template = 1
id = "mechanical.gearRating.agma"
version = "1.0.0"
title = "Spur and helical gear rating, AGMA 2001 (Shigley)"
summary = "The AGMA bending and contact (pitting) stresses of an external spur or helical pair of steel or iron gears as Shigley ch. 14 gives ANSI/AGMA 2001-D04: transmitted load, dynamic factor from the quality number, size, load-distribution, rim-thickness and hardness-ratio factors, elastic coefficient, the pitting geometry factor with load sharing, stress-cycle factors from the life, and the bending and wear factors of safety of pinion and gear."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "spur gear", "helical gear", "AGMA", "AGMA 2001", "gear rating", "bending stress", "contact stress", "pitting", "wear", "factor of safety", "dynamic factor", "load distribution", "geometry factor", "stress cycle factor", "Lewis", "Shigley"]
from = "lacewing:mechanical.gearRating"
license = "GPL-3.0-or-later"
inputs = ["m_n", "φ_n", "ψ", "N_P", "N_G", "F", "H", "n_P", "J_P", "J_G", "Y_P", "Y_G", "Q_v", "K_o", "a_ma", "b_ma", "c_ma", "C_mc", "C_pm", "C_e", "m_(B,P)", "m_(B,G)", "E_P", "ν_P", "E_G", "ν_G", "S_(t,P)", "S_(t,G)", "S_(c,P)", "S_(c,G)", "H_(B,P)", "H_(B,G)", "N_L", "curve", "band", "K_R", "K_T", "C_f"]
outputs = ["m_t", "φ_t", "d_P", "d_G", "m_G", "V", "W_t", "K_v", "V_max", "K_(s,P)", "K_(s,G)", "C_pf", "C_ma", "K_m", "K_(B,P)", "K_(B,G)", "C_p", "Z", "m_N", "I", "C_H", "σ_P", "σ_G", "σ_(c,P)", "σ_(c,G)", "Y_(N,P)", "Y_(N,G)", "Z_(N,P)", "Z_(N,G)", "S_(F,P)", "S_(F,G)", "S_(H,P)", "S_(H,G)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 14-3 to 14-17: Eqs. 14-13 (Cp), 14-15 and 14-16 (stresses), 14-17 and 14-18 (allowables), 14-21 to 14-25 (I, load sharing), 14-27 to 14-29 (Kv), 14-30 to 14-35 and Table 14-9 (Km), 14-36 (CH), 14-40 (KB), 14-41 and 14-42 (SF, SH), Sec. 14-10 Eq. (a) and Table 14-2 (Ks), Figs. 14-2 to 14-8 and Tables 14-3 to 14-7 (strengths, J), Figs. 14-14 and 14-15 (YN, ZN), Table 14-10 (KR), Figs. 14-17 and 14-18 (Ko); Examples 14-4 and 14-5"

[[references]]
work = "ANSI/AGMA 2001-D04, Fundamental Rating Factors and Calculation Methods for Involute Spur and Helical Gear Teeth"
edition = "2004"
locator = "as presented in Shigley ch. 14"

[[tests]]
name = "Shigley Ex. 14-4: spur 17/52, P 10, 1.5 in face, 4 hp at 1800 rev/min, grade 1 steel 240/200 HB"
source = "Shigley 10th Example 14-4 (also Lacewing Lab rating_test): Wt = 164.8 lbf, Kv = 1.377, Km = 1.22, Ks = 1.043 and 1.052, I = 0.121, CH = 1.005, σ = 6417 and 4854 psi, SF = 5.62 and 6.82; σc = 70 360 and 70 660 psi, SH = 1.69 and 1.52. The book takes Cp = 2300 √psi from Table 14-8 where Eq. 14-13 gives 2291, 0.4 % on σc. St = 77.3 HB + 12 800 and Sc = 322 HB + 29 100 psi (Figs. 14-2 and 14-5), J 0.30 and 0.40 (Fig. 14-6), Y 0.303 and 0.412 (Table 14-2), R 0.90 (KR 0.85), 10⁸ pinion cycles on the upper curves"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", F = "1.5 in", H = "4 hp", n_P = "1800 rpm", J_P = "0.30", J_G = "0.40", Y_P = "0.303", Y_G = "0.412", Q_v = "6", K_o = "1", "S_(t,P)" = "31352 psi", "S_(t,G)" = "28260 psi", "S_(c,P)" = "106380 psi", "S_(c,G)" = "93500 psi", "H_(B,P)" = "240", "H_(B,G)" = "200", N_L = "1e8", K_R = "0.85" }
expect = { W_t = "164.8 lbf", K_v = "1.377", K_m = "1.22", "K_(s,P)" = "1.043", "K_(s,G)" = "1.052", I = "0.121", C_H = "1.005", "σ_P" = "6417 psi", "σ_G" = "4854 psi", "S_(F,P)" = "5.62", "S_(F,G)" = "6.82", "σ_(c,P)" = "70360 psi", "σ_(c,G)" = "70660 psi", "S_(H,P)" = "1.69", "S_(H,G)" = "1.52" }
tolerance = "0.7%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 14-5: the same pair helical, Pn 10, ψ 30°"
source = "Shigley 10th Example 14-5 (also Lacewing Lab rating_test): Wt = 142.7 lbf, Kv = 1.404, I = 0.195, σ = 3445 and 2779 psi, SF = 10.5 and 11.9, σc = 48 230 psi, SH = 2.46 and 2.22, with J = 0.423 and 0.529 (Figs. 14-7 and 14-8) and Cp 2300 √psi in the book (2291 by Eq. 14-13)"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "30°", N_P = "17", N_G = "52", F = "1.5 in", H = "4 hp", n_P = "1800 rpm", J_P = "0.423", J_G = "0.529", Y_P = "0.303", Y_G = "0.412", Q_v = "6", K_o = "1", "S_(t,P)" = "31352 psi", "S_(t,G)" = "28260 psi", "S_(c,P)" = "106380 psi", "S_(c,G)" = "93500 psi", "H_(B,P)" = "240", "H_(B,G)" = "200", N_L = "1e8", K_R = "0.85" }
expect = { W_t = "142.7 lbf", K_v = "1.404", I = "0.195", "σ_P" = "3445 psi", "σ_G" = "2779 psi", "S_(F,P)" = "10.5", "S_(F,G)" = "11.9", "σ_(c,P)" = "48230 psi", "S_(H,P)" = "2.46", "S_(H,G)" = "2.22" }
tolerance = "0.7%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 14-4 exactly, Cp by Eq. 14-13"
source = "By hand: V = π 1.7 × 1800/12 = 801.10613 ft/min, Wt = 132 000/V = 164.77218 lbf, B = 0.25 × 6^(2/3) = 0.8255, A = 59.7716, Kv = 1.3771310, Km = 1 + 0.069485 + 0.150491 = 1.2199760, Cp = 2290.6039 √psi, I = 0.12110491, CH = 1.0051182, σP = 6416.8759 psi, SF = 31 352 × 0.97704/(0.85 σP) = 5.6145934; σcP = 70 043.382 psi, SH = 1.6946593; σcG = 70 331.853 psi, SH = 1.5297976"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", F = "1.5 in", H = "4 hp", n_P = "1800 rpm", J_P = "0.30", J_G = "0.40", Y_P = "0.303", Y_G = "0.412", Q_v = "6", K_o = "1", "S_(t,P)" = "31352 psi", "S_(t,G)" = "28260 psi", "S_(c,P)" = "106380 psi", "S_(c,G)" = "93500 psi", "H_(B,P)" = "240", "H_(B,G)" = "200", N_L = "1e8", K_R = "0.85" }
expect = { V = "801.10613 ft/min", W_t = "164.77218 lbf", K_v = "1.3771310", K_m = "1.2199760", C_p = "2290.6039 psi^0.5", I = "0.12110491", C_H = "1.0051182", "σ_P" = "6416.8759 psi", "S_(F,P)" = "5.6145934", "σ_(c,P)" = "70043.382 psi", "S_(H,P)" = "1.6946593", "σ_(c,G)" = "70331.853 psi", "S_(H,G)" = "1.5297976" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 14-4 at 40 hp, which pits"
source = "By hand: ten times the load; σ and so SF scale by 10 (SF = 0.562 and 0.682) and σc by √10 (SH = 0.534 and 0.481), so the pair fails in both modes"
inputs = { m_n = "0.1 in", "φ_n" = "20°", "ψ" = "0°", N_P = "17", N_G = "52", F = "1.5 in", H = "40 hp", n_P = "1800 rpm", J_P = "0.30", J_G = "0.40", Y_P = "0.303", Y_G = "0.412", Q_v = "6", K_o = "1", "S_(t,P)" = "31352 psi", "S_(t,G)" = "28260 psi", "S_(c,P)" = "106380 psi", "S_(c,G)" = "93500 psi", "H_(B,P)" = "240", "H_(B,G)" = "200", N_L = "1e8", K_R = "0.85" }
expect = { W_t = "1648 lbf" }
tolerance = "0.1%"
checks = "fail"
+++

## Spur and helical gear rating, AGMA 2001

The AGMA rating of an external spur or helical pair of steel or iron gears on parallel shafts between bearings, as Shigley ch. 14 presents ANSI/AGMA 2001-D04. The bending stress at the tooth root (Eq. 14-15) and the contact stress at the pitch point (Eq. 14-16) are worked for each gear, and compared with the allowable stress numbers moved from 10⁷ cycles and 99 % reliability to the life and reliability wanted. The bending factor of safety S_F is a ratio of loads and the wear factor S_H a ratio of stresses: compare S_F with S_H² (S_H³ for crowned teeth) to see which mode threatens first (Shigley Sec. 14-17). For idlers and planets, loaded both ways, take 70 % of S_t.

The factors read from charts and tables are inputs: the bending geometry factor J (Fig. 14-6 for spur, Figs. 14-7 and 14-8 for helical, or AGMA 908), the Lewis form factor Y for the size factor (Table 14-2), the overload factor K_o (Figs. 14-17 and 14-18), the mesh alignment constants (Table 14-9), the reliability factor K_R (Table 14-10) and the strengths S_t and S_c (Figs. 14-2 to 14-5, Tables 14-3 to 14-7). The rest are AGMA's equations. Twenty-degree full-depth teeth and standard centre distance are assumed for the pitting geometry factor I.

### Gears

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
N_P = 17_
```

```calc {description="Gear teeth"}
N_G = 52_
```

```calc {description="Face width of the narrower member"}
F = 1.5 in_
```

```calc
> Transverse module
m_t = m_n / cos(ψ) = ?? in
> Transverse pressure angle
φ_t = atan(tan(φ_n) / cos(ψ)) = ?? °
> Pitch diameters
d_P = N_P m_t = ?? in
d_G = N_G m_t = ?? in
> Gear ratio
m_G = N_G / N_P = ??
```

### Load and speed

```calc {description="Power transmitted"}
H = 4 hp_
```

```calc {description="Pinion speed"}
n_P = 1800 rpm_
```

```calc {description="Overload factor, Shigley Figs. 14-17 and 14-18"}
K_o = 1_
```

```calc {description="Transmission accuracy (quality) number Qv, 3 to 12"}
Q_v = 6_
```

```calc
> Pitch-line velocity
V = n_P d_P / 2 = ?? ft/min
> Transmitted tangential load
W_t = H / V = ?? lbf
> Dynamic factor exponent and constant, Eq. 14-28
B = 0.25 (12 - Q_v)^(2 / 3) = ??
A = 50 + 56 (1 - B) = ??
> Dynamic factor, Eq. 14-27 (V in ft/min)
K_v = ((A + sqrt(V / (1 'ft/min'))) / A)^B = ??
> Top of the Qv curve, Eq. 14-29
V_max = (A + Q_v - 3)^2 × 1 'ft/min' = ?? ft/min
```

### Size, load distribution and rim

Shigley's size factor (Sec. 14-10 Eq. a) grows with the tooth size and the Lewis form factor Y of Table 14-2; it is taken at the normal pitch, and never under 1. The load-distribution factor K_m (Eq. 14-30) is for F/d_P ≤ 2 and gears mounted between bearings: C_mc is 1 for uncrowned teeth and 0.8 for crowned; C_pm 1, or 1.1 for a straddle-mounted pinion offset by S₁/S ≥ 0.175; C_e 1, or 0.8 for gearing adjusted at assembly or lapped. The mesh alignment factor's constants come from Table 14-9 for the gearing class (commercial enclosed units here). The rim-thickness factor K_B is 1 for a backup ratio t_R/h_t of 1.2 or more (Eq. 14-40).

```calc {description="Lewis form factor of the pinion, Shigley Table 14-2"}
Y_P = 0.303_
```

```calc {description="Lewis form factor of the gear, Shigley Table 14-2"}
Y_G = 0.412_
```

```calc {description="Mesh alignment constant A, Shigley Table 14-9 (commercial enclosed: 0.127)"}
a_ma = 0.127_
```

```calc {description="Mesh alignment constant B (per inch of face), Shigley Table 14-9 (commercial enclosed: 0.0158)"}
b_ma = 0.0158_
```

```calc {description="Mesh alignment constant C (per square inch of face), Shigley Table 14-9 (commercial enclosed: −0.930e-4)"}
c_ma = -0.930e-4_
```

```calc {description="Lead correction factor: 1 uncrowned, 0.8 crowned teeth"}
C_mc = 1_
```

```calc {description="Pinion proportion modifier: 1, or 1.1 for a straddle-mounted pinion with S1/S ≥ 0.175"}
C_pm = 1_
```

```calc {description="Mesh alignment correction: 1, or 0.8 if adjusted at assembly or lapped"}
C_e = 1_
```

```calc {description="Pinion backup ratio tR/ht (large for a solid gear)"}
m_(B,P) = 2_
```

```calc {description="Gear backup ratio tR/ht (large for a solid gear)"}
m_(B,G) = 2_
```

```calc
> Size factors, Shigley Sec. 14-10 Eq. (a)
K_(s,P) = max(1, 1.192 ((F / (1 'in')) sqrt(Y_P) (m_n / (1 'in')))^0.0535) = ??
K_(s,G) = max(1, 1.192 ((F / (1 'in')) sqrt(Y_G) (m_n / (1 'in')))^0.0535) = ??
> Face width in inches, for Eqs. 14-32 and 14-34
f = F / (1 'in') = ??
> Pinion proportion factor, Eq. 14-32
C_pf = {max(F / (10 d_P), 0.05) - 0.025 if f ≤ 1; max(F / (10 d_P), 0.05) - 0.0375 + 0.0125 f if f ≤ 17; max(F / (10 d_P), 0.05) - 0.1109 + 0.0207 f - 0.000228 f^2 otherwise} = ??
> Mesh alignment factor, Eq. 14-34
C_ma = a_ma + b_ma f + c_ma f^2 = ??
> Load-distribution factor, Eq. 14-30
K_m = 1 + C_mc (C_pf C_pm + C_ma C_e) = ??
> Rim-thickness factors, Eq. 14-40
K_(B,P) = {1 if m_(B,P) ≥ 1.2; 1.6 ln(2.242 / m_(B,P)) otherwise} = ??
K_(B,G) = {1 if m_(B,G) ≥ 1.2; 1.6 ln(2.242 / m_(B,G)) otherwise} = ??
```

### Geometry factors

J is read from Shigley's charts or AGMA 908 for each gear. The pitting geometry factor I (Eq. 14-23) uses the load-sharing ratio m_N = p_N/(0.95 Z) for helical gears (Eq. 14-21), with the line of action Z of standard full-depth teeth (addendum one normal module) at the standard centre distance (Eq. 14-25).

```calc {description="Bending geometry factor J of the pinion, Shigley Fig. 14-6 (spur) or Figs. 14-7 and 14-8 (helical)"}
J_P = 0.30_
```

```calc {description="Bending geometry factor J of the gear, Shigley Fig. 14-6 (spur) or Figs. 14-7 and 14-8 (helical)"}
J_G = 0.40_
```

```calc
> Length of the line of action, Eq. 14-25
Z = sqrt((d_P / 2 + m_n)^2 - (d_P cos(φ_t) / 2)^2) + sqrt((d_G / 2 + m_n)^2 - (d_G cos(φ_t) / 2)^2) - (d_P + d_G) / 2 sin(φ_t) = ?? in
> Load-sharing ratio, Eq. 14-21 (1 for spur gears)
m_N = {1 if ψ == 0; π m_n cos(φ_n) / (0.95 Z) otherwise} = ??
> Pitting geometry factor, Eq. 14-23 (external gears)
I = cos(φ_t) sin(φ_t) / (2 m_N) m_G / (m_G + 1) = ??
```

### Materials

The elastic coefficient comes from the two materials' moduli and Poisson's ratios (Eq. 14-13; Table 14-8 rounds it, 2300 √psi for steel on steel). The strengths S_t and S_c are AGMA's allowable stress numbers at 10⁷ cycles and 0.99 reliability; through-hardened grade 1 steel has S_t = 77.3 H_B + 12 800 psi and S_c = 322 H_B + 29 100 psi (Figs. 14-2 and 14-5). The hardness ratio factor C_H applies to the gear when both are through-hardened and the pinion is harder (Eq. 14-36); for a surface-hardened pinion on a through-hardened gear Eq. 14-37 applies instead (not done here: enter equal hardnesses for C_H = 1, which is on the safe side).

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

```calc {description="Allowable bending stress number of the pinion, Shigley Figs. 14-2 to 14-4, Tables 14-3 and 14-4"}
S_(t,P) = 31352 psi_
```

```calc {description="Allowable bending stress number of the gear, Shigley Figs. 14-2 to 14-4, Tables 14-3 and 14-4"}
S_(t,G) = 28260 psi_
```

```calc {description="Allowable contact stress number of the pinion, Shigley Fig. 14-5, Tables 14-5 to 14-7"}
S_(c,P) = 106380 psi_
```

```calc {description="Allowable contact stress number of the gear, Shigley Fig. 14-5, Tables 14-5 to 14-7"}
S_(c,G) = 93500 psi_
```

```calc {description="Brinell hardness of the pinion (for CH)"}
H_(B,P) = 240_
```

```calc {description="Brinell hardness of the gear (for CH)"}
H_(B,G) = 200_
```

```calc
> Elastic coefficient, Eq. 14-13
C_p = sqrt(1 / (π ((1 - ν_P^2) / E_P + (1 - ν_G^2) / E_G))) = ?? √psi
> Hardness ratio
r_H = H_(B,P) / H_(B,G) = ??
> Hardness ratio factor of the gear, Eq. 14-36
C_H = 1 + {0 if r_H < 1.2; 0.00698 if r_H > 1.7; 8.98e-3 r_H - 8.29e-3 otherwise} (m_G - 1) = ??
```

### Life and reliability

The stress-cycle factors move the strengths from 10⁷ cycles to the life. Below 3×10⁶ cycles in bending (10⁷ in pitting) they follow the curve for the material, Fig. 14-14's equations: 1 for 160 HB, 2 for 250 HB, 3 for 400 HB, 4 nitrided, 5 case carburized (ZN is the nitrided curve for 4 and the steel curve otherwise). Above that the band of Figs. 14-14 and 14-15 has an upper edge (1, for general service) and a lower one (2, for critical service where pitting and tooth wear must be avoided). The gear sees N_L/m_G cycles. K_R is 0.85 at 0.90 reliability, 1 at 0.99, 1.25 at 0.999 (Table 14-10; Eq. 14-38 between). K_T is 1 for oil and blanks below about 120 °C (250 °F); the surface condition factor C_f is 1 for good finishes.

```calc {description="Pinion load cycles over the life"}
N_L = 1e8_
```

```calc {description="Life curve below the knee, Shigley Fig. 14-14: 1 160 HB, 2 250 HB, 3 400 HB, 4 nitrided, 5 carburized"}
curve = 2_
```

```calc {description="Band edge above the knee, Figs. 14-14 and 14-15: 1 upper (general), 2 lower (critical)"}
band = 1_
```

```calc {description="Reliability factor, Shigley Table 14-10 or Eq. 14-38"}
K_R = 0.85_
```

```calc {description="Temperature factor (1 below about 120 °C)"}
K_T = 1_
```

```calc {description="Surface condition factor (1 for good finishes)"}
C_f = 1_
```

```calc
function YN(N, curve, band)
    n = max(N, 1000)
    if n < 3e6
        a = {2.3194 if curve == 1; 4.9404 if curve == 2; 9.4518 if curve == 3; 3.517 if curve == 4; 6.1514 if curve == 5}
        b = {-0.0538 if curve == 1; -0.1045 if curve == 2; -0.148 if curve == 3; -0.0817 if curve == 4; -0.1192 if curve == 5}
        return a n^b
    end
    return {1.3558 n^(-0.0178) if band == 1; 1.6831 n^(-0.0323) otherwise}
end
```

```calc
function ZN(N, curve, band)
    n = max(N, 10000)
    if n < 1e7
        return {1.249 n^(-0.0138) if curve == 4; 2.466 n^(-0.056) otherwise}
    end
    return {1.4488 n^(-0.023) if band == 1; 2.466 n^(-0.056) otherwise}
end
```

```calc
> Bending stress-cycle factors, Fig. 14-14
Y_(N,P) = YN(N_L, curve, band) = ??
Y_(N,G) = YN(N_L / m_G, curve, band) = ??
> Pitting stress-cycle factors, Fig. 14-15
Z_(N,P) = ZN(N_L, curve, band) = ??
Z_(N,G) = ZN(N_L / m_G, curve, band) = ??
```

### Bending

The transverse diametral pitch 1/m_t goes into Eq. 14-15.

```calc
> Bending stress of the pinion, Eq. 14-15
σ_P = W_t K_o K_v K_(s,P) / (m_t F) K_m K_(B,P) / J_P = ?? psi
> Bending stress of the gear
σ_G = W_t K_o K_v K_(s,G) / (m_t F) K_m K_(B,G) / J_G = ?? psi
> Bending factor of safety of the pinion, Eq. 14-41
S_(F,P) = S_(t,P) Y_(N,P) / (K_T K_R) / σ_P = ??
> Bending factor of safety of the gear
S_(F,G) = S_(t,G) Y_(N,G) / (K_T K_R) / σ_G = ??
```

### Pitting

```calc
> Contact stress of the pinion, Eq. 14-16
σ_(c,P) = C_p sqrt(W_t K_o K_v K_(s,P) K_m / (d_P F) C_f / I) = ?? psi
> Contact stress of the gear
σ_(c,G) = C_p sqrt(W_t K_o K_v K_(s,G) K_m / (d_P F) C_f / I) = ?? psi
> Wear factor of safety of the pinion, Eq. 14-42
S_(H,P) = S_(c,P) Z_(N,P) / (K_T K_R) / σ_(c,P) = ??
> Wear factor of safety of the gear (with CH)
S_(H,G) = S_(c,G) Z_(N,G) C_H / (K_T K_R) / σ_(c,G) = ??
```

### Checks

Both factors of safety at least 1 for each gear, inside the scope of Eq. 14-30, and the pitch-line velocity on the quality number's curve.

```calc
@test S_(F,P) ≥ 1
```

```calc
@test S_(F,G) ≥ 1
```

```calc
@test S_(H,P) ≥ 1
```

```calc
@test S_(H,G) ≥ 1
```

```calc
@test F ≤ 2 d_P
```

```calc
@test V ≤ V_max
```
