+++
cruncha-template = 1
id = "mechanical.wormGears.agma"
version = "1.0.0"
title = "Cylindrical worm gearing, AGMA (Shigley)"
summary = "A cylindrical worm and its bronze gear as Shigley Secs. 15-6 to 15-9 give ANSI/AGMA 6034-B92 and 6022-C93: proportions, lead angle, sliding speed and friction, efficiency with the worm and with the gear driving, the tangential, friction and separating forces, input power and friction loss, the AGMA allowable gear load, Buckingham's gear bending stress, the self-locking limit, and the case heat balance with the sump temperature."
category = ["Mechanical engineering", "Gears"]
tags = ["gear", "worm gear", "worm", "worm wheel", "lead angle", "efficiency", "self-locking", "back-driving", "friction", "sliding velocity", "AGMA", "AGMA 6034", "heat balance", "sump temperature", "Shigley", "Juvinall"]
from = "lacewing:mechanical.wormGears"
license = "GPL-3.0-or-later"
inputs = ["p_x", "N_W", "N_G", "d", "φ_n", "F_G", "n_W", "H_o", "K_a", "n_d", "casting", "λ_max", "N_(G,min)", "A", "t_a", "fan"]
outputs = ["m_G", "D", "C", "L", "λ", "a", "b", "d_o", "D_t", "F_(W,max)", "F_e", "V_W", "V_G", "V_s", "f", "e_W", "e_G", "W_(t,G)", "W_(t,W)", "H_W", "H_G", "W_f", "H_f", "W_r", "f_lock", "C_s", "C_m", "C_v", "W_(t,all)", "σ_G", "A_min", "h_CR", "H_loss", "t_s"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 13-11 and 15-6 to 15-9: Eqs. 13-27, 13-28 (lead, lead angle), 15-27 (worm diameter), 15-28 and 15-32 to 15-37 (allowable load, Cs, Cm, Cv), 15-38 (friction), 15-39 to 15-47 (proportions), 15-49 to 15-52 (heat), 15-53 (bending stress), 15-54 to 15-63 (efficiency, forces, powers); Tables 15-9 and 15-10; Examples 15-3 and 15-4"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 16.10 and 16.11: Eq. 16.32 (separating force), Eq. 16.36 (self-locking); Sample Problem 16.2"

[[references]]
work = "ANSI/AGMA 6034-B92 (Annex C) and 6022-C93, cylindrical wormgear speed reducers and gearmotors"
edition = "1992, 1993"
locator = "as presented in Shigley Secs. 15-6 to 15-9"

[[tests]]
name = "Shigley Ex. 15-3: single-thread worm, 24-tooth chilled-cast bronze gear, Pt 4, 3 hp out at 1800 rev/min"
source = "Shigley 10th Example 15-3 (also Lacewing Lab worm_test): px = π/4 in, d = 3 in, φn 14.5°, FG 2 in, Ka 1.25, 600 in² case with a fan, 70 °F air: λ = 4.764°, Vs = 1419 ft/min, f = 0.0178, eW = 0.818, WtG = 1284 lbf, WtW = 131 lbf, Cs 1000, Cm 0.823, Cv 0.211, allowable 1456 lbf, friction loss 1.02 hp, HW 5.61 hp, HG 4.58 hp, σ = 8200 psi, Amin 557 in², sump 166 °F. The 3 in worm is above AGMA's C^0.875/1.6 = 2.33 in and 24 teeth are under Table 15-10's 40 at 14.5°, so the checks fail"
inputs = { p_x = "0.78539816 in", N_W = "1", N_G = "24", d = "3 in", "φ_n" = "14.5°", F_G = "2 in", n_W = "1800 rpm", H_o = "3 hp", K_a = "1.25", n_d = "1", casting = "2", "λ_max" = "16°", "N_(G,min)" = "40", A = "600 in²", t_a = "70 °F", fan = "1" }
expect = { "λ" = "4.764°", V_s = "1419 ft/min", f = "0.0178", e_W = "0.818", "W_(t,G)" = "1284 lbf", "W_(t,W)" = "131 lbf", C_s = "1000", C_m = "0.823", C_v = "0.211", "W_(t,all)" = "1456 lbf", H_f = "1.02 hp", H_W = "5.61 hp", H_G = "4.58 hp", "σ_G" = "8200 psi", A_min = "557 in²", t_s = "166 °F" }
tolerance = "0.4%"
checks = "fail"

[[tests]]
name = "Shigley Ex. 15-4: double-thread worm, 22-tooth sand-cast gear, 10 hp out at 1720 rev/min, Ka 1.25, nd 1.2"
source = "Shigley 10th Example 15-4 (also Lacewing Lab worm_test): px 1.5 in, d 2.5 in, φn 20°, FG 1.5 in, 1100 in² case without a fan: λ = 20.905°, Vs = 1205.1 ft/min, Cs = 702.8, Cm = 0.772, Cv = 0.232, f = 0.0191, eW = 0.942, eG = 0.939, WtG = 1222 lbf, WtW = 495.4 lbf, HW 16.9 hp, HG 15.92 hp, friction loss 0.979 hp, allowable 1239 lbf (1237 lbf by these equations), σ = 4652 psi, sump 144.5 °F"
inputs = { p_x = "1.5 in", N_W = "2", N_G = "22", d = "2.5 in", "φ_n" = "20°", F_G = "1.5 in", n_W = "1720 rpm", H_o = "10 hp", K_a = "1.25", n_d = "1.2", casting = "1", "λ_max" = "25°", "N_(G,min)" = "21", A = "1100 in²", t_a = "70 °F", fan = "0" }
expect = { "λ" = "20.905°", V_s = "1205.1 ft/min", C_s = "702.8", C_m = "0.772", C_v = "0.232", f = "0.0191", e_W = "0.942", e_G = "0.939", "W_(t,G)" = "1222 lbf", "W_(t,W)" = "495.4 lbf", H_W = "16.9 hp", H_G = "15.92 hp", H_f = "0.979 hp", "W_(t,all)" = "1239 lbf", "σ_G" = "4652 psi", t_s = "144.5 °F" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Juvinall Sample Problem 16.2: 2 hp in at 1200 rev/min, double-thread worm, px 5/8 in, 40 teeth, c 5 in"
source = "Juvinall 5th Sample Problem 16.2 (also Lacewing Lab worm_test): dw = 2.04 in (as the book rounds it), dg = 7.96 in, λ = 11.04°, Vs = 652 ft/min, WtW = 103 lb, Wr = 122 lb, efficiency about 87 %, self-locking limit cos φn tan λ = 0.19; the book reads f = 0.026 off Fig. 16.23 where Shigley's Eq. 15-38 gives 0.0255. The output power 1.5367 hp is the one that takes 2 hp at the worm"
inputs = { p_x = "0.625 in", N_W = "2", N_G = "40", d = "2.04 in", "φ_n" = "14.5°", F_G = "1 in", n_W = "1200 rpm", H_o = "1.5367 hp", K_a = "1", n_d = "1", casting = "1", "λ_max" = "16°", "N_(G,min)" = "40", A = "700 in²", t_a = "70 °F", fan = "0" }
expect = { D = "7.96 in", "λ" = "11.04°", V_s = "652 ft/min", "W_(t,W)" = "103 lbf", W_r = "122 lbf", e_W = "0.87", f_lock = "0.19", H_W = "2 hp" }
tolerance = "1%"
checks = "pass"
+++

## Cylindrical worm gearing, AGMA

A cylindrical (single-enveloping) worm of hardened steel driving a bronze worm gear on crossed shafts at 90°, as Shigley Secs. 15-6 to 15-9 present AGMA 6034 and 6022. The friction of the sliding teeth sets the efficiency, the input power and the heat the case must shed. AGMA's allowable tangential load on the gear (Eq. 15-28) is for a case-hardened alloy steel worm and a bronze gear, for lives over 25 000 h. The gear load here includes the application factor K_a and a design factor n_d, and uses the efficiency (Eq. 15-58).

Shigley's empirical equations are in U.S. customary units (inches, ft/min, rev/min, lbf, hp); the template converts where they need it. The tabulated guides are inputs: the largest lead angle for the pressure angle (Table 15-9: 16° at 14.5°, 25° at 20°, 35° at 25°, 45° at 30°) and the fewest gear teeth (Table 15-10: 40 at 14.5°, 21 at 20°, 14 at 25°, 10 at 30°).

### Worm and gear

```calc {description="Axial pitch of the worm (the gear's circular pitch; π/Pt)"}
p_x = 1.5 in_
```

```calc {description="Number of worm threads (starts)"}
N_W = 2_
```

```calc {description="Gear teeth"}
N_G = 22_
```

```calc {description="Worm pitch diameter"}
d = 2.5 in_
```

```calc {description="Normal pressure angle"}
φ_n = 20°_
```

```calc {description="Gear face width"}
F_G = 1.5 in_
```

```calc
> Ratio
m_G = N_G / N_W = ??
> Gear pitch diameter
D = N_G p_x / π = ?? in
> Centre distance
C = (d + D) / 2 = ?? in
> Lead, Eq. 13-27
L = p_x N_W = ?? in
> Lead angle, Eq. 13-28
λ = atan(L / (π d)) = ?? °
> Addendum and dedendum, Eqs. 15-39 and 15-40
a = p_x / π = ?? in
b = 0.3683 p_x = ?? in
> Worm outside diameter and gear throat diameter, Eqs. 15-42 and 15-44
d_o = d + 2 a = ?? in
D_t = D + 2 a = ?? in
> Longest useful worm face, Eq. 15-47
F_(W,max) = 2 sqrt(2 D a) = ?? in
> Effective gear face (no more than 2/3 of the worm diameter)
F_e = min(F_G, 2 d / 3) = ?? in
```

### Speeds, friction and efficiency

```calc {description="Worm speed"}
n_W = 1720 rpm_
```

```calc
> Pitch-line velocities of worm and gear
V_W = n_W d / 2 = ?? ft/min
V_G = n_W D / (2 m_G) = ?? ft/min
> Sliding velocity, Eq. 15-62
V_s = V_W / cos(λ) = ?? ft/min
> Sliding velocity in ft/min, for Eqs. 15-37 and 15-38
v_s = V_s / (1 'ft/min') = ??
> Coefficient of friction, Eq. 15-38
f = {0.15 if v_s == 0; 0.124 exp(-0.074 v_s^0.645) if v_s ≤ 10; 0.103 exp(-0.110 v_s^0.450) + 0.012 otherwise} = ??
> Efficiency with the worm driving, Eq. 15-54
e_W = (cos(φ_n) - f tan(λ)) / (cos(φ_n) + f / tan(λ)) = ??
> Efficiency with the gear driving, Eq. 15-55 (negative or zero: it cannot back-drive)
e_G = (cos(φ_n) - f / tan(λ)) / (cos(φ_n) + f tan(λ)) = ??
> Self-locking limit, Juvinall Eq. 16.36: the gear cannot drive the worm if f exceeds it
f_lock = cos(φ_n) tan(λ) = ??
```

### Forces and powers

```calc {description="Output power at the gear shaft"}
H_o = 10 hp_
```

```calc {description="Application factor"}
K_a = 1.25_
```

```calc {description="Design factor on the gear load"}
n_d = 1.2_
```

```calc
> Gear tangential load, Eq. 15-58
W_(t,G) = n_d H_o K_a / (V_G e_W) = ?? lbf
> Worm tangential load (the gear's axial thrust), Eq. 15-57
W_(t,W) = W_(t,G) (cos(φ_n) sin(λ) + f cos(λ)) / (cos(φ_n) cos(λ) - f sin(λ)) = ?? lbf
> Worm and gear powers, Eqs. 15-59 and 15-60
H_W = W_(t,W) V_W = ?? hp
H_G = W_(t,G) V_G = ?? hp
> Friction force, Eq. 15-61
W_f = f W_(t,G) / (f sin(λ) - cos(φ_n) cos(λ)) = ?? lbf
> Power lost to friction, Eq. 15-63
H_f = abs(W_f) V_s = ?? hp
> Separating (radial) force, Juvinall Eq. 16.32
W_r = W_(t,W) sin(φ_n) / (cos(φ_n) sin(λ) + f cos(λ)) = ?? lbf
```

### AGMA allowable gear load

The materials factor C_s depends on how the bronze gear is cast (Eqs. 15-33 to 15-35, for centre distances over 3 in; Eq. 15-32 below): 1 sand cast, 2 chilled cast, 3 centrifugally cast. The mean gear diameter is taken as its pitch diameter, as Shigley's examples do.

```calc {description="Bronze gear casting: 1 sand cast, 2 chilled cast, 3 centrifugally cast"}
casting = 1_
```

```calc
> Centre distance and gear diameter in inches
c_in = C / (1 'in') = ??
D_in = D / (1 'in') = ??
> Materials factor, Eqs. 15-32 to 15-35
C_s = {720 + 10.37 c_in^3 if c_in ≤ 3; 1000 if casting == 1 and D_in ≤ 2.5; 1190 - 477 log10(D_in) if casting == 1; 1000 if casting == 2 and D_in ≤ 8; 1412 - 456 log10(D_in) if casting == 2; 1000 if D_in ≤ 25; 1251 - 180 log10(D_in) otherwise} = ??
> Ratio correction factor, Eq. 15-36
C_m = {0.02 sqrt(-m_G^2 + 40 m_G - 76) + 0.46 if m_G ≤ 20; 0.0107 sqrt(-m_G^2 + 56 m_G + 5145) if m_G ≤ 76; 1.1483 - 0.00658 m_G otherwise} = ??
> Velocity factor, Eq. 15-37
C_v = {0.659 exp(-0.0011 v_s) if v_s < 700; 13.31 v_s^(-0.571) if v_s < 3000; 65.52 v_s^(-0.774) otherwise} = ??
> Allowable gear tangential load, Eq. 15-28
W_(t,all) = C_s D_in^0.8 (F_e / (1 'in')) C_m C_v × 1 'lbf' = ?? lbf
```

### Gear bending stress

Buckingham's adaptation of the Lewis equation (Eq. 15-53), with the form factor y from the pressure angle: 0.100 at 14.5°, 0.125 at 20°, 0.150 at 25°, 0.175 at 30°.

```calc
> Normal circular pitch
p_n = p_x cos(λ) = ?? in
> Lewis form factor for the pressure angle
y = {0.100 if φ_n < 17°; 0.125 if φ_n < 22.5°; 0.150 if φ_n < 27.5°; 0.175 otherwise} = ??
> Gear bending stress, Eq. 15-53
σ_G = W_(t,G) / (p_n F_e y) = ?? psi
```

### Heat balance

The friction loss heats the oil until the case sheds it (Eqs. 15-49 to 15-51). The heat transfer coefficient of the case is n_W/3939 + 0.13 with a fan on the worm shaft and n_W/6494 + 0.13 without, in ft·lbf/(min·in²·°F) with n_W in rev/min (Eq. 15-50). AGMA's smallest lateral case area is 43.2 C^1.7 in² (Eq. 15-52).

```calc {description="Lateral area of the gear case"}
A = 1100 in²_
```

```calc {description="Ambient air temperature"}
t_a = 70 °F_
```

```calc {description="Fan on the worm shaft: 1 yes, 0 no"}
fan = 0_
```

```calc
> Heat loss rate, Eq. 15-49
H_loss = (1 - e_W) H_W = ?? hp
> Overall heat transfer coefficient of the case, Eq. 15-50
h_CR = ({n_W / (1 'rpm') / 3939 if fan == 1; n_W / (1 'rpm') / 6494 otherwise} + 0.13) × 1 'ft·lbf/(min·in²·Δ°F)' = ?? ft·lbf/(min·in²·Δ°F)
> Sump (oil) temperature, Eq. 15-51
t_s = t_a + H_loss / (h_CR A) = ?? °F
> Smallest lateral case area, Eq. 15-52
A_min = 43.2 c_in^1.7 × 1 'in²' = ?? in²
```

### Checks

AGMA's range of worm diameters for the centre distance (Eq. 15-27, in inches), the lead angle and tooth count of Tables 15-9 and 15-10, the gear load against the allowable, and a case at least AGMA's minimum.

```calc {description="Largest lead angle for the pressure angle, Shigley Table 15-9"}
λ_max = 25°_
```

```calc {description="Fewest gear teeth for the pressure angle, Shigley Table 15-10"}
N_(G,min) = 21_
```

```calc
@test W_(t,G) ≤ W_(t,all)
```

```calc
@test c_in^0.875 / 3 ≤ d / (1 'in') ≤ c_in^0.875 / 1.6
```

```calc
@test λ ≤ λ_max
```

```calc
@test N_G ≥ N_(G,min)
```

```calc
@test A ≥ A_min
```
