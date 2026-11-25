+++
cruncha-template = 1
id = "dynamics.absorber.response"
version = "1.0.0"
title = "Damped vibration absorber, response for a given tuning and damping"
summary = "A tuned mass damper of your own tuning f = ωa/Ωn and damping c/cc on an undamped main mass and spring: the absorber's spring and damper, the main mass's amplitude at the forcing frequency by Den Hartog's Eq. 57, and its resonant amplitude, the largest on the response curve. f = 1 is an absorber tuned to the main system, f = 0 a Lanchester damper. Checks both amplitudes."
category = ["Mechanical engineering", "Vibrations"]
tags = ["vibration absorber", "tuned mass damper", "dynamic vibration absorber", "TMD", "tuning", "damping", "Lanchester damper", "frequency response", "mass ratio", "resonance", "Den Hartog"]
from = "lacewing:dynamics.absorber"
license = "GPL-3.0-or-later"
inputs = ["M", "K", "m", "P_0", "f", "ζ_a", "ω", "x_allow"]
outputs = ["μ", "Ω_n", "x_st", "k_a", "c", "g", "β", "x_1", "β_peak", "x_peak"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Ch. III, §23 (undamped absorber, Eq. 49) and §24 (damped absorber, Eq. 57; the Lanchester damper, Eqs. 66, 67; Fig. 80)"

[[tests]]
name = "Lacewing Lab: Eq. 57 at zero damping is the undamped Eq. 49"
source = "Lacewing Lab vibration_test (Den Hartog 4th Eqs. 49 and 57): μ = 0.2, f = 1, no damping, g = 0.5: |1 − g²| / |(1 − g²)(1 + μ − g²) − μ| = 0.75/0.5125 = 1.4634146. Undamped, the two resonances are unbounded, so the resonant amplitude fails any limit"
inputs = { M = "100 kg", K = "1e4 N/m", m = "20 kg", P_0 = "100 N", f = "1", "ζ_a" = "0", "ω" = "5 rad/s", x_allow = "1 m" }
expect = { "μ" = "0.2", g = "0.5", "β" = "1.4634146", x_1 = "14.634146 mm" }
tolerance = "1e-7"
checks = "fail"

[[tests]]
name = "Lacewing Lab: a rigid damper makes one mass M + m"
source = "Lacewing Lab vibration_test (Den Hartog 4th Eq. 57): with c/cc = 10⁹ the absorber locks to the main mass and x1/xst = 1/|1 − g²(1 + μ)|; μ = 0.2, g = 0.9: 1/|1 − 0.81 × 1.2| = 35.714286. The locked system resonates undamped at g = 1/√1.2, so the resonant amplitude fails"
inputs = { M = "100 kg", K = "1e4 N/m", m = "20 kg", P_0 = "100 N", f = "1", "ζ_a" = "1e9", "ω" = "9 rad/s", x_allow = "1 m" }
expect = { "β" = "35.714286" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Den Hartog Fig. 80: Lanchester damper of a quarter of the main mass"
source = "Lacewing Lab vibration_test, after Den Hartog 4th Eqs. 67 and 69b: f = 0, μ = 0.25 and the optimum c/cc = √(1/(2(2 + μ)(1 + μ))) = 0.42163702 leave a resonant magnification of 1 + 2/μ = 9"
inputs = { M = "200 kg", K = "2e6 N/m", m = "50 kg", P_0 = "500 N", f = "0", "ζ_a" = "0.42163702", "ω" = "100 rad/s", x_allow = "2.5 mm" }
expect = { "β_peak" = "9", x_peak = "2.25 mm", k_a = "0 N/m" }
tolerance = "0.1%"
checks = "pass"

[[tests]]
name = "Hand check: the Lacewing Lab page's entered design, f = 0.8 and c/cc = 0.1"
source = "By hand from Eq. 57 (a Python evaluation of the same formula): M = 200 kg, K = 2 MN/m, m = 50 kg, Ωn = 100 rad/s, k = 50 × 80² = 320 kN/m, c = 0.1 × 2 × 50 × 100 = 1000 N·s/m; at ω = Ωn, x1/xst = 2.4567432; largest over g in 0.3 to 2 is 4.9137030 (under-damped: the two peaks rise above the optimum's 3), x_peak = 1.2284257 mm, over 1 mm"
inputs = { M = "200 kg", K = "2e6 N/m", m = "50 kg", P_0 = "500 N", f = "0.8", "ζ_a" = "0.1", "ω" = "100 rad/s", x_allow = "1 mm" }
expect = { k_a = "320 kN/m", c = "1000 N·s/m", "β" = "2.4567432", x_1 = "0.6141858 mm", "β_peak" = "4.9137030", x_peak = "1.2284257 mm" }
tolerance = "1e-6"
checks = "fail"
+++

## Damped vibration absorber, response for a given tuning and damping

A small mass m on a spring k_a and a viscous damper c, attached to a main mass M on its spring K. The main system is undamped and driven by a harmonic force P_0 on M. The absorber's tuning f is its own natural frequency √(k_a/m) over the main system's Ω_n, and its damping is c over c_c = 2 m Ω_n. After Den Hartog, Mechanical Vibrations, ch. III. For Den Hartog's most favourable tuning and damping, use the most favourable design template.

### Main system and absorber

```calc {description="Main mass, whose vibration is to be reduced"}
M = 200 kg_
```

```calc {description="Main spring stiffness"}
K = 2e6 N/m_
```

```calc {description="Absorber mass"}
m = 50 kg_
```

```calc {description="Amplitude of the harmonic force on the main mass"}
P_0 = 500 N_
```

```calc {description="Absorber tuning ω_a / Ω_n: 1 tuned to the main system, 0 a Lanchester damper (no spring)"}
f = 0.8_
```

```calc {description="Absorber damping c / c_c, with c_c = 2 m Ω_n"}
ζ_a = 0.1_
```

```calc
> Mass ratio
μ = m / M = ??
> Natural frequency of the main system alone
Ω_n = sqrt(K / M) = ?? rad/s
> Static deflection of the main mass under P_0
x_st = P_0 / K = ?? mm
> Absorber spring
k_a = m (f Ω_n)^2 = ?? kN/m
> Absorber damper
c = ζ_a 2 m Ω_n = ?? N·s/m
```

### Response

Eq. 57 gives the main mass's amplitude over its static deflection for a tuning f, damping z and forcing ratio g = ω/Ω_n.

```calc
function absorberRatio(μ, f, z, g)
    a = (2 z g)^2
    n = a + (g^2 - f^2)^2
    d = a (g^2 - 1 + μ g^2)^2 + (μ f^2 g^2 - (g^2 - 1) (g^2 - f^2))^2
    return sqrt(n / d)
end
```

```calc {description="Forcing (circular) frequency"}
ω = 100 rad/s_
```

```calc
> Forcing ratio
g = ω / Ω_n = ??
> Magnification of the main mass at ω, Eq. 57
β = absorberRatio(μ, f, ζ_a, g) = ??
> Amplitude of the main mass at ω
x_1 = β x_st = ?? mm
```

The resonant amplitude is the largest value of Eq. 57 over g from 0.3 to 2, in 6000 steps, so a machine that runs up through its speed range meets it. With little or no damping the peaks are sharp and the scan understates them; with none they are infinite.

```calc
function absorberPeak(μ, f, z)
    best = 0
    for i in 0:6000
        best = max(best, absorberRatio(μ, f, z, 0.3 + 1.7 i / 6000))
    end
    return best
end
```

```calc
> Resonant magnification of the main mass
β_peak = absorberPeak(μ, f, ζ_a) = ??
> Resonant amplitude of the main mass
x_peak = β_peak x_st = ?? mm
```

### Checks

```calc {description="Largest acceptable amplitude of the main mass"}
x_allow = 1 mm_
```

```calc
@test x_1 ≤ x_allow
```

```calc
@test x_peak ≤ x_allow
```
