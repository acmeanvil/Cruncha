+++
cruncha-template = 1
id = "mechanical.journalBearings.pressureFed"
version = "1.0.0"
title = "Pressure-fed journal bearing, central groove (one temperature trial)"
summary = "One trial of the film temperature of a pressure-fed journal bearing with a central circumferential groove: unit load on each land, Sommerfeld number, Raimondi and Boyd's chart readings at that S, the side flow the supply pressure drives, the friction heat it carries away, the temperature rise, the film temperature it implies and whether the trial has converged, with the film thickness, maximum film pressure and Trumpler's checks."
category = ["Mechanical engineering", "Bearings"]
tags = ["journal bearing", "pressure-fed", "forced lubrication", "circumferential groove", "hydrodynamic bearing", "side flow", "temperature rise", "Raimondi", "Boyd", "Sommerfeld number", "Trumpler", "Shigley"]
from = "lacewing:mechanical.journalBearings"
license = "GPL-3.0-or-later"
inputs = ["d", "l′", "c", "N", "W", "p_s", "T_s", "T_av", "μ", "ρ_o", "C_p", "δT", "ε", "f_v", "p_v"]
outputs = ["r", "P", "S", "h_0", "f", "T_q", "H", "Q_s", "ΔT", "T_max", "T_(av,2)", "p_max", "h_(0,min)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 12-11, Pressure-fed bearings: Eqs. 12-22 (side flow), 12-23 (unit load on each land), 12-24 and Eq. (o) (temperature rise); Sec. 12-7 (Trumpler); Figs. 12-16, 12-18, 12-21 (Raimondi and Boyd's charts); Example 12-6"

[[references]]
work = "Raimondi and Boyd, A Solution for the Finite Journal Bearing and Its Application to Analysis and Design, Parts I-III, Trans. ASLE vol. 1"
edition = "1958"
locator = "Part III, the full bearing (tabulated in Harnoy, Bearing Design in Machinery, 2003, Table 8-1)"

[[tests]]
name = "Shigley Ex. 12-6: SAE 20 at 30 psi, d 1.750 in, c 0.0015 in, l′/d ½, 3000 rpm, 900 lbf, sump 120 °F, the converged trial"
source = "Lacewing Lab journal_bearings_test, after Shigley 10th Example 12-6: Tf 168.5 °F (SAE 20 1.693 μreyn there by Seireg and Dandage's fit, Lacewing lubricants_test), ΔT 97.1 °F, Tmax 217.1 °F, h0 0.000312 in, Qs 0.123 in³/s, Hloss 0.156 Btu/s, T 4.58 lbf·in; Trumpler's h0 0.000270 in. The chart readings at S = 0.098 (ε 0.793, (r/c)f 3.40, P/pmax 0.271) are the Lacewing engine's interpolation of Raimondi and Boyd's table"
inputs = { d = "1.750 in", "l′" = "0.875 in", c = "0.0015 in", N = "3000 rpm", W = "900 lbf", p_s = "30 psi", T_s = "120 °F", T_av = "168.5 °F", "μ" = "1.693e-6 lbf·s/in²", "ε" = "0.793", f_v = "3.40", p_v = "0.271" }
expect = { P = "294 psi", "ΔT" = "97.1 Δ°F", T_max = "217.1 °F", h_0 = "0.000312 in", Q_s = "0.123 in³/s", H = "0.156 Btu/s", T_q = "4.58 lbf·in", "h_(0,min)" = "0.000270 in", "T_(av,2)" = "168.5 °F" }
tolerance = "1.5%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 12-6's converged trial exactly"
source = "By hand: P = 900/(4 × 0.875 × 0.875) = 293.87755 psi, S = 583.333² × 1.693e-6 × 50/293.87755 = 0.0980154, Qs = π × 30 × 0.875 × 0.0015³ × (1 + 1.5 × 0.793²)/(3 × 1.693e-6 × 0.875) = 0.1217028 in³/s, f = 3.4 × 0.0015/0.875 = 0.00582857, T = 4.59 lbf·in, H = 2π × 4.59 × 50 lbf·in/s = 0.2184835 hp, ΔT = H/(ρCp·Qs) = 97.13974 °F, Tmax = 217.13974 °F, Tav,2 = 168.56987 °F, h0 = 0.0003105 in, pmax = 293.87755/0.271 + 30 = 1114.4190 psi"
inputs = { d = "1.750 in", "l′" = "0.875 in", c = "0.0015 in", N = "3000 rpm", W = "900 lbf", p_s = "30 psi", T_s = "120 °F", T_av = "168.5 °F", "μ" = "1.693e-6 lbf·s/in²", "ε" = "0.793", f_v = "3.40", p_v = "0.271" }
expect = { P = "293.877551 psi", S = "0.0980153598", Q_s = "0.121702814 in³/s", f = "0.00582857143", T_q = "4.59 lbf·in", H = "0.218483489 hp", "ΔT" = "97.1397409 Δ°F", T_max = "217.139741 °F", "T_(av,2)" = "168.56987 °F", h_0 = "0.0003105 in", p_max = "1114.41901 psi" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Ex. 12-6 tried first at 140 °F: far from converged"
source = "By hand: SAE 20 at 140 °F is 3.045 μreyn (Seireg and Dandage's fit), S = 0.1763; chart readings there for l′/d = ½ (ε 0.707, (r/c)f 5.17, P/pmax 0.316, the Lacewing engine's interpolation): Qs = 0.0609282 in³/s, ΔT = 295.0469 °F, so the film would run at 267.52 °F, not 140 °F, and the outlet at 415 °F"
inputs = { d = "1.750 in", "l′" = "0.875 in", c = "0.0015 in", N = "3000 rpm", W = "900 lbf", p_s = "30 psi", T_s = "120 °F", T_av = "140 °F", "μ" = "3.045e-6 lbf·s/in²", "ε" = "0.707", f_v = "5.17", p_v = "0.316" }
expect = { Q_s = "0.0609281775 in³/s", "ΔT" = "295.046861 Δ°F", "T_(av,2)" = "267.523431 °F" }
tolerance = "1e-6"
checks = "fail"
+++

## Pressure-fed journal bearing, central groove

A full journal bearing fed with oil under pressure through a circumferential groove at its middle, after Shigley Sec. 12-11. The groove splits the bearing into two lands of length l′, each carrying half the load, and the supply pressure drives the side flow out of both ends. That flow carries away the whole friction heat, so the temperature rise follows from the heat and the flow rather than from the charts' flow variables, which do not apply here.

The viscosity depends on the film temperature, which depends on the friction: the solution is a trial. Pick a trial T_av, take the oil's viscosity there (see the lubricant viscosity templates), read the charts at the S this gives, and compare the film temperature T_(av,2) the heat balance returns with the trial. Shigley's Ex. 12-6 converges in two or three trials; a new trial between the two usually does it.

### Bearing

```calc {description="Journal diameter"}
d = 1.750 in_
```

```calc {description="Length of each land beside the groove"}
l′ = 0.875 in_
```

```calc {description="Radial clearance"}
c = 0.0015 in_
```

```calc {description="Journal speed"}
N = 3000 rpm_
```

```calc {description="Radial load on the whole bearing"}
W = 900 lbf_
```

```calc
> Journal radius
r = d / 2 = ?? in
> Speed in revolutions per second
n = N / (1 'rev') = ?? 1/s
> Unit load: each land carries half the load, Eq. 12-23
P = W / (4 r l′) = ?? psi
> Land length to diameter: the charts' curve to read
λ = l′ / d = ??
```

### Supply and oil

```calc {description="Supply pressure at the groove (gauge)"}
p_s = 30 psi_
```

```calc {description="Supply (sump) temperature"}
T_s = 120 °F_
```

```calc {description="Trial average film temperature"}
T_av = 168.5 °F_
```

```calc {description="Absolute viscosity at the trial film temperature"}
μ = 1.693e-6 lbf·s/in²_
```

```calc {description="Density of the oil"}
ρ_o = 0.0311 lbm/in³_
```

```calc {description="Specific heat of the oil"}
C_p = 0.42 Btu/(lbm·°F)_
```

```calc {description="How close the returned film temperature must come to the trial"}
δT = 2 Δ°F_
```

```calc {description="Sommerfeld number at the trial viscosity, Eq. 12-7"}
S = (r / c)^2 μ n / P = ??
```

### Chart readings

At the S above, on the curve for l′/d (Shigley Figs. 12-16, 12-18 and 12-21, or Harnoy Table 8-1).

```calc {description="Eccentricity ratio ε = 1 − h0/c, Shigley Fig. 12-16"}
ε = 0.793_
```

```calc {description="Friction variable (r/c)f, Shigley Fig. 12-18"}
f_v = 3.40_
```

```calc {description="Maximum film pressure ratio P/pmax, Shigley Fig. 12-21"}
p_v = 0.271_
```

### Heat balance

```calc
> Side flow driven by the supply pressure, both ends, Eq. 12-22
Q_s = π p_s r c^3 (1 + 1.5 ε^2) / (3 μ l′) = ?? in³/s
> Coefficient of friction
f = f_v c / r = ?? 
> Friction torque on the whole journal
T_q = f W r = ?? lbf·in
> Friction heat, all carried off by the side flow
H = 2 π T_q n = ?? Btu/s
> Temperature rise, Eq. (o) of Sec. 12-11
ΔT = H / (ρ_o C_p Q_s) = ?? Δ°F
> Outlet temperature
T_max = T_s + ΔT = ?? °F
> Film temperature the heat balance returns
T_(av,2) = T_s + ΔT / 2 = ?? °F
```

### Film and pressure

```calc
> Minimum film thickness
h_0 = (1 - ε) c = ?? in
> Maximum film pressure, the supply pressure added
p_max = P / p_v + p_s = ?? psi
> Trumpler's least film thickness
h_(0,min) = 0.0002 'in' + 0.00004 d = ?? in
```

### Checks

The trial must have converged before the other results mean anything. Trumpler's criteria follow (Shigley Sec. 12-7): the film, an outlet under 250 °F, and a unit load under 300 psi; for his factor of 2 on the load, run again at 2W.

```calc
@test abs(T_(av,2) - T_av) ≤ δT
```

```calc
@test h_0 ≥ h_(0,min)
```

```calc
@test T_max ≤ 250 '°F'
```

```calc
@test P ≤ 300 'psi'
```
