+++
cruncha-template = 1
id = "dynamics.absorber.optimum"
version = "1.0.0"
title = "Damped vibration absorber, most favourable design (Den Hartog)"
summary = "A tuned mass damper on an undamped main mass and spring driven by a harmonic force: Den Hartog's most favourable tuning and damping for the mass ratio, the absorber's spring and damper, and the resonant amplitude it leaves, confirmed on the full response curve; for comparison the undamped tuned absorber's two natural frequencies, the optimum damping when tuned to the main system, and a Lanchester damper of the same mass. Checks the main mass's resonant amplitude."
category = ["Mechanical engineering", "Vibrations"]
tags = ["vibration absorber", "tuned mass damper", "dynamic vibration absorber", "TMD", "optimum tuning", "optimum damping", "Lanchester damper", "fixed points", "mass ratio", "resonance", "Den Hartog"]
from = "lacewing:dynamics.absorber"
license = "GPL-3.0-or-later"
inputs = ["M", "K", "m", "P_0", "x_allow"]
outputs = ["μ", "Ω_n", "x_st", "f_opt", "k_a", "ζ_opt", "c_opt", "β_opt", "β_curve", "x_peak", "g_1", "g_2", "ζ_tuned", "β_L", "ζ_L", "c_L"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Ch. III, §23 (undamped absorber, Eq. 50) and §24 (damped absorber: Eq. 57, the fixed points, Eqs. 63 and 64 for the most favourable tuning, 69 for its damping, 69a tuned to the main system, 66, 67 and 69b for the Lanchester damper; Fig. 80)"

[[tests]]
name = "Den Hartog Fig. 80: absorber a quarter of the main mass"
source = "Lacewing Lab vibration_test, after Den Hartog 4th §24 and Fig. 80: μ = 1/4 gives f = 1/(1 + μ) = 0.8 and fixed points at √(1 + 2/μ) = 3 times xst; Eq. 57 at the optimum damping peaks at 3 to within 2 % (the curve's maximum lies a little above the fixed points); a Lanchester damper of the same mass leaves 1 + 2/μ = 9. M = 200 kg, K = 2 MN/m, m = 50 kg, 500 N (the Lacewing page's opening case): xst = 0.25 mm"
inputs = { M = "200 kg", K = "2e6 N/m", m = "50 kg", P_0 = "500 N", x_allow = "1 mm" }
expect = { "μ" = "0.25", f_opt = "0.8", "β_opt" = "3", "β_curve" = "3", "β_L" = "9", x_st = "0.25 mm", "Ω_n" = "100 rad/s" }
tolerance = "2%"
checks = "pass"

[[tests]]
name = "Den Hartog Eq. 50: μ = 0.2, undamped and tuned"
source = "Lacewing Lab vibration_test, after Den Hartog 4th §23, Eq. 50: an undamped absorber of μ = 0.2 tuned to the main system splits the resonance into 0.80 and 1.25 times the tuned frequency, whose product is 1"
inputs = { M = "100 kg", K = "1e6 N/m", m = "20 kg", P_0 = "100 N", x_allow = "1 mm" }
expect = { "μ" = "0.2", g_1 = "0.80", g_2 = "1.25" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Hand check: μ = 1/4 exactly, and too little absorber mass for 0.5 mm"
source = "By hand: Ωn = √(2e6/200) = 100 rad/s; f = 0.8, absorber spring 50 × 80² = 320 kN/m; (c/cc)² = 3 × 0.25/(8 × 1.25³) = 0.048, c/cc = 0.21908902, c = 0.21908902 × 2 × 50 × 100 = 2190.8902 N·s/m; x_peak = 3 × 0.25 = 0.75 mm, over 0.5 mm; Eq. 50 roots √(1.125 ∓ √0.265625) = 0.78077641, 1.2807764; tuned to f = 1, c/cc = √(0.25(3.25 + √(0.25/2.25))/(8 × 1.25)) = 0.29930475; Lanchester c/cc = √(1/(2 × 2.25 × 1.25)) = 0.42163702, c = 4216.3702 N·s/m"
inputs = { M = "200 kg", K = "2e6 N/m", m = "50 kg", P_0 = "500 N", x_allow = "0.5 mm" }
expect = { k_a = "320 kN/m", "ζ_opt" = "0.21908902", c_opt = "2190.8902 N·s/m", x_peak = "0.75 mm", g_1 = "0.78077641", g_2 = "1.2807764", "ζ_tuned" = "0.29930475", "ζ_L" = "0.42163702", c_L = "4216.3702 N·s/m" }
tolerance = "1e-7"
checks = "fail"
+++

## Damped vibration absorber, most favourable design

A small mass m on a spring k_a and a viscous damper c, attached to a main mass M on its spring K, cuts the main mass's resonance. The main system is taken as undamped and driven by a harmonic force P_0 on M; the absorber's own damping is c over c_c = 2 m Ω_n, the critical damping of the absorber mass at the main natural frequency. After Den Hartog, Mechanical Vibrations, ch. III.

Every response curve of the main mass passes through two fixed points whatever the damping. Den Hartog's most favourable design tunes the absorber to make the two equally high (Eq. 63), then picks the damping that puts the curve's maximum at them (Eq. 69). For an absorber of your own tuning and damping, use the damped absorber response template.

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

The mass ratio μ sets how much an absorber can do: a larger one helps more, at the price of weight and of a large relative motion in the absorber spring.

```calc
> Mass ratio
μ = m / M = ??
> Natural frequency of the main system alone
Ω_n = sqrt(K / M) = ?? rad/s
> Static deflection of the main mass under P_0
x_st = P_0 / K = ?? mm
```

### Most favourable tuning and damping

```calc
> Most favourable tuning ω_a / Ω_n, Eq. 63
f_opt = 1 / (1 + μ) = ??
> Absorber spring
k_a = m (f_opt Ω_n)^2 = ?? kN/m
> Most favourable damping c / c_c, Eq. 69
ζ_opt = sqrt(3 μ / (8 (1 + μ)^3)) = ??
> Absorber damper
c_opt = ζ_opt 2 m Ω_n = ?? N·s/m
> Height of the fixed points, the resonant magnification left, Eq. 64
β_opt = sqrt(1 + 2 / μ) = ??
> Resonant amplitude of the main mass
x_peak = β_opt x_st = ?? mm
```

The main mass's amplitude for any tuning f, damping z and forcing ratio g = ω/Ω_n is Eq. 57. Its largest value over g from 0.3 to 2 confirms the design: Eq. 69 is an approximation, so the curve's maximum lies a per cent or so above the fixed points.

```calc
function absorberRatio(μ, f, z, g)
    a = (2 z g)^2
    n = a + (g^2 - f^2)^2
    d = a (g^2 - 1 + μ g^2)^2 + (μ f^2 g^2 - (g^2 - 1) (g^2 - f^2))^2
    return sqrt(n / d)
end
```

```calc
function absorberPeak(μ, f, z)
    best = 0
    for i in 0:6000
        best = max(best, absorberRatio(μ, f, z, 0.3 + 1.7 i / 6000))
    end
    return best
end
```

```calc {description="Largest x_1 / x_st on Eq. 57's curve for the most favourable design"}
β_curve = absorberPeak(μ, f_opt, ζ_opt) = ??
```

### Alternatives

An undamped absorber tuned to the main system (f = 1) removes the resonance at Ω_n but leaves two new ones either side of it (Eq. 50), spreading apart as μ grows. Tuned to f = 1, the damping that does best is Eq. 69a. A Lanchester damper is a mass coupled by a damper alone (no spring, f = 0); its resonant amplitude 1 + 2/μ is higher than any tuned absorber of the same mass leaves (Eqs. 66, 67, 69b).

```calc
> Lower natural frequency of the undamped tuned absorber, over Ω_n, Eq. 50
g_1 = sqrt(1 + μ / 2 - sqrt(μ + μ^2 / 4)) = ??
> Upper natural frequency, Eq. 50
g_2 = sqrt(1 + μ / 2 + sqrt(μ + μ^2 / 4)) = ??
> Optimum damping c / c_c when tuned to the main system, Eq. 69a
ζ_tuned = sqrt(μ (μ + 3 + sqrt(μ / (μ + 2))) / (8 (1 + μ))) = ??
```

```calc
> Resonant magnification of a Lanchester damper, Eq. 67
β_L = 1 + 2 / μ = ??
> Its optimum damping c / c_c, Eq. 69b
ζ_L = sqrt(1 / (2 (2 + μ) (1 + μ))) = ??
> Its damper
c_L = ζ_L 2 m Ω_n = ?? N·s/m
```

### Check

```calc {description="Largest acceptable amplitude of the main mass"}
x_allow = 1 mm_
```

```calc
@test x_peak ≤ x_allow
```
