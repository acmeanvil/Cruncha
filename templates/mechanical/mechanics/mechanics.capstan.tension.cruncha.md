+++
cruncha-template = 1
id = "mechanics.capstan.tension"
version = "1.0.0"
title = "Capstan: tension from the wrap"
summary = "The capstan (Euler–Eytelwein) equation for a rope, belt or band slipping over a drum: the tight-side tension T1 = T0·e^(fφ) for a wrap of n turns, the tension ratio and the friction force on the drum."
category = ["Mechanical engineering", "Mechanics"]
tags = ["capstan", "rope friction", "belt friction", "Euler-Eytelwein", "wrap angle", "drum", "bollard", "winch", "band brake", "tension ratio", "Den Hartog"]
from = "lacewing:mechanics.capstan"
license = "GPL-3.0-or-later"
inputs = ["T_0", "f", "n"]
outputs = ["φ", "T_ratio", "T_1", "F_f"]

[[references]]
work = "Den Hartog, J.P., Mechanics (Dover reprint)"
edition = "1948"
locator = "§20i, rope friction on a drum, and its table of T1/T0 at f = 0.5"

[[tests]]
name = "Den Hartog §20i: one turn at f = 0.5 multiplies the pull about 23 times"
source = "Lacewing Lab mechanics_test, after Den Hartog §20i: T1/T0 = 23.1 for one turn at f = 0.5"
inputs = { T_0 = "1 N", f = "0.5", n = "1" }
expect = { T_ratio = "23.1", T_1 = "23.1 N" }
tolerance = "0.5%"

[[tests]]
name = "Den Hartog §20i: three turns at f = 0.5"
source = "Lacewing Lab mechanics_test, after Den Hartog §20i: T1/T0 = 12 392 for three turns at f = 0.5"
inputs = { T_0 = "1 N", f = "0.5", n = "3" }
expect = { T_ratio = "12392" }
tolerance = "0.01%"

[[tests]]
name = "Hand check: Lacewing Lab's opening case, 200 N pull, f = 0.25, two turns"
source = "By hand: φ = 4π = 12.566371 rad, T1/T0 = e^π = 23.140693, T1 = 4 628.1385 N, friction force 4 428.1385 N"
inputs = { T_0 = "200 N", f = "0.25", n = "2" }
expect = { "φ" = "12.566371", T_ratio = "23.140693", T_1 = "4628.1385 N", F_f = "4428.1385 N" }
tolerance = "1e-7"
+++

## Capstan: tension from the wrap

A flexible rope over a fixed drum, at the point of slipping. An element of rope over the drum presses on it with a force T dφ and so carries a friction force f T dφ; integrating dT/T = f dφ over the wrap angle φ gives T₁/T₀ = e^(fφ) (Den Hartog §20i). The drum's radius does not enter. The same law governs a belt slipping on a pulley and a band brake, where T₁ − T₀ times the drum radius is the braking torque.

The rope is taken as weightless and perfectly flexible. The result is very sensitive to the friction coefficient, which sits in the exponent: for a hold, use a conservative (low) value.

```calc {description="Slack-side pull: the hand's pull or the belt's slack side"}
T_0 = 200 N_
```

```calc {description="Coefficient of friction between rope and drum, at slipping"}
f = 0.25_
```

```calc {description="Wrap in full turns (1 turn = 360°; 0.5 for half a turn of contact)"}
n = 2_
```

```calc
> Wrap angle
φ = 2 π n = ??
> Tension ratio, §20i
T_ratio = exp(f φ) = ??
> Tight-side tension the rope holds before it slips
T_1 = T_0 T_ratio = ?? N
> Friction force the drum takes
F_f = T_1 - T_0 = ?? N
```
