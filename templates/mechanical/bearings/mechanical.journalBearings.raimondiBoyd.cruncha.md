+++
cruncha-template = 1
id = "mechanical.journalBearings.raimondiBoyd"
version = "1.0.0"
title = "Hydrodynamic journal bearing, Raimondi-Boyd charts"
summary = "A full (360°) hydrodynamic journal bearing under a steady load at a known average film temperature: unit load, Sommerfeld number, then, from Raimondi and Boyd's chart readings at that S, the minimum film thickness, eccentricity, friction coefficient, torque and power loss, oil flow and side flow, temperature rise, inlet and outlet temperatures and the maximum film pressure, with Petroff's friction beside it and Trumpler's and McKee's design checks."
category = ["Mechanical engineering", "Bearings"]
tags = ["journal bearing", "hydrodynamic bearing", "plain bearing", "sleeve bearing", "Raimondi", "Boyd", "Sommerfeld number", "film thickness", "eccentricity", "friction", "oil flow", "side flow", "temperature rise", "Trumpler", "Petroff", "lubrication", "Shigley"]
from = "lacewing:mechanical.journalBearings"
license = "GPL-3.0-or-later"
inputs = ["d", "l", "c", "N", "W", "W_st", "μ", "T_av", "ρ_o", "C_p", "h_v", "φ", "f_v", "Q_v", "Q_r", "p_v", "θ_pmax", "θ_p0"]
outputs = ["r", "P", "S", "h_0", "ε", "e", "f", "T_q", "H", "Q", "Q_s", "ΔT", "T_1", "T_max", "p_max", "f_P", "h_(0,min)", "P_st", "μNP"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 12-3 to 12-8: Eqs. 12-6 (Petroff), 12-7 (Sommerfeld number), 12-14 and 12-15 (temperature rise); Figs. 12-16 to 12-22 (Raimondi and Boyd's charts); Trumpler's criteria, Sec. 12-7; McKee's stability limit, Sec. 12-4; Examples 12-1 to 12-4"

[[references]]
work = "Raimondi and Boyd, A Solution for the Finite Journal Bearing and Its Application to Analysis and Design, Parts I-III, Trans. ASLE vol. 1"
edition = "1958"
locator = "Part III, the full bearing with film rupture at zero pressure (the charts Shigley and Juvinall reproduce; tabulated in Harnoy, Bearing Design in Machinery, 2003, Table 8-1)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Secs. 13.8 to 13.11: Figs. 13.13 to 13.19, Eq. 13.15 (Trumpler), Table 13.2 (unit loads); Sample Problem 13.3"

[[tests]]
name = "Shigley Exs. 12-1 to 12-4: 4 μreyn, 1800 rpm, 500 lbf, d = l = 1.5 in, c 0.0015 in"
source = "Lacewing Lab journal_bearings_test, after Shigley 10th Examples 12-1 to 12-4 with the book's chart readings (h0/c 0.42, φ 53°, (r/c)f 3.50, Q/(rcNl) 4.28, Qs/Q 0.655, P/pmax 0.42, θpmax 18.5°, θp0 75°): S 0.135, h0 0.00063 in, f 0.0070, T 2.62 lbf·in, H 0.075 hp, Q 0.217 in³/s, Qs 0.142 in³/s, pmax 529 psi. The average film temperature is not part of the examples (140 °F here). μN/P = 5.4·10⁻⁷ is under McKee's 1.7·10⁻⁶, so the checks fail"
inputs = { d = "1.5 in", l = "1.5 in", c = "0.0015 in", N = "1800 rpm", W = "500 lbf", W_st = "500 lbf", "μ" = "4e-6 lbf·s/in²", T_av = "140 °F", h_v = "0.42", "φ" = "53 deg", f_v = "3.50", Q_v = "4.28", Q_r = "0.655", p_v = "0.42", "θ_pmax" = "18.5 deg", "θ_p0" = "75 deg" }
expect = { S = "0.135", h_0 = "0.00063 in", f = "0.0070", T_q = "2.62 lbf·in", H = "0.075 hp", Q = "0.217 in³/s", Q_s = "0.142 in³/s", p_max = "529 psi" }
tolerance = "0.3%"
checks = "fail"

[[tests]]
name = "Hand check: Exs. 12-1 to 12-4 exactly"
source = "By hand: P = 500/(1.5 × 1.5) = 222.222 psi, S = 500² × 4e-6 × 30/222.222 = 0.135, h0 = 0.42 × 0.0015 = 0.00063 in, f = 3.5 × 0.0015/0.75 = 0.007, T = 0.007 × 500 × 0.75 = 2.625 lbf·in, H = 2π × 2.625 × 30 lbf·in/s = 0.0749698 hp, Q = 4.28 × 0.75 × 0.0015 × 30 × 1.5 = 0.216675 in³/s, Qs = 0.141922 in³/s; ΔT = 4π × 3.5 × 222.222/((1 − 0.3275) × 4.28)/(0.0311 × 0.42 × 9338.03 lbf·in/Btu) = 27.83969 °F, T1 = 126.08016 °F, Tmax = 153.91984 °F; pmax = 529.10053 psi; Petroff f = 2π² × 5.4e-7 × 500 = 0.00532959; h0,min = 0.00026 in"
inputs = { d = "1.5 in", l = "1.5 in", c = "0.0015 in", N = "1800 rpm", W = "500 lbf", W_st = "500 lbf", "μ" = "4e-6 lbf·s/in²", T_av = "140 °F", h_v = "0.42", "φ" = "53 deg", f_v = "3.50", Q_v = "4.28", Q_r = "0.655", p_v = "0.42", "θ_pmax" = "18.5 deg", "θ_p0" = "75 deg" }
expect = { P = "222.222222 psi", S = "0.135", h_0 = "0.00063 in", "ε" = "0.58", f = "0.007", T_q = "2.625 lbf·in", H = "0.0749698247 hp", Q = "0.216675 in³/s", Q_s = "0.141922125 in³/s", "ΔT" = "27.8396864 Δ°F", T_1 = "126.080157 °F", T_max = "153.919843 °F", p_max = "529.100529 psi", f_P = "0.00532958638", "h_(0,min)" = "0.00026 in", "μNP" = "5.4e-7" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: the same bearing with a 16 μreyn oil, inside every criterion"
source = "By hand: S = 0.54; chart readings for l/d = 1 at that S (h0/c 0.77, (r/c)f 11.0, Q/(rcNl) 3.65, Qs/Q 0.314, P/pmax 0.524, from the Lacewing engine's interpolation of Raimondi and Boyd's table): h0 = 0.001155 in, f = 0.022, T = 8.25 lbf·in, H = 0.235619 hp, Q = 0.184781 in³/s, Qs = 0.0580213 in³/s, ΔT = 81.84735 °F, Tmax = 180.92367 °F at Tav 140 °F, pmax = 424.08821 psi, μN/P = 2.16e-6"
inputs = { d = "1.5 in", l = "1.5 in", c = "0.0015 in", N = "1800 rpm", W = "500 lbf", W_st = "500 lbf", "μ" = "16e-6 lbf·s/in²", T_av = "140 °F", h_v = "0.77", "φ" = "72.4 deg", f_v = "11.0", Q_v = "3.65", Q_r = "0.314", p_v = "0.524", "θ_pmax" = "10.5 deg", "θ_p0" = "104 deg" }
expect = { S = "0.54", h_0 = "0.001155 in", f = "0.022", T_q = "8.25 lbf·in", H = "0.235619449 hp", Q = "0.18478125 in³/s", Q_s = "0.0580213125 in³/s", "ΔT" = "81.8473455 Δ°F", T_max = "180.923673 °F", p_max = "424.08821 psi", "μNP" = "2.16e-6" }
tolerance = "1e-6"
checks = "pass"
+++

## Hydrodynamic journal bearing, Raimondi and Boyd

A full (360°) journal bearing running on a thick oil film under a steady load fixed in direction, analysed with Raimondi and Boyd's solution for the finite bearing as Shigley (Ch. 12) and Juvinall (Ch. 13) present it. Every performance variable is a function of the Sommerfeld number S and the length-to-diameter ratio l/d. This template computes S; you read the variables off the charts at that S and l/d, and it turns them into film thickness, friction, flow, temperature rise and pressure.

The viscosity is taken at a known average film temperature T_av, the books' simple case. For oil fed once from a sump at a known inlet temperature, iterate: guess T_av, take μ there, and repeat until T_1 = T_av − ΔT/2 comes back to the sump temperature (Shigley Sec. 12-8). For a pressure-fed bearing with a central groove, use the pressure-fed template.

### Bearing

```calc {description="Journal diameter"}
d = 1.5 in_
```

```calc {description="Bearing length"}
l = 1.5 in_
```

```calc {description="Radial clearance (half the bore less the journal diameter)"}
c = 0.0015 in_
```

```calc {description="Journal speed (the significant speed |Nj + Nb − 2Nf|, Shigley Eq. 12-13)"}
N = 1800 rpm_
```

```calc {description="Steady radial load"}
W = 500 lbf_
```

```calc {description="Load at starting and stopping (the running load if not known)"}
W_st = 500 lbf_
```

```calc
> Journal radius
r = d / 2 = ?? in
> Speed in revolutions per second, Shigley's N
n = N / (1 'rev') = ?? 1/s
> Unit load on the projected area
P = W / (l d) = ?? psi
> Length-to-diameter ratio: the charts' curve to read
λ = l / d = ??
```

### Lubricant

The absolute viscosity at the average film temperature, from the oil's viscosity-temperature chart or a fit (Shigley Fig. 12-12 or Table 12-1; see the lubricant viscosity templates). 1 μreyn is 10⁻⁶ lbf·s/in² (6.895 mPa·s). The volumetric heat capacity defaults to Shigley's oil: 0.0311 lbm/in³ and 0.42 Btu/(lbm·°F).

```calc {description="Absolute viscosity at the average film temperature"}
μ = 4e-6 lbf·s/in²_
```

```calc {description="Average film temperature (mean of inlet and outlet), Shigley Eq. 12-14"}
T_av = 140 °F_
```

```calc {description="Density of the oil"}
ρ_o = 0.0311 lbm/in³_
```

```calc {description="Specific heat of the oil"}
C_p = 0.42 Btu/(lbm·°F)_
```

```calc
> Sommerfeld number (bearing characteristic number), Eq. 12-7
S = (r / c)^2 μ n / P = ??
```

### Chart readings

Read each variable off Raimondi and Boyd's charts at the S above, on the curve for this l/d (interpolate between the l/d curves with the length-ratio template where needed): Shigley Figs. 12-16 to 12-22, Juvinall Figs. 13.13 to 13.19, or Harnoy's Table 8-1 of the same solution.

```calc {description="Minimum film thickness variable h0/c, Shigley Fig. 12-16"}
h_v = 0.42_
```

```calc {description="Position of the minimum film from the load line, Shigley Fig. 12-17"}
φ = 53°_
```

```calc {description="Friction variable (r/c)f, Shigley Fig. 12-18"}
f_v = 3.50_
```

```calc {description="Flow variable Q/(rcNl), Shigley Fig. 12-19"}
Q_v = 4.28_
```

```calc {description="Side flow ratio Qs/Q, Shigley Fig. 12-20"}
Q_r = 0.655_
```

```calc {description="Maximum film pressure ratio P/pmax, Shigley Fig. 12-21"}
p_v = 0.42_
```

```calc {description="Position of the maximum film pressure from the load line, Shigley Fig. 12-22"}
θ_pmax = 18.5°_
```

```calc {description="Position where the film pressure ends, Shigley Fig. 12-22"}
θ_p0 = 75°_
```

### Film, friction and power

```calc
> Minimum film thickness, Ex. 12-1
h_0 = h_v c = ?? in
> Eccentricity ratio e/c
ε = 1 - h_v = ??
> Eccentricity of the journal
e = ε c = ?? in
> Coefficient of friction
f = f_v c / r = ??
> Friction torque, Ex. 12-2
T_q = f W r = ?? lbf·in
> Power lost in friction, all of it heat in the film
H = 2 π T_q n = ?? hp
```

### Flow and temperature

The oil the journal pumps into the clearance is Q; Q_s leaks out of the ends and must be made up from the supply. Shigley's Eq. 12-15 has the side flow leave at the average temperature and the rest at the outlet temperature.

```calc
> Oil flow into the clearance, Ex. 12-3
Q = Q_v r c n l = ?? in³/s
> Side flow
Q_s = Q_r Q = ?? in³/s
> Temperature rise, Eq. 12-15
ΔT = 4 π f_v P / (ρ_o C_p (1 - Q_r / 2) Q_v) = ?? Δ°F
> Inlet temperature, Eq. 12-14
T_1 = T_av - ΔT / 2 = ?? °F
> Outlet (largest) oil temperature
T_max = T_av + ΔT / 2 = ?? °F
```

### Film pressure

```calc {description="Maximum film pressure, Ex. 12-4"}
p_max = P / p_v = ?? psi
```

### Petroff and the design criteria

Petroff's equation treats the journal as concentric (no load); it approaches Raimondi and Boyd's friction at light load and falls below it as the load rises. Trumpler's criteria for a sound design (Shigley Sec. 12-7, Juvinall Eq. 13.15) are a minimum film of 0.0002 + 0.00004 d in for a finely finished journal and clean oil, an outlet temperature under 250 °F for light oils, a starting unit load under 300 psi, and a design factor of 2 or more on the running load: to check that, run the template again at 2W and look at h_0 there. McKee's limit keeps the bearing on the thick-film side of minimum friction, where a temperature rise corrects itself.

```calc
> Petroff's coefficient of friction, Eq. 12-6
f_P = 2 π^2 (μ n / P) (r / c) = ??
> Trumpler's least film thickness
h_(0,min) = 0.0002 'in' + 0.00004 d = ?? in
> Starting unit load
P_st = W_st / (l d) = ?? psi
> Bearing characteristic μN/P (McKee)
μNP = μ n / P = ??
```

### Checks

```calc
@test h_0 ≥ h_(0,min)
```

```calc
@test T_max ≤ 250 '°F'
```

```calc
@test P_st ≤ 300 'psi'
```

```calc
@test μNP ≥ 1.7e-6
```
