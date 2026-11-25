+++
cruncha-template = 1
id = "mechanics.capstan.wrap"
version = "1.0.0"
title = "Capstan: wrap needed to hold a load"
summary = "The wrap a rope needs around a drum for a slack-side pull T0 to hold a load T1 by friction, φ = ln(T1/T0)/f from the capstan equation, in radians, degrees and turns."
category = ["Mechanical engineering", "Mechanics"]
tags = ["capstan", "rope friction", "belt friction", "Euler-Eytelwein", "wrap angle", "turns", "drum", "bollard", "winch", "mooring", "tension ratio", "Den Hartog"]
from = "lacewing:mechanics.capstan"
license = "GPL-3.0-or-later"
inputs = ["T_0", "T_1", "f"]
outputs = ["T_ratio", "φ", "φ_deg", "n"]

[[references]]
work = "Den Hartog, J.P., Mechanics (Dover reprint)"
edition = "1948"
locator = "§20i, rope friction on a drum, and its table of T1/T0 at f = 0.5"

[[tests]]
name = "Den Hartog §20i: a ratio of 23.1 at f = 0.5 takes one turn"
source = "Lacewing Lab mechanics_test, after Den Hartog §20i: T1/T0 = 23.1 for one full turn at f = 0.5, so φ = ln 23.1/0.5 = 2π"
inputs = { T_0 = "1 N", T_1 = "23.1 N", f = "0.5" }
expect = { "φ" = "6.2832", n = "1" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: 200 N pull holding 1 kN at f = 0.25"
source = "By hand: φ = ln 5/0.25 = 6.4377516 rad = 368.85600°, 1.0246 turns"
inputs = { T_0 = "200 N", T_1 = "1000 N", f = "0.25" }
expect = { T_ratio = "5", "φ" = "6.4377516", "φ_deg" = "368.85600", n = "1.0246000" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "A load smaller than the pull"
source = "By hand: T1 = 150 N < T0 = 200 N needs no wrap; the formula gives a negative angle"
inputs = { T_0 = "200 N", T_1 = "150 N", f = "0.25" }
expect = { T_ratio = "0.75" }
tolerance = "1e-9"
checks = "fail"
+++

## Capstan: wrap needed to hold a load

The capstan equation T₁/T₀ = e^(fφ) (Den Hartog §20i), solved for the wrap angle: the turns of rope around a drum, bollard or winch barrel for a pull T₀ to hold a load T₁ at the point of slipping. The drum's radius does not enter. The rope is taken as weightless and perfectly flexible.

Friction varies, and the wrap needed is inversely proportional to it: use a conservative (low) coefficient and add a margin of wrap.

```calc {description="Slack-side pull: the hand's pull or the belt's slack side"}
T_0 = 200 N_
```

```calc {description="Load to hold: the tension on the loaded side"}
T_1 = 1000 N_
```

```calc {description="Coefficient of friction between rope and drum, at slipping"}
f = 0.25_
```

```calc
> Tension ratio
T_ratio = T_1 / T_0 = ??
> Wrap angle needed, §20i
φ = ln(T_ratio) / f = ??
> In degrees
φ_deg = φ × 180 / π = ??
> In full turns
n = φ / (2 π) = ??
```

### Check

The load must be more than the pull.

```calc
@test T_1 > T_0
```
