+++
cruncha-template = 1
id = "mechanical.shaftCriticalSpeed.influence"
version = "1.0.0"
title = "Shaft critical speed from influence coefficients (Rayleigh, Dunkerley, exact lumped value)"
summary = "The first critical speed of any shaft carrying lumped weights, from the matrix of influence coefficients found by your own deflection analysis: the static deflections, Rayleigh's estimate (high), each mass alone, Dunkerley's estimate (low) and the exact value for the lumped masses on a massless shaft, against twice the operating speed."
category = ["Mechanical engineering", "Shafts"]
tags = ["shaft", "critical speed", "whirling", "natural frequency", "Rayleigh", "Dunkerley", "influence coefficients", "lumped mass", "frequency determinant", "Shigley", "Juvinall"]
from = "lacewing:mechanical.shaftCriticalSpeed"
license = "GPL-3.0-or-later"
inputs = ["W", "δ", "n_op"]
outputs = ["y", "y_max", "ω_R", "ω_D", "ω_1", "n_1", "r_cr"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 7-6: Eqs. 7-23 (Rayleigh), 7-25 (deflections), 7-26 (frequency determinant), 7-32 (Dunkerley); Example 7-5"

[[references]]
work = "Juvinall, Marshek: Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 17.4: Eqs. 17.1, 17.2 (Rayleigh), 17.4 (Dunkerley); Sample Problem 17.1"

[[tests]]
name = "Juvinall Sample Problem 17.1: 70 lb and 30 lb, influence coefficients given"
source = "Lacewing Lab critical_speed_test, after Juvinall 5th Sample Problem 17.1: a11 3.4, a22 20.4, a12 = a21 6.8 (10⁻⁶ in/lb); deflections 4.42·10⁻⁴ and 1.088·10⁻³ in, Rayleigh 706.3 rad/s, Dunkerley 673.9 rad/s (the book takes g = 386 in/s²). The exact two-mass value 699.786 rad/s from the 2 × 2 frequency determinant by hand"
inputs = { W = "[70; 30] lbf", "δ" = "[0.0000034, 0.0000068; 0.0000068, 0.0000204] in/lbf", n_op = "3000 rpm" }
expect = { y_max = "0.001088 in", "ω_R" = "706.3 rad/s", "ω_D" = "673.9 rad/s", "ω_1" = "699.786 rad/s" }
tolerance = "0.05%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 7-5 by its printed influence coefficients"
source = "Shigley 10th Example 7-5: δ11 = 2.061, δ22 = 3.534, δ12 = δ21 = 2.224 (10⁻⁴ in/lbf), 35 and 55 lbf: y 0.01945 and 0.02722 in, Rayleigh 124.8, Dunkerley 120.3 rad/s; exact 124.674 rad/s by hand (NumPy). At 1200 rev/min the first critical speed is under twice the operating speed"
inputs = { W = "[35; 55] lbf", "δ" = "[0.0002061, 0.0002224; 0.0002224, 0.0003534] in/lbf", n_op = "1200 rpm" }
expect = { y_max = "0.02722 in", "ω_R" = "124.8 rad/s", "ω_D" = "120.3 rad/s", "ω_1" = "124.674 rad/s" }
tolerance = "0.1%"
checks = "fail"
+++

## Shaft critical speed from influence coefficients

The first critical speed of a shaft carrying lumped weights W_i, from the influence coefficients δ_ij (the deflection at mass i from a unit load at mass j) found by a deflection analysis of the actual shaft: stepped, overhung or on any supports, after Shigley Sec. 7-6 and Juvinall Sec. 17.4. The matrix is symmetric (Maxwell's reciprocity), its rows and columns in the order of W.

Rayleigh's method uses the static deflections as the mode shape and overestimates the critical speed; Dunkerley's adds the masses one at a time and underestimates it. The exact value for the lumped masses on a massless shaft lies between: the root of the frequency determinant (Eq. 7-26), found by power iteration on the symmetric matrix M^½ δ M^½. The shaft's own mass lowers the critical speed; include it as more lumped masses, or use the simply supported template for a uniform shaft. Shigley asks for a first critical speed at least twice the operating speed. Give the weights as forces: a mass m is W = m g.

```calc {description="Weights of the lumped masses (a column, one per mass)"}
W = [70; 30] lbf_
```

```calc {description="Influence coefficients δ_ij, deflection at i per unit load at j (symmetric, in the order of W)"}
δ = [0.0000034, 0.0000068; 0.0000068, 0.0000204] in/lbf_
```

```calc {description="Operating speed"}
n_op = 3000 rpm_
```

```calc {description="Standard gravity"}
g = 9.80665 m/s²
```

```calc
> Static deflections under the weights, Eq. 7-25
y = δ W = ?? in
> Largest static deflection
y_max = max(y) = ?? in
```

```calc
function rayleigh(W, y, g)
    p = 0 'N·m'
    q = 0 'N·m^2'
    for i in 1:length(W)
        p = p + W[i] y[i]
        q = q + W[i] y[i]^2
    end
    return sqrt(g p / q)
end
```

```calc
function dunkerley(W, D, g)
    s = 0 's^2'
    for i in 1:length(W)
        s = s + W[i] / g D[i, i]
    end
    return 1 / sqrt(s)
end
```

```calc
function exactSpeed(W, D, g)
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
                t = t + sqrt(W[i] W[j]) / g D[i, j] / 1 's^2' v[j]
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
> Rayleigh's estimate, Eq. 7-23 (high)
ω_R = rayleigh(W, y, g) = ?? rad/s
> Dunkerley's estimate, Eq. 7-32 (low)
ω_D = dunkerley(W, δ, g) = ?? rad/s
> Exact first critical speed of the lumped masses, Eq. 7-26
ω_1 = exactSpeed(W, δ, g) = ?? rad/s
> The same in revolutions per minute
n_1 = ω_1 = ?? rpm
> Ratio of Dunkerley's (low) estimate to the operating speed
r_cr = ω_D / n_op = ??
```

### Check

Shigley (Sec. 7-6) asks for a first critical speed at least twice the operating speed; Dunkerley's estimate is the conservative one to check.

```calc
@test r_cr ≥ 2
```
