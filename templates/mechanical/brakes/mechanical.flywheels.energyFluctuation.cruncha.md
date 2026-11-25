+++
cruncha-template = 1
id = "mechanical.flywheels.energyFluctuation"
version = "1.0.0"
title = "Flywheel for an energy fluctuation, with a flat disk's size and stresses (Shigley)"
summary = "The flywheel inertia that holds a cycle's energy fluctuation to a coefficient of speed fluctuation, the fastest and slowest speeds and the kinetic energy at the fastest; then a flat disk (solid or with a bore) of that inertia: its thickness, mass, rim speed and the largest tangential and radial stresses at the fastest speed."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["flywheel", "energy fluctuation", "coefficient of speed fluctuation", "inertia", "rotating disk", "hoop stress", "rim speed", "Shigley"]
from = "lacewing:mechanical.flywheels"
license = "GPL-3.0-or-later"
inputs = ["ΔE", "C_s", "n", "ρ", "r_o", "r_i", "ν", "σ_allow"]
outputs = ["I", "n_2", "n_1", "E_2", "t", "m", "v", "σ_t", "σ_r"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-12: Eqs. 16-61 to 16-64, Fig. 16-28; Example 16-6. Sec. 3-15: Eq. 3-55 (stresses in a rotating disk)"

[[tests]]
name = "Shigley Ex. 16-6: 3531 lbf·in at 250 rad/s, Cs = 0.1"
source = "Lacewing Lab brakes_test, after Shigley 10th Example 16-6: I = 0.565 lbf·s²·in, ω2 = 262.5 and ω1 = 237.5 rad/s. The disk is not in the example: a 6 in steel disk (0.282 lbm/in³) of that inertia, by hand"
inputs = { "ΔE" = "3531 lbf·in", C_s = "0.1", n = "250 rad/s", "ρ" = "0.282 lb/in³", r_o = "6 in", r_i = "0 in", "ν" = "0.29", "σ_allow" = "20 ksi" }
expect = { I = "0.565 lbf·s²·in", n_2 = "262.5 rad/s", n_1 = "237.5 rad/s" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: a cast-iron disk with a bore"
source = "Lacewing Lab brakes_test (disk sizing), by hand: 5000 J, Cs 0.05, 150 rad/s: I = 4.4444444 kg·m², ω2 = 153.75 rad/s; ρ 7200 kg/m³, ro 0.3, ri 0.1 m, ν 0.26: t = 2I/(πρ(ro⁴ − ri⁴)) = 49.121896 mm, m = 88.888889 kg, v = 46.125 m/s, σt at the bore = ¼ρω2²((3 + ν)ro² + (1 − ν)ri²) = 12.799134 MPa, σr at √(ri ro) = (3 + ν)ρω2²(ro − ri)²/8 = 2.7742804 MPa. At 49 mm the disk is thicker than ro/10, so the thin-disk check fails"
inputs = { "ΔE" = "5000 J", C_s = "0.05", n = "150 rad/s", "ρ" = "7200 kg/m³", r_o = "0.3 m", r_i = "0.1 m", "ν" = "0.26", "σ_allow" = "20 MPa" }
expect = { I = "4.4444444 kg·m²", n_2 = "153.75 rad/s", n_1 = "146.25 rad/s", E_2 = "52531.25 J", t = "49.121896 mm", m = "88.888889 kg", v = "46.125 m/s", "σ_t" = "12.799134 MPa", "σ_r" = "2.7742804 MPa" }
tolerance = "1e-7"
checks = "fail"

[[tests]]
name = "Hand check: the same as a solid disk, over a 5 MPa allowable"
source = "By hand: ri = 0: t = 48.515453 mm, m = 98.765432 kg, σt = σr at the centre = (3 + ν)ρω2²ro²/8 = 6.2421308 MPa, over 5 MPa"
inputs = { "ΔE" = "5000 J", C_s = "0.05", n = "150 rad/s", "ρ" = "7200 kg/m³", r_o = "0.3 m", r_i = "0 m", "ν" = "0.26", "σ_allow" = "5 MPa" }
expect = { t = "48.515453 mm", m = "98.765432 kg", "σ_t" = "6.2421308 MPa", "σ_r" = "6.2421308 MPa" }
tolerance = "1e-7"
checks = "fail"
+++

## Flywheel for an energy fluctuation

A flywheel stores the energy a machine's torque fluctuation gives up and takes back in each cycle (Shigley Sec. 16-12). If ΔE = E_2 − E_1 is the largest swing of energy in a cycle, the biggest loop of the torque-angle diagram above or below the mean torque (Fig. 16-28), and the speed may vary by the coefficient of speed fluctuation C_s = (ω_2 − ω_1)/ω about the mean ω = (ω_1 + ω_2)/2, then ½ I (ω_2² − ω_1²) = C_s I ω² gives the inertia (Eq. 16-64). The inertia is everything that turns with the flywheel, referred to its shaft. Example 16-6 finds ΔE by integrating a tabulated torque-angle cycle with the trapezoidal rule; do that in a Python cell or by hand and enter the result.

A flat disk of constant thickness then gives that inertia; its stresses at the fastest speed are those of a thin rotating disk (Shigley Eq. 3-55): with a bore, the tangential (hoop) stress is largest at the bore and the radial stress at r = √(r_i r_o); a solid disk has both largest, and equal, at the centre. A bore, however small, doubles the hoop stress. The disk equations hold for r_o of at least ten times t, and ignore spokes, hub and keyway.

### Inputs

```calc {description="Largest energy fluctuation in a cycle, E2 − E1"}
ΔE = 3531 lbf·in_
```

```calc {description="Coefficient of speed fluctuation allowed, (ω2 − ω1)/ω"}
C_s = 0.1_
```

```calc {description="Mean (nominal) speed"}
n = 250 rad/s_
```

```calc {description="Density of the disk"}
ρ = 0.282 lb/in³_
```

```calc {description="Outer radius of the disk"}
r_o = 6 in_
```

```calc {description="Bore radius (0 for a solid disk)"}
r_i = 0 in_
```

```calc {description="Poisson's ratio of the disk material"}
ν = 0.29_
```

```calc {description="Allowable tensile stress at the fastest speed, for the material and factor of safety chosen"}
σ_allow = 20 ksi_
```

### Inertia and speeds

```calc
> Inertia that holds the fluctuation to C_s, Eq. 16-64
I = ΔE / (C_s n^2) = ?? lbf·s²·in
> Fastest speed, Eqs. 16-62 and 16-63
n_2 = n (2 + C_s) / 2 = ?? rpm
> Slowest speed
n_1 = 2 n - n_2 = ?? rpm
> Kinetic energy at the fastest, Eq. 16-61
E_2 = I n_2^2 / 2 = ?? lbf·in
```

### A flat disk of that inertia

```calc
> Thickness for the inertia
t = 2 I / (π ρ (r_o^4 - r_i^4)) = ?? in
> Mass
m = π ρ t (r_o^2 - r_i^2) = ?? lb
> Rim speed at the fastest
v = n_2 r_o = ?? ft/s
> Largest tangential stress at the fastest speed, Eq. 3-55: at the centre (solid) or at the bore
σ_t = {(3 + ν) ρ n_2^2 r_o^2 / 8 if r_i == 0 'in'; ρ n_2^2 ((3 + ν) r_o^2 + (1 - ν) r_i^2) / 4 otherwise} = ?? psi
> Largest radial stress, Eq. 3-55: at the centre (solid) or at √(r_i r_o)
σ_r = (3 + ν) ρ n_2^2 (r_o - r_i)^2 / 8 = ?? psi
```

### Checks

```calc
@test σ_t ≤ σ_allow
```

```calc
@test t ≤ r_o / 10
```
