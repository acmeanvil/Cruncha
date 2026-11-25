+++
cruncha-template = 1
id = "mechanical.shaftCriticalSpeed.simplySupported"
version = "1.0.0"
title = "Shaft critical speed: uniform simply supported shaft with lumped masses (Rayleigh, Dunkerley)"
summary = "The first critical speed of a uniform shaft on two bearings carrying any number of lumped weights: influence coefficients of the simply supported beam, Rayleigh's estimate (high), Dunkerley's (low), the exact value for the lumped masses on a massless shaft, the shaft's own critical speed and Dunkerley with it, against twice the operating speed."
category = ["Mechanical engineering", "Shafts"]
tags = ["shaft", "critical speed", "whirling", "natural frequency", "Rayleigh", "Dunkerley", "influence coefficients", "lumped mass", "rotordynamics", "Shigley", "Juvinall"]
from = "lacewing:mechanical.shaftCriticalSpeed"
license = "GPL-3.0-or-later"
inputs = ["W", "a", "l", "d", "E", "ρ", "n_op"]
outputs = ["I", "EI", "ω_R", "ω_D", "ω_1", "ω_11", "ω_s", "ω_(D,s)", "n_cr", "r_cr"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-6: Eqs. 7-22 (uniform shaft), 7-23 (Rayleigh), 7-24 (influence coefficients), 7-25, 7-26 (frequency determinant), 7-32 (Dunkerley), 7-34 (Dunkerley with the shaft); Example 7-5"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.4: Eqs. 17.1, 17.2 (Rayleigh), 17.4 (Dunkerley)"

[[tests]]
name = "Shigley Ex. 7-5: 1 in steel shaft, 31 in span, 35 lbf at 7 in and 55 lbf at 20 in"
source = "Lacewing Lab critical_speed_test, after Shigley 10th Example 7-5: Rayleigh 124.8 rad/s (1192 rev/min), ω11 231.4 rad/s, Dunkerley 120.3 rad/s, the shaft alone 520.4 rad/s (γ = 0.282 lbf/in³), Dunkerley with the shaft 117.3 rad/s. The exact lumped value 124.677 rad/s (between the two) is the largest eigenvalue of M^½DM^½ computed by hand (NumPy). At 600 rev/min the lowest estimate is 1.87 times the operating speed, under Shigley's 2"
inputs = { W = "[35; 55] lbf", a = "[7; 20] in", l = "31 in", d = "1 in", E = "30 Mpsi", "ρ" = "0.282 lb/in³", n_op = "600 rpm" }
expect = { "ω_R" = "124.8 rad/s", "ω_11" = "231.4 rad/s", "ω_D" = "120.3 rad/s", "ω_s" = "520.4 rad/s", "ω_(D,s)" = "117.3 rad/s", "ω_1" = "124.677 rad/s", n_cr = "1120 rpm" }
tolerance = "0.1%"
checks = "fail"

[[tests]]
name = "Hand check: three masses on a 60 mm steel shaft, 1.2 m span"
source = "By hand (NumPy), Eqs. 7-22 to 7-34: 200, 100, 150 kg at 0.3, 0.6, 0.9 m, E 207 GPa, ρ 7850 kg/m³: Rayleigh 116.03094, Dunkerley 111.00296, exact 116.01352, ω11 180.32050, shaft alone 527.93306, Dunkerley with the shaft 108.62776 rad/s; at 400 rev/min (41.888 rad/s) the ratio is 2.593297"
inputs = { W = "[1961.33; 980.665; 1470.9975] N", a = "[0.3; 0.6; 0.9] m", l = "1.2 m", d = "60 mm", E = "207 GPa", "ρ" = "7850 kg/m³", n_op = "400 rpm" }
expect = { "ω_R" = "116.03094 rad/s", "ω_D" = "111.00296 rad/s", "ω_1" = "116.01352 rad/s", "ω_11" = "180.32050 rad/s", "ω_s" = "527.93306 rad/s", "ω_(D,s)" = "108.62776 rad/s", r_cr = "2.593297" }
tolerance = "1e-6"
checks = "pass"
+++

## Shaft critical speed: simply supported shaft with lumped masses

The first (lowest) critical speed of a uniform shaft on two bearings that carries gears, pulleys or rotors, treated as lumped weights W_i at distances a_i from the left bearing, after Shigley Sec. 7-6. At a critical speed the shaft whirls and its deflection grows without bound; Shigley asks for the first critical speed to be at least twice the operating speed.

The influence coefficient δ_ij is the deflection at mass i from a unit load at mass j (Eq. 7-24, from the simply supported beam). Rayleigh's method takes the static deflection curve as the mode shape and overestimates; Dunkerley's adds the masses one at a time and underestimates, and can take the shaft's own mass as one more term (Eq. 7-34). The exact first critical speed of the lumped masses on a massless shaft lies between the two: it is the root of the frequency determinant (Eq. 7-26), found here by power iteration on the symmetric matrix M^½ D M^½. The shaft's own mass lowers it further; Dunkerley with the shaft is the conservative figure to check.

Stepped shafts, overhangs, other supports and higher modes need a fuller analysis (for example the influence-coefficient template with coefficients from a beam analysis). Give the weights as forces: a mass m is W = m g.

### Shaft

```calc {description="Span between the bearings"}
l = 31 in_
```

```calc {description="Shaft diameter (uniform)"}
d = 1 in_
```

```calc {description="Young's modulus"}
E = 30 Mpsi_
```

```calc {description="Density of the shaft material, for its own mass"}
ρ = 0.282 lb/in³_
```

```calc {description="Standard gravity"}
g = 9.80665 m/s²
```

```calc
> Second moment of area
I = π d^4 / 64 = ?? in^4
> Bending stiffness
EI = E I = ?? lbf·in²
```

### Masses

```calc {description="Weights of the lumped masses (a column, one per mass)"}
W = [35; 55] lbf_
```

```calc {description="Distance of each mass from the left bearing (a column, in the order of W)"}
a = [7; 20] in_
```

```calc {description="Operating speed"}
n_op = 600 rpm_
```

### Influence coefficients and estimates

```calc
function influence(x, s, l, EI)
    if x ≤ s
        b = l - s
        return b x (l^2 - b^2 - x^2) / (6 EI l)
    end
    return s (l - x) (2 l x - s^2 - x^2) / (6 EI l)
end
```

```calc
function rayleigh(W, a, l, EI, g)
    n = length(W)
    p = 0 'N·m'
    q = 0 'N·m^2'
    for i in 1:n
        y = 0 'm'
        for j in 1:n
            y = y + W[j] influence(a[i], a[j], l, EI)
        end
        p = p + W[i] y
        q = q + W[i] y^2
    end
    return sqrt(g p / q)
end
```

```calc
function dunkerleySum(W, a, l, EI, g)
    s = 0 's^2'
    for i in 1:length(W)
        s = s + W[i] / g influence(a[i], a[i], l, EI)
    end
    return s
end
```

```calc
function exactSpeed(W, a, l, EI, g)
    n = length(W)
    v = ones(n, 1) / sqrt(n)
    λ = 0
    λ_0 = -1
    k = 0
    while abs(λ - λ_0) > 1e-13 abs(λ) and k < 200
        λ_0 = λ
        for i in 1:n
            t = 0
            for j in 1:n
                t = t + sqrt(W[i] W[j]) / g influence(a[i], a[j], l, EI) / 1 's^2' v[j]
            end
            if i == 1
                w = [t]
            else
                w = vcat(w, t)
            end
        end
        p = 0
        q = 0
        for i in 1:n
            p = p + w[i] v[i]
            q = q + w[i]^2
        end
        λ = p
        v = w / sqrt(q)
        k = k + 1
    end
    return 1 / sqrt(λ 1 's^2')
end
```

```calc
> Rayleigh's estimate from the static deflections, Eqs. 7-23, 7-25 (high)
ω_R = rayleigh(W, a, l, EI, g) = ?? rad/s
> Dunkerley's estimate, Eq. 7-32 (low)
ω_D = 1 / sqrt(dunkerleySum(W, a, l, EI, g)) = ?? rad/s
> Exact first critical speed of the lumped masses on a massless shaft, Eq. 7-26
ω_1 = exactSpeed(W, a, l, EI, g) = ?? rad/s
> The first mass alone on the shaft, 1/√(m₁δ₁₁)
ω_11 = sqrt(g / (W[1] influence(a[1], a[1], l, EI))) = ?? rad/s
```

### The shaft's own mass

```calc
> First critical speed of the uniform shaft alone, Eq. 7-22
ω_s = (π / l)^2 sqrt(EI / (ρ π d^2 / 4)) = ?? rad/s
> Dunkerley with the shaft, Eq. 7-34
ω_(D,s) = 1 / sqrt(1 / ω_s^2 + dunkerleySum(W, a, l, EI, g)) = ?? rad/s
> The same in revolutions per minute
n_cr = ω_(D,s) = ?? rpm
> Ratio of the lowest estimate to the operating speed
r_cr = ω_(D,s) / n_op = ??
```

### Check

Shigley (Sec. 7-6) asks for a first critical speed at least twice the operating speed.

```calc
@test r_cr ≥ 2
```
