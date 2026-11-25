+++
cruncha-template = 1
id = "dynamics.naturalFrequencies.circularPlate"
version = "1.0.0"
title = "Natural frequency of a uniform circular plate"
summary = "A natural frequency of a thin, uniform circular plate, ωn = a √(D/(μ1 r⁴)) with the flexural rigidity D = Et³/(12(1 − ν²)) and the mass per area μ1 = ρt, for the edge condition and mode whose coefficient a you give from the table (clamped edge, free edge, clamped at the centre). Checks a forcing frequency's separation from it."
category = ["Mechanical engineering", "Vibrations"]
tags = ["natural frequency", "plate", "circular plate", "diaphragm", "disk", "bending vibration", "flexural rigidity", "clamped edge", "free edge", "nodal diameter", "nodal circle", "Den Hartog", "Blevins"]
from = "lacewing:dynamics.naturalFrequencies"
license = "GPL-3.0-or-later"
inputs = ["E", "ν", "ρ", "t", "r", "a_p", "ω_f", "s_min"]
outputs = ["D", "μ_1", "ω_n", "f_n", "s"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Appendix II, A collection of formulas: Eq. 6a (plate constant D), Part VI, Eq. 25 and its coefficients a for circular plates"

[[references]]
work = "Blevins, Formulas for Natural Frequency and Mode Shape"
edition = "1979 (Krieger reprint 2001)"
locator = "Ch. 11, Table 11-1 (circular plates: λ² for the edge conditions and mode numbers)"

[[tests]]
name = "Hand check: 2 mm steel plate of 100 mm radius, clamped edge (the Lacewing Lab page's opening case)"
source = "By hand: D = 200e9 × 0.002³/(12 × 0.91) = 146.52015 N·m, μ1 = 7850 × 0.002 = 15.7 kg/m², a = 10.21 (Den Hartog Eq. 25, clamped edge, fundamental): ωn = 10.21 √(146.52015/(15.7 × 0.1⁴)) = 3119.0662 rad/s, fn = 496.41480 Hz. Forcing at 3000 rpm (314.16 rad/s) is far below"
inputs = { E = "200 GPa", "ν" = "0.3", "ρ" = "7850 kg/m³", t = "2 mm", r = "100 mm", a_p = "10.21", "ω_f" = "3000 rpm" }
expect = { D = "146.52015 N·m", "μ_1" = "15.7 kg/m²", "ω_n" = "3119.0662 rad/s", f_n = "496.41480 Hz", s = "0.89927778" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: the same plate with free edges and two nodal diameters, forced at 250 Hz"
source = "By hand: a = 5.25 (Den Hartog Eq. 25, free edge, two perpendicular nodal diameters): ωn = 5.25 × 305.49130 = 1603.8293 rad/s; forcing at 2π × 250 = 1570.7963 rad/s is 2.1 % away"
inputs = { E = "200 GPa", "ν" = "0.3", "ρ" = "7850 kg/m³", t = "2 mm", r = "100 mm", a_p = "5.25", "ω_f" = "1570.7963 rad/s", s_min = "0.2" }
expect = { "ω_n" = "1603.8293 rad/s", s = "0.0205963" }
tolerance = "1e-5"
checks = "fail"
+++

## Natural frequency of a uniform circular plate

A thin, flat circular plate of uniform thickness and material vibrating in bending, by classical (Kirchhoff) plate theory: the thickness small against the radius (under about a tenth), deflections small against the thickness, and no in-plane load or fluid around it (water or air loading lowers the frequency). After Den Hartog, Mechanical Vibrations, Appendix II, Eqs. 6a and 25.

### Plate

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Density"}
ρ = 7850 kg/m³_
```

```calc {description="Thickness"}
t = 2 mm_
```

```calc {description="Radius to the edge"}
r = 100 mm_
```

The coefficient a depends on the edge condition and the mode (its nodal diameters and circles). Take it from Den Hartog's list under Eq. 25 or Blevins Table 11-1 (λ²); Den Hartog's: 10.21 clamped edge, fundamental; 5.25 free edge, two perpendicular nodal diameters (the lowest mode of a free plate); 9.07 free edge, one nodal circle; 3.75 free edge, clamped at the centre (umbrella mode). Den Hartog's free-edge values are for ν = 0.3.

```calc {description="Frequency coefficient a for the edge condition and mode, Den Hartog App. II Eq. 25 or Blevins Table 11-1 (λ²)"}
a_p = 10.21_
```

```calc
> Plate constant (flexural rigidity), Eq. 6a
D = E t^3 / (12 (1 - ν^2)) = ?? N·m
> Mass per area
μ_1 = ρ t = ?? kg/m²
> Natural frequency, Eq. 25
ω_n = a_p sqrt(D / (μ_1 r^4)) = ?? rad/s
f_n = ω_n / (2 π) = ?? Hz
```

### Separation from the forcing frequency

```calc {description="Forcing (circular) frequency"}
ω_f = 3000 rpm_
```

```calc {description="Smallest acceptable separation, as a fraction of the natural frequency"}
s_min = 0.2_
```

```calc {description="Separation of the forcing frequency from the mode"}
s = abs(ω_f / ω_n - 1) = ??
```

```calc
@test s ≥ s_min
```
