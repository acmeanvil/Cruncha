+++
cruncha-template = 1
id = "mechanical.diskClutches.annularPad"
version = "1.0.0"
title = "Caliper disk brake, annular pads (Shigley)"
summary = "A caliper disk brake with annular-sector pads, by uniform wear: the largest pad pressure a required brake torque needs, the actuating force on each pad, the equivalent and force-location radii (worn and new), and the hydraulic pressure in the cylinder behind each pad."
category = ["Mechanical engineering", "Clutches and brakes"]
tags = ["brake", "disk brake", "caliper brake", "annular pad", "uniform wear", "uniform pressure", "hydraulic pressure", "pad", "Shigley"]
from = "lacewing:mechanical.diskClutches"
license = "GPL-3.0-or-later"
inputs = ["r_o", "r_i", "θ_p", "f", "T", "n_p", "d_c", "p_allow"]
outputs = ["T_pad", "p_a", "F", "r_e", "r̄", "r_(e,new)", "r̄_new", "p_h"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 16-6: Fig. 16-19, Eqs. 16-33 to 16-36 (uniform wear), 16-39 and 16-40 (uniform pressure); Table 16-3 (friction materials); Example 16-3"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 18.3 (caliper disk brakes sized with the plate-clutch equations); Tables 18.1 and 18.2"

[[tests]]
name = "Shigley Ex. 16-3: two annular pads, 13 000 lbf·in, 1.5 in cylinders"
source = "Lacewing Lab brakes_test, after Shigley 10th Example 16-3: ro 5.5, ri 3.875 in, 108° pads, f 0.37: pa 315.8 psi, F 3748 lbf, re 4.688 in, r̄ 4.024 in, hydraulic pressure 2121 psi"
inputs = { r_o = "5.5 in", r_i = "3.875 in", "θ_p" = "108°", f = "0.37", T = "13000 lbf·in", n_p = "2", d_c = "1.5 in", p_allow = "400 psi" }
expect = { p_a = "315.8 psi", F = "3748 lbf", r_e = "4.688 in", "r̄" = "4.024 in", p_h = "2121 psi" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 16-3 at 17 000 lbf·in: over the allowable pressure"
source = "By hand: pa scales with the torque, 315.75058 × 17000/13000 = 412.90460 psi, over 400 psi"
inputs = { r_o = "5.5 in", r_i = "3.875 in", "θ_p" = "108°", f = "0.37", T = "17000 lbf·in", n_p = "2", d_c = "1.5 in", p_allow = "400 psi" }
expect = { p_a = "412.90460 psi" }
tolerance = "0.01%"
checks = "fail"

[[tests]]
name = "Hand check: new-pad radii, Eqs. 16-39 and 16-40"
source = "By hand: ro 5.5, ri 3.875 in, 108° = 1.8849556 rad: re new = 2(5.5³ − 3.875³)/(3(5.5² − 3.875²)) = 4.7344444 in; (cos θ1 − cos θ2)/(θ2 − θ1) = 2 sin 54°/1.8849556 = 0.85839369; r̄ new = 4.0640172 in"
inputs = { r_o = "5.5 in", r_i = "3.875 in", "θ_p" = "108°", f = "0.37", T = "13000 lbf·in", n_p = "2", d_c = "1.5 in", p_allow = "400 psi" }
expect = { "r_(e,new)" = "4.7344444 in", "r̄_new" = "4.0640172 in" }
tolerance = "1e-6"
checks = "pass"
+++

## Caliper disk brake, annular pads

A disk gripped by pads shaped as annular sectors between radii r_i and r_o, subtending the angle θ_p (Shigley Sec. 16-6, Fig. 16-19). The pads share the brake torque equally; a single caliper has two, one on each side of the disk. The design basis is uniform wear, a worn-in pad with p·r constant and the largest pressure p_a at the inner radius. A new pad has nearly uniform pressure, which moves the friction radius out a little: the new-pad radii are given for comparison.

### Inputs

The friction coefficient and the allowable pressure are the lining's, from Shigley Table 16-3 or Juvinall Tables 18.1 and 18.2.

```calc {description="Outer radius of the pads"}
r_o = 5.5 in_
```

```calc {description="Inner radius of the pads"}
r_i = 3.875 in_
```

```calc {description="Angle each pad subtends, θ2 − θ1, centred on the pad's axis"}
θ_p = 108°_
```

```calc {description="Dynamic friction coefficient of the lining, Shigley Table 16-3"}
f = 0.37_
```

```calc {description="Brake torque, all pads"}
T = 13000 lbf·in_
```

```calc {description="Number of pads sharing the torque"}
n_p = 2_
```

```calc {description="Bore of the hydraulic cylinder behind each pad"}
d_c = 1.5 in_
```

```calc {description="Allowable pressure of the lining, Shigley Table 16-3"}
p_allow = 400 psi_
```

### Pressure, force and radii

```calc
> Torque each pad carries
T_pad = T / n_p = ?? lbf·in
> Largest pressure, at the inner radius, from the pad torque, Eq. 16-34
p_a = 2 T_pad / (θ_p f r_i (r_o^2 - r_i^2)) = ?? psi
> Actuating force on each pad, Eq. 16-33
F = θ_p p_a r_i (r_o - r_i) = ?? lbf
> Equivalent (friction) radius, T_pad = f F r_e, Eq. 16-35
r_e = (r_o + r_i) / 2 = ?? in
> Radius of the line of action of F, where the piston goes, Eq. 16-36
r̄ = 2 sin(θ_p / 2) / θ_p r_e = ?? in
```

```calc
> Friction radius of a new pad, uniform pressure, Eq. 16-39
r_(e,new) = 2 (r_o^3 - r_i^3) / (3 (r_o^2 - r_i^2)) = ?? in
> Line of action of F on a new pad, Eq. 16-40
r̄_new = 2 sin(θ_p / 2) / θ_p r_(e,new) = ?? in
```

```calc {description="Hydraulic pressure that makes F in one cylinder"}
p_h = F / (π d_c^2 / 4) = ?? psi
```

### Checks

```calc
@test p_a ≤ p_allow
```
