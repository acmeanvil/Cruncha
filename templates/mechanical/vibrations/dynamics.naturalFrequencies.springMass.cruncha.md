+++
cruncha-template = 1
id = "dynamics.naturalFrequencies.springMass"
version = "1.0.0"
title = "Natural frequency of a mass on a spring that has mass"
summary = "The natural frequency of a mass carried by a spring or beam whose own mass is not negligible, ωn = √(k/(M + share·m)), by Rayleigh's method: a third of a coil spring's mass, 0.23 of a cantilever's, about half of a simply supported beam's, moves with the end mass. With the massless-spring frequency for comparison, and a forcing frequency's separation checked."
category = ["Mechanical engineering", "Vibrations"]
tags = ["natural frequency", "spring mass", "effective mass", "Rayleigh", "coil spring", "cantilever", "beam", "single degree of freedom", "resonance", "Den Hartog"]
from = "lacewing:dynamics.naturalFrequencies"
license = "GPL-3.0-or-later"
inputs = ["k", "M", "m_s", "α_m", "ω_f", "s_min"]
outputs = ["M_eff", "ω_n", "f_n", "ω_0", "s"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Appendix II, A collection of formulas, Part III: Eq. 12 (a spring with its own mass, m/3), Eqs. 15 and 16 (a cantilever and a simply supported beam with their own mass); §2.3 (Rayleigh's method)"

[[tests]]
name = "Lacewing Lab: coil spring of 3 kg carrying 1 kg"
source = "Lacewing Lab vibration_test (Den Hartog 4th Eq. 12): k = 1000 N/m, M = 1 kg, spring mass 3 kg with a third moving: ωn = √(1000/(1 + 1)) = √500 = 22.360680 rad/s"
inputs = { k = "1000 N/m", M = "1 kg", m_s = "3 kg", "α_m" = "0.3333333333333", "ω_f" = "10 rad/s" }
expect = { M_eff = "2 kg", "ω_n" = "22.360680 rad/s" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: the Lacewing Lab page's opening case, forced near resonance"
source = "By hand: k = 100 kN/m, M = 10 kg, spring mass 2 kg (coil spring, a third): ωn = √(1e5/10.666667) = 96.824584 rad/s, fn = 15.410111 Hz, against 100 rad/s with the spring's mass ignored. Forcing at 1000 rpm (104.72 rad/s) is 8.2 % above"
inputs = { k = "100 kN/m", M = "10 kg", m_s = "2 kg", "α_m" = "0.3333333333333", "ω_f" = "1000 rpm", s_min = "0.2" }
expect = { "ω_n" = "96.824584 rad/s", f_n = "15.410111 Hz", "ω_0" = "100 rad/s", s = "0.0815410" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: cantilever of 3 kg carrying 1 kg"
source = "By hand (Den Hartog Eq. 15, 0.23 of the beam's mass): ωn = √(1000/(1 + 0.69)) = 24.325213 rad/s"
inputs = { k = "1000 N/m", M = "1 kg", m_s = "3 kg", "α_m" = "0.23", "ω_f" = "10 rad/s" }
expect = { "ω_n" = "24.325213 rad/s" }
tolerance = "1e-7"
checks = "pass"
+++

## Natural frequency of a mass on a spring that has mass

A mass M on a spring, or at the end or middle of a beam, whose own mass m_s is not negligible. Rayleigh's method assumes the spring deflects in the shape it takes under a static load at the mass; its kinetic energy then equals that of a fraction α_m of its mass moving with M. The result is close (within a per cent or so) while the spring's mass is up to about the end mass. For a heavier spring, or for the spring's own surge modes, use the coil spring templates' surge frequency or the beam template.

### System

```calc {description="Stiffness of the spring or beam at the mass"}
k = 100 kN/m_
```

```calc {description="Mass carried"}
M = 10 kg_
```

```calc {description="Spring's or beam's own mass"}
m_s = 2 kg_
```

The share α_m of the spring's mass that moves with M, from Den Hartog Appendix II: 1/3 for a coil spring (Eq. 12), 0.23 for a cantilever with the mass at its tip (Eq. 15), 0.5 for a simply supported beam with the mass at mid-span (Eq. 16; Rayleigh's exact share is 17/35 = 0.486).

```calc {description="Share of the spring's mass moving with M, Den Hartog App. II Eqs. 12, 15, 16"}
α_m = 0.3333333333_
```

```calc
> Effective mass
M_eff = M + α_m m_s = ?? kg
> Natural frequency, Den Hartog Eqs. 12, 15, 16
ω_n = sqrt(k / M_eff) = ?? rad/s
f_n = ω_n / (2 π) = ?? Hz
> Natural frequency with the spring's mass ignored
ω_0 = sqrt(k / M) = ?? rad/s
```

### Separation from the forcing frequency

```calc {description="Forcing (circular) frequency"}
ω_f = 3000 rpm_
```

```calc {description="Smallest acceptable separation, as a fraction of the natural frequency"}
s_min = 0.2_
```

```calc {description="Separation of the forcing frequency from the natural frequency"}
s = abs(ω_f / ω_n - 1) = ??
```

```calc
@test s ≥ s_min
```
