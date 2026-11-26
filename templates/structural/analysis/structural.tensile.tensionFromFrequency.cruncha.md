+++
cruncha-template = 1
id = "structural.tensile.tensionFromFrequency"
version = "1.0.0"
title = "Tension from a measured frequency: rod, bar or cable, pinned ends"
summary = "The tension in a rod, bar or cable from a measured natural frequency and its mode number, with the member's bending stiffness included for pinned ends, the taut-string tension for comparison, the share of the frequency due to bending, and the stress."
category = ["Structural engineering", "Analysis"]
tags = ["tension", "frequency", "vibration", "taut string", "tie rod", "cable", "stay", "hanger", "tension measurement", "Den Hartog"]
from = "lacewing:structural.tensileFrequency"
license = "GPL-3.0-or-later"
inputs = ["ρ", "A", "E", "I", "σ_u", "L", "f", "n"]
outputs = ["μ", "T", "T_s", "e_s", "s_b", "σ", "r_u"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "3rd (McGraw-Hill, 1947)"
locator = "§31, Strings and organ pipes, pp. 170 to 175: the equation of the string (Eq. 93) and its natural frequencies ωn = (nπ/l)√(T/μ1) from Eqs. 94 to 97 (Lacewing Lab cites the same section of the 4th edition)"

[[tests]]
name = "Lacewing Lab's opening case: 25 mm A36 rod, 9.144 m, 7.5 Hz in mode 1"
source = "Lacewing Lab frequency page defaults (A36: ρ 7850 kg/m³, E 199.95 GPa, Fu 399.9 MPa): by hand, A = 490.874 mm², I = 19 174.76 mm⁴, μ = 3.85336 kg/m; string 4μL²f² = 72.4927 kN; less π²EI/L² = 0.45256 kN gives 72.0402 kN, 0.63 % below the string; s_b = 0.0062 (ξ = L√(T/EI) = 39.64). A beam-element eigenvalue check (200 elements, geometric stiffness) gives 7.5000 Hz at that tension"
inputs = { "ρ" = "7850 kg/m^3", A = "490.87385 mm^2", E = "199.95 GPa", I = "19174.760 mm^4", "σ_u" = "399.9 MPa", L = "9.144 m", f = "7.5 Hz", n = "1" }
expect = { "μ" = "3.8533597 kg/m", T = "72.040176 kN", T_s = "72.492739 kN", e_s = "0.0062820861", s_b = "0.0062428683", "σ" = "146.75904 MPa", r_u = "0.36698935" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "A short, stiff rod in mode 2: the string is 43 % high"
source = "By hand: the same rod, 3 m at 40 Hz in mode 2: string 4 × 3.85336 × 9 × 1600/4 = 55.4884 kN, less 4π² × 3833.99/9 = 16.8178 kN gives 38.6706 kN; s_b = 16.8178/55.4884 = 0.303086 (ξ = 9.53)"
inputs = { "ρ" = "7850 kg/m^3", A = "490.87385 mm^2", E = "199.95 GPa", I = "19174.760 mm^4", "σ_u" = "399.9 MPa", L = "3 m", f = "40 Hz", n = "2" }
expect = { T = "38.670604 kN", T_s = "55.48838 kN", e_s = "0.4348982", s_b = "0.30308645" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "1 in. steel rod, 20 ft, 12 Hz"
source = "By hand: A = 0.785398 in², I = 0.0490874 in⁴, μ = 490 lb/ft³ × A = 3.97717 kg/m, string 4μL²f² = 19.1382 kips, less π² × 29e6 × 0.0490874 lbf·in²/240² = 0.24392 kips gives 18.8942 kips; s_b = 0.0127451"
inputs = { "ρ" = "490 lb/ft^3", A = "0.78539816 in^2", E = "29000 ksi", I = "0.049087385 in^4", "σ_u" = "58 ksi", L = "20 ft", f = "12 Hz", n = "1" }
expect = { "μ" = "3.9771708 kg/m", T = "18.894243 kips", T_s = "19.138162 kips", s_b = "0.012745149" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "A cable as a taut string: I = 0"
source = "By hand: a strand of 1000 mm² metallic area at 7850 kg/m³ (μ = 7.85 kg/m), 30 m between sockets, 2.0 Hz in mode 1: T = 4 × 7.85 × 900 × 4 = 113.04 kN, the string exactly"
inputs = { "ρ" = "7850 kg/m^3", A = "1000 mm^2", E = "160 GPa", I = "0 mm^4", "σ_u" = "1500 MPa", L = "30 m", f = "2 Hz", n = "1" }
expect = { T = "113.04 kN", T_s = "113.04 kN", e_s = "0", s_b = "0" }
tolerance = "1e-9"
checks = "pass"

[[tests]]
name = "Below the bending frequency alone: compression, the check fails"
source = "By hand: the 25 mm rod over 9.144 m has a pinned bending frequency of 0.593 Hz with no tension; 0.5 Hz would need a compressive load"
inputs = { "ρ" = "7850 kg/m^3", A = "490.87385 mm^2", E = "199.95 GPa", I = "19174.760 mm^4", "σ_u" = "399.9 MPa", L = "9.144 m", f = "0.5 Hz", n = "1" }
expect = { T_s = "322.18995 N" }
tolerance = "1e-6"
checks = "fail"
+++

## Tension from a measured frequency (pinned ends)

The axial tension in a rod, bar or cable stretched between two supports, from one of its measured natural frequencies of transverse vibration. A taut string vibrates at f = (n/2L)√(T/μ) (Den Hartog §31), so a measured frequency gives the tension directly. A rod or bar also resists bending, which raises its frequencies: the equation of motion EI w'''' − T w'' + μ ẅ = 0 with pinned (simply supported) ends gives exactly f = (n/2L)√(T/μ)·√(1 + n²π²EI/(TL²)), which solves for T in closed form. Ignoring the bending stiffness overestimates the tension.

Assumptions: a straight, uniform, linear elastic member, vibrating in a plane, with ends that hold position but don't restrain rotation. Real end fixity lies between pinned and clamped; a clamped (fixed) end gives a lower tension for the same frequency, and the clamped template gives that. A stranded cable bends far more easily than a solid bar of its diameter, so give a cable I = 0 (a string). The measured frequency must come from the free length L: dampers, clamps or attachments along it change the length that vibrates.

### Member

For a solid round rod of diameter d, A = πd²/4 and I = πd⁴/64; for a bar w × t, A = wt and I = wt³/12 about the weak axis (it vibrates about the axis with the lowest frequency). For a cable, A is the metallic area.

```calc {description="Density of the material"}
ρ = 7850 kg/m³_
```

```calc {description="Cross-section (metallic) area"}
A = 490.874 mm²_
```

```calc {description="Modulus of elasticity"}
E = 199.95 GPa_
```

```calc {description="Second moment of area about the axis it vibrates about (0 for a cable)"}
I = 19174.76 mm⁴_
```

```calc {description="Ultimate tensile strength, for the stress ratio"}
σ_u = 399.9 MPa_
```

### Measurement

The mode number counts half-waves along the length: 1 for the fundamental. A mode taken for the wrong one gives a tension off by the square of the ratio.

```calc {description="Free (vibrating) length between supports"}
L = 9.144 m_
```

```calc {description="Measured natural frequency"}
f = 7.5 Hz_
```

```calc {description="Mode number of the measured frequency (half-waves)"}
n = 1_
```

### Tension

```calc
> Mass per length
μ = ρ A = ?? kg/m
> Tension as a taut string, Den Hartog §31
T_s = 4 μ L^2 f^2 / n^2 = ?? kN
> Tension with the bending stiffness, pinned ends
T = T_s - n^2 π^2 E I / L^2 = ?? kN
> How far the string formula is high
e_s = T_s / T - 1 = ??
```

### Bending share and stress

s_b is the part of the string tension 4μL²f²/n² that the bending stiffness supplies instead of the tension: zero for a string, one for a beam with no tension. It is n²π²/(ξ² + n²π²) in terms of Lacewing Lab's stiffness parameter ξ = L√(T/EI): ξ above about 50 (s_b under about 0.4 % in mode 1) means bending hardly matters; larger s_b means that it does, and that the end fixity matters with it.

```calc
> Share of the string tension carried by bending
s_b = n^2 π^2 E I / (L^2 T_s) = ??
> Axial stress
σ = T / A = ?? MPa
> Share of the ultimate strength
r_u = σ / σ_u = ??
```

### Checks

A tension at or below zero means bending stiffness alone gives that frequency or more: the mode number is wrong, or the member is in compression.

```calc
@test T > 0 'N'
```

```calc
@test r_u < 1
```
