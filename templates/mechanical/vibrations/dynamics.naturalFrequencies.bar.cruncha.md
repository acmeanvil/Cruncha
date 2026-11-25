+++
cruncha-template = 1
id = "dynamics.naturalFrequencies.bar"
version = "1.0.0"
title = "Natural frequencies of a uniform bar, longitudinal and torsional"
summary = "The longitudinal (axial) natural frequencies of a uniform bar and the torsional natural frequencies of a uniform round shaft or tube, (n − ½)π or nπ times the wave speed over the length, clamped-free or with both ends alike (clamped-clamped or free-free). Checks a forcing frequency's separation from every mode."
category = ["Mechanical engineering", "Vibrations"]
tags = ["natural frequency", "bar", "rod", "shaft", "longitudinal vibration", "axial vibration", "torsional vibration", "wave speed", "clamped-free", "mode", "resonance", "Den Hartog", "Blevins"]
from = "lacewing:dynamics.naturalFrequencies"
license = "GPL-3.0-or-later"
inputs = ["E", "G", "ρ", "l", "ends", "ω_f", "s_min"]
outputs = ["c_L", "c_T", "ω_ax", "ω_tor", "ω_(ax,1)", "f_(ax,1)", "ω_(tor,1)", "f_(tor,1)", "s"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "Appendix II, A collection of formulas, Part IV: Eq. 19 (clamped-free bar, longitudinal), Eq. 20 (both ends clamped); the same with GIp for AE for a shaft in torsion"

[[references]]
work = "Blevins, Formulas for Natural Frequency and Mode Shape"
edition = "1979 (Krieger reprint 2001)"
locator = "Ch. 8, Table 8-16 (longitudinal and torsional vibration of uniform beams)"

[[tests]]
name = "Lacewing Lab: steel cantilever bar, longitudinal"
source = "Lacewing Lab vibration_test (Den Hartog 4th Eq. 19): E = 2·10¹¹ Pa, μ1 = 0.785 kg/m on 10⁻⁴ m² (ρ = 7850 kg/m³), l = 1 m: ω1 = ½π √(AE/(μ1 l²)) = ½π √(2·10⁷/0.785) = 7928.6646 rad/s"
inputs = { E = "200 GPa", "ρ" = "7850 kg/m³", l = "1 m", ends = "1", "ω_f" = "1000 rad/s" }
expect = { "ω_(ax,1)" = "7928.6646 rad/s" }
tolerance = "1e-7"
checks = "pass"

[[tests]]
name = "Hand check: 1 m steel bar and shaft, both ends clamped, forced near the first torsional mode"
source = "By hand: cL = √(200e9/7850) = 5047.5446 m/s, cT = √(79e9/7850) = 3172.3326 m/s; both ends alike, ω = nπc/l: ω_ax,1 = 15857.329 rad/s (2523.7723 Hz), ω_tor,1 = 9966.1769 rad/s (1586.1663 Hz). Forcing at 9000 rad/s is 9.7 % below the torsional mode"
inputs = { E = "200 GPa", G = "79 GPa", "ρ" = "7850 kg/m³", l = "1 m", ends = "2", "ω_f" = "9000 rad/s", s_min = "0.15" }
expect = { c_L = "5047.5446 m/s", c_T = "3172.3326 m/s", "ω_(ax,1)" = "15857.329 rad/s", "f_(ax,1)" = "2523.7723 Hz", "ω_(tor,1)" = "9966.1769 rad/s", "f_(tor,1)" = "1586.1663 Hz", s = "0.0969456" }
tolerance = "1e-6"
checks = "fail"
+++

## Natural frequencies of a uniform bar, longitudinal and torsional

A straight bar of uniform section and material vibrating along its axis, and a uniform round shaft or tube twisting about it. Both obey the wave equation: the frequencies are the wave speed over the length times (n − ½)π for a bar clamped at one end and free at the other (Den Hartog Eq. 19), or nπ with both ends clamped or both free (Eq. 20; the free-free bar also has a rigid-body mode at zero, not listed). For a shaft in torsion GI_p replaces AE and ρI_p replaces μ_1, so neither depends on the section; that holds for a circular section only, since other sections warp.

### Bar

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Shear modulus"}
G = 79 GPa_
```

```calc {description="Density"}
ρ = 7850 kg/m³_
```

```calc {description="Length between the ends"}
l = 1 m_
```

```calc {description="Ends: 1 clamped-free; 2 both clamped or both free"}
ends = 1_
```

```calc
> Wave speed along the bar
c_L = sqrt(E / ρ) = ?? m/s
> Torsional wave speed in a round shaft
c_T = sqrt(G / ρ) = ?? m/s
> Mode factors: n − ½ clamped-free, n both ends alike
n_m = {[0.5; 1.5; 2.5; 3.5; 4.5] if ends == 1; [1; 2; 3; 4; 5] if ends == 2} = ??
```

```calc
> Longitudinal natural frequencies, Den Hartog Eqs. 19 and 20
ω_ax = n_m π c_L / l = ?? rad/s
> Torsional natural frequencies of a round shaft
ω_tor = n_m π c_T / l = ?? rad/s
```

```calc
> First longitudinal mode
ω_(ax,1) = ω_ax[1] = ?? rad/s
f_(ax,1) = ω_(ax,1) / (2 π) = ?? Hz
> First torsional mode
ω_(tor,1) = ω_tor[1] = ?? rad/s
f_(tor,1) = ω_(tor,1) / (2 π) = ?? Hz
```

### Separation from the forcing frequency

The check asks the nearest longitudinal or torsional mode to be at least s_min away from the forcing frequency, as a fraction of its frequency. A mode that the forcing can't excite (an axial force can't drive torsion) needs no separation: judge which apply.

```calc {description="Forcing (circular) frequency"}
ω_f = 1000 rad/s_
```

```calc {description="Smallest acceptable separation, as a fraction of the natural frequency"}
s_min = 0.2_
```

```calc {description="Separation of the forcing frequency from the nearest mode"}
s = min(min(abs(ω_f / ω_ax - 1)), min(abs(ω_f / ω_tor - 1))) = ??
```

```calc
@test s ≥ s_min
```
