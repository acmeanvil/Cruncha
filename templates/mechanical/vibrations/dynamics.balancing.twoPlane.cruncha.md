+++
cruncha-template = 1
id = "dynamics.balancing.twoPlane"
version = "1.0.0"
title = "Two-plane balancing by influence coefficients"
summary = "The correction weights for a rigid rotor in two planes from three runs at the balancing speed (as found, a trial weight in plane I, a trial weight in plane II), each read as amplitude and phase at two bearings: the four complex influence numbers, the unbalance in each plane and the weight to add, with its angle."
category = ["Mechanical engineering", "Vibrations"]
tags = ["balancing", "two-plane balancing", "dynamic balancing", "field balancing", "influence coefficients", "trial weight", "correction weight", "unbalance", "rotor", "complex numbers", "Den Hartog"]
from = "lacewing:dynamics.balancing"
license = "GPL-3.0-or-later"
inputs = ["V_a0", "θ_a0", "V_b0", "θ_b0", "W_1", "φ_1", "V_a1", "θ_a1", "V_b1", "θ_b1", "W_2", "φ_2", "V_a2", "θ_a2", "V_b2", "θ_b2"]
outputs = ["W_I", "ψ_I", "W_II", "ψ_II", "A_aI", "A_bI", "A_aII", "A_bII"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "§48 (balancing in two planes: influence numbers, Eqs. 150a and 150b) and §49 (flexible rotors)"

[[tests]]
name = "Lacewing Lab page: readings made from 3 at 70° in plane I and 5 at 200° in plane II"
source = "Lacewing Lab vibration_screens_test (Den Hartog 4th §48, Eqs. 150a, 150b): readings va0 = 6.225∠108.33°, vb0 = 8.994∠195.44°; trial 10∠0° in I: va1 = 20.066∠27.87°, vb1 = 4.909∠170.38°; trial 10∠0° in II: va2 = 5.331∠46.34°, vb2 = 9.298∠354.91°. Corrections 3 at 250° (within 0.2°) and 5 at 20°, within 0.01 from the readings' rounding"
inputs = { V_a0 = "6.225 μm", "θ_a0" = "108.33°", V_b0 = "8.994 μm", "θ_b0" = "195.44°", W_1 = "10 g", "φ_1" = "0°", V_a1 = "20.066 μm", "θ_a1" = "27.87°", V_b1 = "4.909 μm", "θ_b1" = "170.38°", W_2 = "10 g", "φ_2" = "0°", V_a2 = "5.331 μm", "θ_a2" = "46.34°", V_b2 = "9.298 μm", "θ_b2" = "354.91°" }
expect = { W_I = "3 g", "ψ_I" = "250°", W_II = "5 g", "ψ_II" = "20°" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Lacewing Lab: a known unbalance recovered from three runs"
source = "Lacewing Lab vibration_test (Den Hartog 4th Eq. 150a): influence numbers αaI = 2∠10°, αbI = 0.5∠40°, αaII = 0.6∠−20°, αbII = 1.8∠5° per gram, unbalances 3∠70° and 5∠200° g, trial weights 1∠0° and 1∠90° g; the readings, V = αU, by hand (to six decimals): va0 = 6.224843∠108.334490°, vb0 = 8.994265∠195.436632°, va1 = 6.256153∠89.894638°, vb1 = 8.542043∠194.042341°, va2 = 6.705819∠105.153130°, vb2 = 8.847109∠183.894393°. Corrections 3 g at 250°, 5 g at 20°"
inputs = { V_a0 = "6.224843 μm", "θ_a0" = "108.334490°", V_b0 = "8.994265 μm", "θ_b0" = "195.436632°", W_1 = "1 g", "φ_1" = "0°", V_a1 = "6.256153 μm", "θ_a1" = "89.894638°", V_b1 = "8.542043 μm", "θ_b1" = "194.042341°", W_2 = "1 g", "φ_2" = "90°", V_a2 = "6.705819 μm", "θ_a2" = "105.153130°", V_b2 = "8.847109 μm", "θ_b2" = "183.894393°" }
expect = { W_I = "3 g", "ψ_I" = "250°", W_II = "5 g", "ψ_II" = "20°", A_aI = "2 μm/g", A_bI = "0.5 μm/g", A_aII = "0.6 μm/g", A_bII = "1.8 μm/g" }
tolerance = "1e-5"
checks = "pass"
+++

## Two-plane balancing by influence coefficients

A rigid rotor's unbalance can always be corrected by one weight in each of two planes. With the machine running at the balancing speed, read the vibration at both bearings, a and b, as an amplitude and a phase from one reference mark on the rotor; then repeat with a trial weight in plane I only, and again with a trial weight in plane II only (the first removed). The vibration at each bearing is the sum of each plane's unbalance times an influence number (Den Hartog Eq. 150a), so the three runs give the four influence numbers and the two unbalances (Eq. 150b), in complex arithmetic. After Den Hartog, Mechanical Vibrations, §48.

The machine must respond linearly, every run must be at the same speed, and weights and phases must be measured the same way round from the same mark. A rotor running near a flexible (bending) critical speed is not rigid: balance it at its running speed (Den Hartog §49).

### As found

Any one amplitude unit may be used for all the readings (displacement, velocity or acceleration), and any one weight unit, or mass times radius, for the trial weights; the corrections come out in that unit, at the trial weights' radius.

```calc {description="Bearing a, as found: amplitude"}
V_a0 = 6.225 μm_
```

```calc {description="Bearing a, as found: phase from the reference mark"}
θ_a0 = 108.33°_
```

```calc {description="Bearing b, as found: amplitude"}
V_b0 = 8.994 μm_
```

```calc {description="Bearing b, as found: phase"}
θ_b0 = 195.44°_
```

### Trial weight in plane I

```calc {description="Trial weight in plane I"}
W_1 = 10 g_
```

```calc {description="Angle of the trial weight in plane I from the reference mark"}
φ_1 = 0°_
```

```calc {description="Bearing a with the trial weight in plane I: amplitude"}
V_a1 = 20.066 μm_
```

```calc {description="Bearing a with the trial weight in plane I: phase"}
θ_a1 = 27.87°_
```

```calc {description="Bearing b with the trial weight in plane I: amplitude"}
V_b1 = 4.909 μm_
```

```calc {description="Bearing b with the trial weight in plane I: phase"}
θ_b1 = 170.38°_
```

### Trial weight in plane II

```calc {description="Trial weight in plane II (the plane I weight removed)"}
W_2 = 10 g_
```

```calc {description="Angle of the trial weight in plane II"}
φ_2 = 0°_
```

```calc {description="Bearing a with the trial weight in plane II: amplitude"}
V_a2 = 5.331 μm_
```

```calc {description="Bearing a with the trial weight in plane II: phase"}
θ_a2 = 46.34°_
```

```calc {description="Bearing b with the trial weight in plane II: amplitude"}
V_b2 = 9.298 μm_
```

```calc {description="Bearing b with the trial weight in plane II: phase"}
θ_b2 = 354.91°_
```

### Influence numbers and unbalances

The readings become complex numbers in units of the first reading V_a0, and the weights in units of W_1. Complex numbers are written here as pairs [real, imaginary], with the small functions below for their product, quotient, size and angle (0 to 360°).

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
> Readings as complex numbers (in units of V_a0)
v_a0 = polar(1, θ_a0) = ??
v_b0 = polar(V_b0 / V_a0, θ_b0) = ??
v_a1 = polar(V_a1 / V_a0, θ_a1) = ??
v_b1 = polar(V_b1 / V_a0, θ_b1) = ??
v_a2 = polar(V_a2 / V_a0, θ_a2) = ??
v_b2 = polar(V_b2 / V_a0, θ_b2) = ??
> Trial weights as complex numbers (in units of W_1)
w_1 = polar(1, φ_1) = ??
w_2 = polar(W_2 / W_1, φ_2) = ??
```

```calc
> Influence numbers, Eq. 150a: change at each bearing per unit trial weight
α_aI = cdiv(v_a1 - v_a0, w_1) = ??
α_bI = cdiv(v_b1 - v_b0, w_1) = ??
α_aII = cdiv(v_a2 - v_a0, w_2) = ??
α_bII = cdiv(v_b2 - v_b0, w_2) = ??
> Determinant of the influence numbers
Δ_α = cmul(α_aI, α_bII) - cmul(α_aII, α_bI) = ??
> Unbalances in planes I and II, Eq. 150b (in units of W_1)
u_I = cdiv(cmul(α_bII, v_a0) - cmul(α_aII, v_b0), Δ_α) = ??
u_II = cdiv(cmul(α_aI, v_b0) - cmul(α_bI, v_a0), Δ_α) = ??
```

The influence numbers hold for this machine at this speed, and can be reused to balance it again without trial runs.

```calc
> Size of the influence numbers: vibration per unit weight
A_aI = cabs(α_aI) V_a0 / W_1 = ??
A_bI = cabs(α_bI) V_a0 / W_1 = ??
A_aII = cabs(α_aII) V_a0 / W_1 = ??
A_bII = cabs(α_bII) V_a0 / W_1 = ??
```

### Corrections

Each correction equals the unbalance found in its plane and lies 180° opposite it. Fit both, run, and repeat with the new readings if the vibration is not yet low enough.

```calc
> Correction weight in plane I
W_I = cabs(u_I) W_1 = ?? g
> Its angle from the reference mark
ψ_I = carg(-u_I) = ?? °
> Correction weight in plane II
W_II = cabs(u_II) W_1 = ?? g
> Its angle
ψ_II = carg(-u_II) = ?? °
```
