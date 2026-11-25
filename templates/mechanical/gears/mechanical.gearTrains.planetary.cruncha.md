+++
cruncha-template = 1
id = "mechanical.gearTrains.planetary"
version = "1.0.0"
title = "Planetary gear train (Willis' equation)"
summary = "A planetary (epicyclic) train of sun, planets on an arm, and ring: the train value with the arm held, the third member's speed from the two you give by Willis' equation, the planets' absolute speed and their speed on the arm, the speed ratio between the input and output members, the torques on the three members of an ideal train, and the checks that equally spaced planets assemble and clear each other."
category = ["Mechanical engineering", "Gears"]
tags = ["gear train", "planetary", "epicyclic", "sun gear", "ring gear", "planet carrier", "arm", "Willis", "train value", "speed ratio", "overdrive", "Shigley", "Juvinall"]
from = "lacewing:mechanical.gearTrains"
license = "GPL-3.0-or-later"
inputs = ["S", "R", "n", "e_x", "solve", "n_S", "n_R", "n_A", "inp", "out", "T_in"]
outputs = ["e", "P", "ω_S", "ω_R", "ω_A", "ω_P", "ω_(P,rel)", "r_io", "T_S", "T_R", "T_A", "q_n", "c_tip"]

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 15.13: Eqs. 15.28 and 15.29 (Willis' equation), Figs. 15.30 to 15.33 (torques, assembly of equally spaced planets)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 13-13, Eq. 13-32; Example 13-6"

[[tests]]
name = "Shigley Ex. 13-6: sun 20, planets 30, ring 80 held, sun at 100 rev/min clockwise"
source = "Shigley 10th Example 13-6 (also Lacewing Lab gear_trains_test): e = −20/80 = −0.25; the arm turns at 20 rev/min clockwise and the planets at 33⅓ rev/min counterclockwise (clockwise negative here). Four planets are taken: (20 + 80)/4 = 25 assembles"
inputs = { S = "20", R = "80", n = "4", e_x = "0", solve = "3", n_S = "-100 rpm", n_R = "0 rpm", n_A = "0 rpm", inp = "1", out = "3", T_in = "0 N·m" }
expect = { e = "-0.25", P = "30", "ω_A" = "-20 rpm", "ω_P" = "33.333333 rpm", r_io = "5" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Juvinall Figs. 15.31 and 15.32: ring in, sun held, arm out"
source = "Juvinall 5th Sec. 15.13, Figs. 15.31 and 15.32 (also Lacewing Lab gear_trains_test): ωi/ωo = To/Ti = 1 + S/R; with S 30, R 90 and 10 N·m on the ring at 100 rev/min, the arm turns at 75 rev/min against a torque of −13.33 N·m and the held sun takes 3.33 N·m"
inputs = { S = "30", R = "90", n = "3", e_x = "0", solve = "3", n_S = "0 rpm", n_R = "100 rpm", n_A = "0 rpm", inp = "2", out = "3", T_in = "10 N·m" }
expect = { "ω_A" = "75 rpm", r_io = "1.3333333", T_A = "-13.333333 N·m", T_S = "3.3333333 N·m", T_R = "10 N·m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Juvinall Fig. 15.32: the overdrive, arm in, sun held, ring out"
source = "Juvinall 5th Sec. 15.13 (also Lacewing Lab gear_trains_test): the inverse ratio, the ring at (1 + S/R) times the arm, with 10/(1 + S/R) N·m on it"
inputs = { S = "30", R = "90", n = "3", e_x = "0", solve = "2", n_S = "0 rpm", n_R = "0 rpm", n_A = "100 rpm", inp = "3", out = "2", T_in = "10 N·m" }
expect = { "ω_R" = "133.33333 rpm", r_io = "0.75", T_R = "-7.5 N·m" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Juvinall Fig. 15.33: sun 20, ring 70, four planets will not assemble equally spaced"
source = "Juvinall 5th Sec. 15.13, Fig. 15.33 (also Lacewing Lab gear_trains_test): 25-tooth planets; (S + R)/n = 22.5 is not whole for four planets (two give 45 and assemble)"
inputs = { S = "20", R = "70", n = "4", e_x = "0", solve = "3", n_S = "100 rpm", n_R = "0 rpm", n_A = "0 rpm", inp = "1", out = "3", T_in = "0 N·m" }
expect = { P = "25", q_n = "22.5" }
tolerance = "1e-9"
checks = "fail"

[[tests]]
name = "Hand check: five 25-tooth planets around the 20-tooth sun overlap"
source = "Lacewing Lab gear_trains_test and by hand: (S + P) sin(π/5) − (P + 2) = 45 × 0.587785 − 27 = −0.549663 modules; (S + R)/5 = 18 is whole, but the tips collide"
inputs = { S = "20", R = "70", n = "5", e_x = "0", solve = "3", n_S = "100 rpm", n_R = "0 rpm", n_A = "0 rpm", inp = "1", out = "3", T_in = "0 N·m" }
expect = { q_n = "18", c_tip = "-0.5496635" }
tolerance = "1e-6"
checks = "fail"
+++

## Planetary gear train

A planetary train has three coaxial members: the sun gear, the ring gear and the arm (carrier) on which the planets turn. With the arm held it is an ordinary train, whose train value e = (ω_R − ω_A)/(ω_S − ω_A) relates the speeds relative to the arm (Willis' equation, Juvinall Eqs. 15.28 and 15.29, Shigley Eq. 13-32). Any two speeds give the third; a held member has speed 0. Speeds are signed: one sense positive, the other negative.

For the simple train (one sun, one ring, planets meshing both) e = −S/R. For stepped (compound) planets, or two suns, enter your own train value e_x for the two members you call sun and ring; the planet speed and assembly checks then don't apply. The torques are those of an ideal train, with no losses: they sum to zero, and in the arm's frame the power balances, so T_R = −T_S/e (Juvinall Fig. 15.31). A bevel differential is this train with e = −1; it has its own template.

### Teeth

```calc {description="Sun teeth"}
S = 20_
```

```calc {description="Ring teeth"}
R = 80_
```

```calc {description="Number of equally spaced planets"}
n = 4_
```

```calc {description="Train value of your own, ring over sun with the arm held (0: the simple train's −S/R)"}
e_x = 0_
```

```calc
> Train value with the arm held
e = {-S / R if e_x == 0; e_x otherwise} = ??
> Planet teeth, for a coaxial simple train of one module
P = (R - S) / 2 = ??
```

### Speeds

Give the speeds of two members and choose the one to solve for: 1 the sun, 2 the ring, 3 the arm. The speed given for the member being solved is ignored.

```calc {description="Member to solve for: 1 sun, 2 ring, 3 arm"}
solve = 3_
```

```calc {description="Sun speed (signed)"}
n_S = -100 rpm_
```

```calc {description="Ring speed (signed; 0 if held)"}
n_R = 0 rpm_
```

```calc {description="Arm (carrier) speed (signed; 0 if held)"}
n_A = 0 rpm_
```

```calc
> Sun speed, Willis' equation
ω_S = {n_A + (n_R - n_A) / e if solve == 1; n_S otherwise} = ?? rpm
> Ring speed
ω_R = {n_A + e (n_S - n_A) if solve == 2; n_R otherwise} = ?? rpm
> Arm speed
ω_A = {(n_R - e n_S) / (1 - e) if solve == 3; n_A otherwise} = ?? rpm
> Planet speed (absolute), Shigley Ex. 13-6
ω_P = ω_A - (ω_S - ω_A) S / P = ?? rpm
> Planet speed relative to the arm
ω_(P,rel) = ω_P - ω_A = ?? rpm
```

### Input, output and torques

```calc {description="Input member: 1 sun, 2 ring, 3 arm"}
inp = 1_
```

```calc {description="Output member: 1 sun, 2 ring, 3 arm"}
out = 3_
```

```calc {description="Torque applied to the input member (signed like its speed; 0 for none)"}
T_in = 0 N·m_
```

```calc
> Speed ratio, input over output (negative: opposite senses)
r_io = {ω_S if inp == 1; ω_R if inp == 2; ω_A if inp == 3} / {ω_S if out == 1; ω_R if out == 2; ω_A if out == 3} = ??
> Torque on the sun, ideal train
T_S = {T_in if inp == 1; -T_in e if inp == 2; -T_in / (1 - 1 / e) if inp == 3} = ?? N·m
> Torque on the ring
T_R = -T_S / e = ?? N·m
> Torque on the arm
T_A = -(T_S + T_R) = ?? N·m
```

### Assembly

Equally spaced planets mesh with sun and ring only if (S + R)/n is a whole number (Juvinall Sec. 15.13, Fig. 15.33). Their tip circles, of a standard one-module addendum, clear each other when the centre spacing (S + P) sin(π/n) modules exceeds the planet's outside diameter P + 2 (a geometric check, not a formula of either book). A coaxial train of one module needs R − S even.

```calc
> Assembly number
q_n = (S + R) / n = ??
> Tip clearance between adjacent planets, in modules
c_tip = (S + P) sin(π / n) - (P + 2) = ??
```

```calc
@test mod(R - S, 2) == 0
```

```calc
@test mod(S + R, n) == 0
```

```calc
@test c_tip > 0
```
