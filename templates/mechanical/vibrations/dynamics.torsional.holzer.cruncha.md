+++
cruncha-template = 1
id = "dynamics.torsional.holzer"
version = "1.0.0"
title = "Torsional vibration of a chain of disks (Holzer)"
summary = "The torsional natural frequencies and mode shapes of disks on shafts, free at both ends (an engine, flywheel and propeller; a drive train), by Holzer's method: the inertia torque carried along the chain, its remainder after the last disk zero at a natural frequency, found by a scan in ω and bisection. Checks a forcing frequency's separation from every mode found."
category = ["Mechanical engineering", "Vibrations"]
tags = ["torsional vibration", "Holzer", "natural frequency", "mode shape", "drive train", "crankshaft", "flywheel", "propeller shaft", "Den Hartog", "resonance"]
from = "lacewing:dynamics.torsional"
license = "GPL-3.0-or-later"
inputs = ["I", "k", "ω_max", "ω_f", "s_min"]
outputs = ["ω", "N_m", "ω_1", "f_1", "ω_2", "β_1", "β_2", "s"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "§40 and §41 (Holzer's method; the six-cylinder Diesel ship drive of Fig. 143); Eq. 14 (two disks on a shaft); Eq. 144 (crank inertia)"

[[tests]]
name = "Den Hartog §41: six-cylinder Diesel ship drive, flywheel and propeller"
source = "Lacewing Lab vibration_test, after Den Hartog 4th §41 and Fig. 143: six cranks of 2,560 lb·in·s², flywheel 75,000, propeller 24,000; crank shafts 675·10⁶ and the propeller shaft 13.5·10⁶ lb·in/rad. The book's trials give ω² ≈ 711 and 17,930 (rad/s)², ω = 26.7 and 133.9 rad/s"
inputs = { I = "[2560; 2560; 2560; 2560; 2560; 2560; 75000; 24000] lbf·in·s²", k = "[675000; 675000; 675000; 675000; 675000; 675000; 13500] kip·in/rad", "ω_max" = "200 rad/s", "ω_f" = "60 rad/s" }
expect = { "ω_1" = "26.66 rad/s", "ω_2" = "133.9 rad/s" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Hand check: two disks on a shaft equal Eq. 14"
source = "By hand: Den Hartog's generator example (p. 62 of the 4th ed.), k = 10.2·10⁶ lb·in/rad, I = 150 and 600 lb·in·s²: Eq. 14 ω = √(k(I1 + I2)/(I1 I2)) = √(10.2e6 × 750/90000) = 291.5476 rad/s (the book rounds to 290). Running at 1.08 times that is within the 20 % separation, so the check fails"
inputs = { I = "[150; 600] lbf·in·s²", k = "[10200] kip·in/rad", "ω_max" = "400 rad/s", "ω_f" = "314.87 rad/s" }
expect = { "ω_1" = "291.5476 rad/s", N_m = "1", s = "0.08" }
tolerance = "1e-4"
checks = "fail"
+++

## Torsional vibration by Holzer's method

The disks of a shaft system (cranks, flywheel, gears, rotors, propeller) are rigid rotating inertias, and each length of shaft between two of them is a massless torsional spring. Undamped, and free at both ends of the chain (nothing ties it to ground), so the system also has a rigid-body mode at zero frequency, which is not listed. After Den Hartog, Mechanical Vibrations, §40 and §41.

Holzer's method: assume a frequency ω, give the first disk an amplitude of 1 rad, and walk along the chain. Each disk adds its inertia torque I ω² β to the torque in the shaft, and each shaft twists by that torque over its stiffness. At a natural frequency the torque left over after the last disk is zero. The template scans ω up to ω_max in 2000 steps for sign changes of that remainder and refines each by bisection. Each step is rounded to 15 significant figures (`roundSig`), which keeps the engine's exact fractions from growing without bound along a long chain.

### The chain

Give each disk's mass moment of inertia about the shaft axis, in order along the shaft, and the torsional stiffness of each shaft to the next disk (one fewer than the disks). For a crank, use the rotating parts plus half the reciprocating mass at the crank radius (Den Hartog Eq. 144); for a round shaft, k = G J / l with J = π d⁴/32, and shafts in series add as springs in series. Den Hartog's inch units, lb·in·s² and lb·in/rad, are `lbf·in·s²` and `lbf·in/rad`.

```calc {description="Mass moments of inertia of the disks, in order along the shaft"}
I = [0.5; 0.8; 2.0] kg·m²_
```

```calc {description="Torsional stiffness of each shaft to the next disk (one fewer than the disks)"}
k = [80000; 50000] N·m/rad_
```

```calc {description="Highest frequency to search up to"}
ω_max = 1000 rad/s_
```

```calc
function holzerRemainder(I, k, w)
    β = 1
    T = 0 I[1] w^2
    n = length(I)
    for i in 1:n
        T = roundSig(T + I[i] w^2 β, 15)
        if i < n
            β = roundSig(β - T / k[i], 15)
        end
    end
    return T
end
```

```calc
function holzerShape(I, k, w)
    β = 1
    T = 0 I[1] w^2
    v = [1]
    n = length(I)
    for i in 1:n
        T = roundSig(T + I[i] w^2 β, 15)
        if i < n
            β = roundSig(β - T / k[i], 15)
            v = vcat(v, β)
        end
    end
    return v
end
```

```calc
function holzerModes(I, k, wmax)
    steps = 2000
    out = [0]
    count = 0
    w_0 = wmax / steps / 1000
    r_0 = holzerRemainder(I, k, w_0)
    for i in 1:steps
        w_1 = wmax i / steps
        r_1 = holzerRemainder(I, k, w_1)
        if sign(r_0) ≠ sign(r_1)
            a = w_0
            b = w_1
            r_a = r_0
            for j in 1:60
                c = (a + b) / 2
                r_c = holzerRemainder(I, k, c)
                if sign(r_a) == sign(r_c)
                    a = c
                    r_a = r_c
                else
                    b = c
                end
            end
            count = count + 1
            if count == 1
                out = [(a + b) / (2 wmax)]
            else
                out = vcat(out, (a + b) / (2 wmax))
            end
        end
        w_0 = w_1
        r_0 = r_1
    end
    return out wmax
end
```

### Natural frequencies and mode shapes

```calc {description="Natural frequencies below ω_max, lowest first (rigid-body mode at zero left out)"}
ω = holzerModes(I, k, ω_max) = ?? rad/s
```

```calc
> Number of modes found below ω_max
N_m = length(ω) = ??
> First (lowest elastic) natural frequency
ω_1 = ω[1] = ?? rad/s
f_1 = ω_1 / (2 π) = ?? Hz
```

```calc {description="Second natural frequency"}
ω_2 = ω[2] = ?? rad/s
```

Mode n has n nodes along the shaft: a node is where the amplitude changes sign. A chain of two disks has only one mode, so ω_2 and β_2 are errors there; raise ω_max if fewer modes are found than you expect. The shapes are the disks' relative twist amplitudes, the first disk 1.

```calc {description="Mode shape of the first mode, first disk 1"}
β_1 = holzerShape(I, k, ω_1) = ??
```

```calc {description="Mode shape of the second mode, first disk 1"}
β_2 = holzerShape(I, k, ω_2) = ??
```

### Separation from the forcing frequency

A running speed, or one of its harmonics (the firing orders of an engine), near a natural frequency excites that mode. The check asks every mode found to be at least s_min away from the forcing frequency, as a fraction of the mode's frequency.

```calc {description="Forcing frequency: a running speed or one of its harmonic orders"}
ω_f = 100 rad/s_
```

```calc {description="Smallest acceptable separation, as a fraction of the natural frequency"}
s_min = 0.2_
```

```calc {description="Separation of the forcing frequency from the nearest mode found"}
s = min(abs(ω_f / ω - 1)) = ??
```

```calc
@test s ≥ s_min
```
