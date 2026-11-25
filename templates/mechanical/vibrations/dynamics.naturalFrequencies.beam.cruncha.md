+++
cruncha-template = 1
id = "dynamics.naturalFrequencies.beam"
version = "1.0.0"
title = "Natural frequencies of a uniform beam in bending"
summary = "The transverse (bending) natural frequencies of a uniform Euler-Bernoulli beam, ωn = an √(EI/(μ1 l⁴)), for the end conditions whose mode coefficients an you give from the table: cantilever, simply supported, free-free, clamped-clamped, clamped-hinged. Checks a forcing frequency's separation from every mode."
category = ["Mechanical engineering", "Vibrations"]
tags = ["natural frequency", "beam", "bending vibration", "transverse vibration", "cantilever", "simply supported", "clamped", "free-free", "mode", "Euler-Bernoulli", "resonance", "Den Hartog", "Blevins"]
from = "lacewing:dynamics.naturalFrequencies"
license = "GPL-3.0-or-later"
inputs = ["E", "ρ", "A", "I", "l", "a", "ω_f", "s_min"]
outputs = ["μ_1", "ω", "f", "ω_1", "f_1", "s"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Appendix II, A collection of formulas, Part IV: Eq. 21 and its table of the coefficients an for the first five modes"

[[references]]
work = "Blevins, Formulas for Natural Frequency and Mode Shape"
edition = "1979 (Krieger reprint 2001)"
locator = "Ch. 8, Table 8-1 (single-span beams: λi for the end conditions, an = λi²)"

[[tests]]
name = "Lacewing Lab: simply supported, a1 = 9.87"
source = "Lacewing Lab vibration_test (Den Hartog 4th Eq. 21): E = 2·10¹¹ Pa, I = 10⁻⁸ m⁴, μ1 = 2 kg/m (here ρ = 2000 kg/m³ on 10⁻³ m²), l = 1 m: ω1 = 9.87 √(2000/2) = 312.12 rad/s; the exact π² gives 312.104"
inputs = { E = "200 GPa", "ρ" = "2000 kg/m³", A = "1000 mm²", I = "10000 mm⁴", l = "1 m", a = "[9.87; 39.5; 88.9; 158; 247]", "ω_f" = "100 rad/s" }
expect = { "μ_1" = "2 kg/m", "ω_1" = "312.12 rad/s" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Hand check: 1 m steel cantilever of 20 mm round bar (the Lacewing Lab page's opening case)"
source = "By hand: A = π 20²/4 = 314.15927 mm², I = π 20⁴/64 = 7853.9816 mm⁴, μ1 = 7850 × 3.1415927e-4 = 2.4661502 kg/m, √(EI/(μ1 l⁴)) = √(1570.7963/2.4661502) = 25.237724 rad/s; a = 3.52, 22.0, 61.7: ω = 88.836786, 555.22991, 1557.1675 rad/s, f1 = 14.138814 Hz. Forcing at 90 rad/s is 1.3 % from the first mode"
inputs = { E = "200 GPa", "ρ" = "7850 kg/m³", A = "314.15927 mm²", I = "7853.9816 mm⁴", l = "1 m", a = "[3.52; 22.0; 61.7; 121; 200]", "ω_f" = "90 rad/s", s_min = "0.2" }
expect = { "μ_1" = "2.4661502 kg/m", "ω_1" = "88.836786 rad/s", f_1 = "14.138814 Hz", s = "0.01309384" }
tolerance = "1e-5"
checks = "fail"
+++

## Natural frequencies of a uniform beam in bending

The bending (transverse) natural frequencies of a straight beam of uniform section and material, by Euler-Bernoulli theory: shear deformation and rotary inertia are left out, which is good while the beam is long compared with its depth (about ten times, for the modes listed). After Den Hartog, Mechanical Vibrations, Appendix II, Eq. 21; Blevins gives the same coefficients with more end conditions.

### Beam

For a solid round bar of diameter d, A = π d²/4 and I = π d⁴/64. I is the second moment of area about the axis the beam bends about.

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Density"}
ρ = 7850 kg/m³_
```

```calc {description="Cross-section area"}
A = 314.159 mm²_
```

```calc {description="Second moment of area about the bending axis"}
I = 7853.98 mm⁴_
```

```calc {description="Length between the ends"}
l = 1 m_
```

### Mode coefficients

The coefficients a_n = (β_n l)² of the first modes depend only on the end conditions. Take them from Den Hartog's table under Eq. 21 or Blevins Table 8-1 (λ_i², λ_i being his frequency parameter): about 3.52, 22.0, 61.7, 121, 200 for a cantilever; (nπ)², 9.87, 39.5, 88.9, 158, 247, simply supported; 22.4, 61.7, 121, 200, 298 for free-free and for clamped-clamped; 15.4, 50.0, 104, 178, 272 for clamped-hinged. A hinged-free beam has those last four after a rigid-body mode at zero: leave the zero out.

```calc {description="Mode coefficients a_n for the end conditions, Den Hartog App. II Eq. 21 table or Blevins Table 8-1 (λ_i²)"}
a = [3.52; 22.0; 61.7; 121; 200]_
```

```calc
> Mass per length
μ_1 = ρ A = ?? kg/m
> Natural frequencies, Den Hartog Eq. 21
ω = a sqrt(E I / (μ_1 l^4)) = ?? rad/s
> The same in cycles per second
f = ω / (2 π) = ?? Hz
```

```calc
> Fundamental
ω_1 = ω[1] = ?? rad/s
f_1 = ω_1 / (2 π) = ?? Hz
```

### Separation from the forcing frequency

Keep forcing frequencies (running speeds, blade passing, vortex shedding) away from every natural frequency. The check asks the nearest mode to be at least s_min away, as a fraction of its frequency.

```calc {description="Forcing (circular) frequency"}
ω_f = 30 rad/s_
```

```calc {description="Smallest acceptable separation, as a fraction of the natural frequency"}
s_min = 0.2_
```

```calc {description="Separation of the forcing frequency from the nearest mode"}
s = min(abs(ω_f / ω - 1)) = ??
```

```calc
@test s ≥ s_min
```
