+++
cruncha-template = 1
id = "dynamics.balancing.singlePlane"
version = "1.0.0"
title = "Single-plane balancing with one trial weight"
summary = "The correction weight for a narrow rotor (a disk, fan, pulley or grinding wheel) balanced in one plane from two runs at the balancing speed, as found and with a trial weight, each read as amplitude and phase at one bearing: the complex influence number, the unbalance and the weight to add, with its angle."
category = ["Mechanical engineering", "Vibrations"]
tags = ["balancing", "single-plane balancing", "static balancing", "field balancing", "influence coefficient", "trial weight", "correction weight", "unbalance", "rotor", "fan", "complex numbers", "Den Hartog"]
from = "lacewing:dynamics.balancing"
license = "GPL-3.0-or-later"
inputs = ["V_0", "θ_0", "W_t", "φ_t", "V_1", "θ_1"]
outputs = ["A", "θ_A", "W_c", "ψ_c"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "§48 (balancing by influence numbers, Eq. 150a, here with one plane and one bearing)"

[[tests]]
name = "Hand check: a known unbalance recovered"
source = "By hand: influence number 0.15∠30° mil/g, unbalance 40 g at 250°, trial weight 10 g at 0°; readings V = αU: as found 6.0 mil at 280°, with the trial 5.665125 mil at 294.407098°. The correction is 40 g at 70°"
inputs = { V_0 = "6 mil", "θ_0" = "280°", W_t = "10 g", "φ_t" = "0°", V_1 = "5.665125 mil", "θ_1" = "294.407098°" }
expect = { A = "0.15 mil/g", "θ_A" = "30°", W_c = "40 g", "ψ_c" = "70°" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check: a trial weight that halves the vibration"
source = "By hand: as found 4 mm/s at 90°; 20 g at 90° brings it to 2 mm/s at 90°, so the influence number is (2 − 4)/20 = 0.1 mm/s per gram at 180° and the unbalance 40 g at 270°: add 40 g at 90°"
inputs = { V_0 = "4 mm/s", "θ_0" = "90°", W_t = "20 g", "φ_t" = "90°", V_1 = "2 mm/s", "θ_1" = "90°" }
expect = { A = "0.1 m/(s·kg)", "θ_A" = "180°", W_c = "40 g", "ψ_c" = "90°" }
tolerance = "1e-9"
checks = "pass"
+++

## Single-plane balancing with one trial weight

A rotor short along its axis compared with its diameter (a disk, fan, pulley, flywheel or grinding wheel) can be balanced in one plane. Run it at the balancing speed and read the vibration at the nearer bearing, amplitude and phase from a reference mark on the rotor; stop, fit a trial weight, and read again. The change in the vibration per unit trial weight is the influence number α, and the unbalance is the as-found reading over α (Den Hartog Eq. 150a, for one plane). For a longer rotor use the two-plane template.

The machine must respond linearly, both runs must be at the same speed, and weights and phases must be measured the same way round from the same mark. Make the trial weight large enough to change the reading clearly, typically by a quarter or more in amplitude or 30° in phase.

### Readings

Any one amplitude unit may be used (displacement, velocity or acceleration), and any one weight unit, or mass times radius; the correction comes out in the trial weight's unit, at its radius.

```calc {description="As found: amplitude"}
V_0 = 6 mil_
```

```calc {description="As found: phase from the reference mark"}
θ_0 = 280°_
```

```calc {description="Trial weight"}
W_t = 10 g_
```

```calc {description="Angle of the trial weight from the reference mark"}
φ_t = 0°_
```

```calc {description="With the trial weight: amplitude"}
V_1 = 5.665125 mil_
```

```calc {description="With the trial weight: phase"}
θ_1 = 294.407098°_
```

### Influence number and correction

The readings become complex numbers in units of the first reading V_0, and the weight in units of W_t. Complex numbers are written here as pairs [real, imaginary], with the small functions below for their product, quotient, size and angle (0 to 360°).

```calc
function polar(r, θ)
    return [r cos(θ), r sin(θ)]
end
```

```calc
function cmul(a, b)
    return [a[1] b[1] - a[2] b[2], a[1] b[2] + a[2] b[1]]
end
```

```calc
function cdiv(a, b)
    d = b[1]^2 + b[2]^2
    return [(a[1] b[1] + a[2] b[2]) / d, (a[2] b[1] - a[1] b[2]) / d]
end
```

```calc
function cabs(a)
    return hypot(a[1], a[2])
end
```

```calc
function carg(a)
    return mod(atan2(a[1], a[2]), 2 π)
end
```

```calc
> Readings as complex numbers (in units of V_0)
v_0 = polar(1, θ_0) = ??
v_1 = polar(V_1 / V_0, θ_1) = ??
> Trial weight as a complex number (in units of W_t)
w_t = polar(1, φ_t) = ??
> Influence number
α = cdiv(v_1 - v_0, w_t) = ??
> Unbalance (in units of W_t)
u = cdiv(v_0, α) = ??
```

The influence number holds for this machine at this speed, and can be reused to balance it again without a trial run.

```calc
> Size of the influence number: vibration per unit weight
A = cabs(α) V_0 / W_t = ??
> Its phase
θ_A = carg(α) = ?? °
```

The correction equals the unbalance and lies 180° opposite it. Remove the trial weight (or allow for it), fit the correction, and run again.

```calc
> Correction weight
W_c = cabs(u) W_t = ?? g
> Its angle from the reference mark
ψ_c = carg(-u) = ?? °
```
