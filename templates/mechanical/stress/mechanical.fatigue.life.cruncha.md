+++
cruncha-template = 1
id = "mechanical.fatigue.life"
version = "1.0.0"
title = "Fatigue: finite life on the S-N line and Miner's rule (Shigley)"
summary = "The S-N line of a steel part from its endurance limit and ultimate strength (fatigue strength fraction f, Sf = a·N^b), each load level of a duty cycle reduced to an equivalent fully reversed stress by Gerber, and the damage summed by Miner's rule."
category = ["Mechanical engineering", "Stress and failure"]
tags = ["fatigue", "S-N", "finite life", "cycles to failure", "Miner", "Palmgren-Miner", "cumulative damage", "duty cycle", "Gerber", "Basquin", "Shigley"]
from = "lacewing:mechanical.fatigue"
license = "GPL-3.0-or-later"
inputs = ["S_ut", "S_y", "S_e", "n_i", "σ_(a,i)", "σ_(m,i)", "N_req"]
outputs = ["f", "a_SN", "b_SN", "σ_rev", "σ_(rev,max)", "σ_peak", "D", "D_tot"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 6-7 and 6-15: Eqs. 6-10 to 6-12 and Fig. 6-18 (fatigue strength fraction f), Eqs. 6-13 to 6-16 (S-N line), Eq. 6-17 (below 10³ cycles), Example 6-12 (equivalent reversed stress), Eq. 6-58 (Miner's rule); Examples 6-7 and 6-15"

[[tests]]
name = "Shigley Ex. 6-15: three cycles of (σa, σm) = (70, 10), (10, 50), (10, −30) kpsi"
source = "Lacewing Lab stress_test (duty cycle, Gerber): Sut 151 kpsi, Se 67.5 kpsi, Sy 120 kpsi; cycle 1 σrev = 70.3 kpsi with N about 619 000 (Shigley reads f = 0.795 off Fig. 6-18; Eqs. 6-10 to 6-12 give 0.797, so N is about 0.7 % lower: 614 500), the other two infinite. D = 1/N per block"
inputs = { S_ut = "151 ksi", S_y = "120 ksi", S_e = "67.5 ksi", n_i = "[1, 1, 1]", "σ_(a,i)" = "[70, 10, 10] ksi", "σ_(m,i)" = "[10, 50, -30] ksi", N_req = "100000" }
expect = { f = "0.797", "σ_(rev,max)" = "70.31 ksi", D = "1.6155e-6", D_tot = "0.16155" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 6-7: reversed 403 MPa on a 690 MPa steel with Se = 280 MPa"
source = "Lacewing Lab stress_test: about 32 300 cycles at a reversed σ′a = 403 MPa (Shigley reads f = 0.845 off Fig. 6-18; Eqs. 6-10 to 6-12 give 0.8436, so N is about 0.5 % shorter: 32 150). One level, given as two with the second at zero cycles"
inputs = { S_ut = "690 MPa", S_y = "600 MPa", S_e = "280 MPa", n_i = "[1, 0]", "σ_(a,i)" = "[403, 0] MPa", "σ_(m,i)" = "[0, 0] MPa", N_req = "10000" }
expect = { f = "0.8436", a_SN = "1210 MPa", b_SN = "-0.1059", D = "3.0960e-5" }
tolerance = "1%"
checks = "pass"

[[tests]]
name = "Ex. 6-15's block needed a million times: Miner's sum exceeds 1"
source = "As Ex. 6-15, 10⁶ repetitions of a block that fails after about 619 000 (Lacewing Lab: 614 500)"
inputs = { S_ut = "151 ksi", S_y = "120 ksi", S_e = "67.5 ksi", n_i = "[1, 1, 1]", "σ_(a,i)" = "[70, 10, 10] ksi", "σ_(m,i)" = "[10, 50, -30] ksi", N_req = "1000000" }
expect = { D_tot = "1.6273" }
tolerance = "0.01%"
checks = "fail"

[[tests]]
name = "Below 10³ cycles: reversed 0.85 Sut, which also yields"
source = "Lacewing Lab duty-cycle test: below f·Sut the line runs to Sut at one cycle (Eq. 6-17). By hand: Sut 151 kpsi, f = 0.79703, N = 0.85^(3/log 0.79703) = 141.0 cycles, D = 0.007093; σa = 128.35 kpsi exceeds Sy = 120 kpsi, so the yield check fails"
inputs = { S_ut = "151 ksi", S_y = "120 ksi", S_e = "67.5 ksi", n_i = "[1, 0]", "σ_(a,i)" = "[128.35, 0] ksi", "σ_(m,i)" = "[0, 0] ksi", N_req = "1" }
expect = { D = "0.0070934" }
tolerance = "0.01%"
checks = "fail"
+++

## Finite life and Miner's rule

The life of a steel part under fully reversed or fluctuating stress beyond its endurance limit, by the stress-life method (high-cycle fatigue, about 10³ cycles and more). Below 10³ cycles the line is extended to Sut at one cycle, but low-cycle fatigue is better treated by the strain-life method.

The S-N line is straight on log-log axes from f Sut at 10³ cycles to S_e at 10⁶ (Sf = a N^b). The fraction f follows from Sut by Eqs. 6-10 to 6-12 (Shigley's Fig. 6-18), with the true fracture strength approximated by σ′F = Sut + 345 MPa (50 kpsi) and S′e by Eq. 6-8; below Sut = 70 kpsi, f = 0.9. At or below S_e the life is taken as infinite, which holds for steels. Aluminum alloys and other metals with no endurance limit need a measured S-N curve.

### Strengths

```calc {description="Tensile ultimate strength"}
S_ut = 151 ksi_
```

```calc {description="Tensile yield strength"}
S_y = 120 ksi_
```

```calc {description="Endurance limit of the part (Marin factors), at 10⁶ cycles"}
S_e = 67.5 ksi_
```

### The S-N line

```calc
> True fracture strength, approximated as Sut + 50 kpsi (Sec. 6-7)
σ_F′ = S_ut + 345 'MPa' = ?? ksi
> Rotating-beam endurance limit of Eq. 6-8, to which Fig. 6-18 is drawn
S_(e,ref) = { 0.5 S_ut if S_ut ≤ 1400 'MPa'; 700 'MPa' otherwise } = ?? ksi
> Exponent of the specimen's line, Eq. 6-11
b_F = -log10(σ_F′ / S_(e,ref)) / log10(2e6) = ??
```

```calc {description="Fraction of Sut that is the fatigue strength at 10³ cycles, Eq. 6-10 (Fig. 6-18)"}
f = { 0.9 if S_ut < 70 'ksi'; σ_F′ / S_ut × (2e3)^b_F otherwise } = ??
```

```calc {description="S-N line coefficient, Eq. 6-14"}
a_SN = (f S_ut)^2 / S_e = ?? ksi
```

```calc {description="S-N line exponent, Eq. 6-15"}
b_SN = -log10(f S_ut / S_e) / 3 = ??
```

### Duty cycle

One repetition of the duty cycle has load levels i, each with n_i cycles at an alternating stress σ_(a,i) and a mean stress σ_(m,i). Enter the stresses at the critical point as von Mises equivalents with the notch factors applied (σ′a and σ′m of the fluctuating-stress template). Give each level its own entry, at least two: a one-entry vector `[x]` reads as a plain number, so add a level with zero cycles for a single load.

```calc {description="Cycles of each level in one repetition of the duty cycle"}
n_i = [1, 1, 1]_
```

```calc {description="Alternating stress of each level (von Mises, notch included)"}
σ_(a,i) = [70, 10, 10] ksi_
```

```calc {description="Mean stress of each level (von Mises, notch included)"}
σ_(m,i) = [10, 50, -30] ksi_
```

```calc {description="Repetitions of the duty cycle required"}
N_req = 100000_
```

Each level becomes the fully reversed stress that is as damaging, by the Gerber line through (σm, σa) (Shigley Ex. 6-12): σrev = σa / (1 − (σm/Sut)²). A compressive mean gets no benefit (σrev = σa).

```calc {description="Equivalent fully reversed stress of each level, Gerber"}
σ_rev = σ_(a,i) / (1 - ((σ_(m,i) + abs(σ_(m,i))) / (2 S_ut))^2) = ?? ksi
```

```calc {description="Largest equivalent reversed stress of the duty cycle"}
σ_(rev,max) = max(σ_rev) = ?? ksi
```

### Damage by Miner's rule

The life N of each level is read off the line: infinite at or below S_e, (σrev / a)^(1/b) between 10³ and 10⁶ cycles (Eq. 6-16), and Sut N^(log f / 3) above f Sut (Eq. 6-17). Miner's rule (Eq. 6-58) sums n/N over the levels; the duty cycle can be repeated 1/D times before failure. The rule ignores the order of the loads, and real damage at failure scatters widely about 1.

```calc
function minerDamage(n, rev, se, sut, f, a, b)
    d = 0
    for i in 1:length(n)
        s = rev[i]
        if s > se
            if s ≥ f sut
                d = d + n[i] / (s / sut)^(3 / log10(f))
            else
                d = d + n[i] / (s / a)^(1 / b)
            end
        end
    end
    return d
end
```

```calc {description="Damage of one repetition of the duty cycle, Eq. 6-58"}
D = minerDamage(n_i, σ_rev, S_e, S_ut, f, a_SN, b_SN) = ??
```

```calc {description="Damage over the required repetitions"}
D_tot = N_req D = ??
```

### First-cycle yield

The method assumes the part doesn't yield: the largest stress of every level, σa + |σm|, must stay below S_y (which also keeps every mean stress below Sut).

```calc {description="Largest stress of the duty cycle, σa + |σm|"}
σ_peak = max(σ_(a,i) + abs(σ_(m,i))) = ?? ksi
```

### Checks

```calc
@test D_tot ≤ 1
```

```calc
@test σ_peak < S_y
```
