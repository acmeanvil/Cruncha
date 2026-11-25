+++
cruncha-template = 1
id = "mechanical.adhesiveJoints.singleLap"
version = "1.0.0"
title = "Adhesive single-lap joint, Hart-Smith elastic analysis"
summary = "Average shear, the peak shear with the adherends bending (Hart-Smith's end moment, with Goland and Reissner's for comparison), Volkersen's shear-lag peak without bending, and the peak peel stress at the ends of a balanced single-lap bonded joint, with the lap-shear strength over the average shear checked against a design factor."
category = ["Mechanical engineering", "Fastening"]
tags = ["adhesive", "bonded joint", "single lap", "lap shear", "peel stress", "Hart-Smith", "Goland-Reissner", "Volkersen", "shear lag", "ASTM D1002", "epoxy", "Shigley", "Juvinall"]
from = "lacewing:mechanical.adhesiveJoints"
license = "GPL-3.0-or-later"
inputs = ["E", "ν", "t", "G", "E_a", "η", "b", "l", "P", "S_s", "n_d"]
outputs = ["τ_av", "D", "ξc", "k", "k_GR", "M_0", "λ", "λ′", "τ_max", "r_40", "τ_V", "χc", "σ_max", "σ_long", "n_av"]

[[references]]
work = "Hart-Smith, L. J., Adhesive-Bonded Single-Lap Joints, NASA CR-112236"
edition = "1973"
locator = "Sec. 2: Eqs. 5, 11 (flexural rigidity), 26-29, 37 (shear), 39 (end moment factor), 40-41 (peak to average shear), 47-49 (Goland and Reissner's factor); Sec. 5: Eqs. 97-105 and 113 (peel stress)"

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 9-9, Fig. 9-28 (single-lap shear: Volkersen, Goland and Reissner, with bending), Table 9-7 (lap-shear strengths by adhesive chemistry)"

[[references]]
work = "Juvinall & Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 11.8 (adhesives: design guidance, average lap shear)"

[[tests]]
name = "ASTM D1002 coupon: 0.064 in aluminium, 1/2 in overlap, 1000 lbf, epoxy"
source = "Lacewing Lab adhesive_joints_test coupon (after Shigley 10th Fig. 9-28: E 10 Mpsi, ν 0.33, t 0.064 in, epoxy Ea 500 ksi with G = Ea/2.7, b 1 in, l 0.5 in, 1000 lbf; a 0.005 in bond line taken); by hand from Hart-Smith's equations: D = 245.1502 lbf·in, ξc = 0.504922, k = 0.646240 (Goland-Reissner 0.666833), M0 = 22.29528 lbf·in/in, τmax = 8476.105 psi = 4.238053 τav (Eq. 40), Volkersen 5428.988 psi, χc = 5.312812, σmax = 10069.81 psi (Eq. 113: 10068.90 psi)"
inputs = { E = "10 Mpsi", "ν" = "0.33", t = "0.064 in", G = "185185.185185 psi", E_a = "500 ksi", "η" = "0.005 in", b = "1 in", l = "0.5 in", P = "1000 lbf", S_s = "4500 psi", n_d = "2" }
expect = { "τ_av" = "2000 psi", D = "245.1501889 lbf·in", "ξc" = "0.5049215335", k = "0.6462400974", k_GR = "0.6668332039", M_0 = "22.29528336 lbf", "τ_max" = "8476.105373 psi", r_40 = "4.238052687", "τ_V" = "5428.988154 psi", "χc" = "5.312812043", "σ_max" = "10069.80862 psi", "σ_long" = "10068.89663 psi" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Long overlap: Hart-Smith's limits"
source = "Lacewing Lab adhesive_joints_test, after Hart-Smith NASA CR-112236: for a long overlap Goland and Reissner's k tends to 1/(1 + 2√2) = 0.2612 (Eq. 49) and Hart-Smith's to 0 (Eq. 50), and the exact peel peak reaches the long-overlap form of Eq. 113; E 70 GPa, ν 0, t 2 mm, G 1 GPa, Ea 3 GPa, η 0.2 mm, b 25 mm, l 2 m, 20 kN"
inputs = { E = "70 GPa", "ν" = "0", t = "2 mm", G = "1 GPa", E_a = "3 GPa", "η" = "0.2 mm", b = "25 mm", l = "2 m", P = "20 kN", S_s = "20 MPa", n_d = "2" }
expect = { k_GR = "0.2612038750", k = "0.0003345518166", "σ_max" = "118024.8089 Pa", "σ_long" = "118024.8089 Pa", "τ_max" = "53.81115038 MPa", r_40 = "134.5278760" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "D1002 coupon with a weaker adhesive: lap-shear strength 1.5 times the average"
source = "As the coupon: τav = 2000 psi against a lap-shear strength of 3000 psi, under the design factor of 2"
inputs = { E = "10 Mpsi", "ν" = "0.33", t = "0.064 in", G = "185185.185185 psi", E_a = "500 ksi", "η" = "0.005 in", b = "1 in", l = "0.5 in", P = "1000 lbf", S_s = "3000 psi", n_d = "2" }
expect = { n_av = "1.5" }
tolerance = "1e-9"
checks = "fail"
+++

## Adhesive single-lap joint (Hart-Smith)

A balanced single-lap joint: two identical adherends bonded over an overlap l and width b, pulled by a load P. The load's offset bends the adherends, which raises the shear at the ends of the overlap and opens the bond there with a peel (transverse tension) stress, the usual start of failure. This is Hart-Smith's elastic analysis (NASA CR-112236, 1973). It corrects Goland and Reissner's (1944) moment at the end of the overlap, whose factor is shown beside it. Volkersen's shear lag, without bending, is shown too; it underestimates a single lap's peak, as Shigley's Fig. 9-28 shows.

Both adherends and the adhesive are elastic. The adhesive's plasticity, which Hart-Smith shows raises the joint's strength, is not modelled. A wide joint is in plane strain; enter ν = 0 for plane stress (a narrow strip).

### Adherends

```calc {description="Modulus of each adherend"}
E = 10 Mpsi_
```

```calc {description="Poisson's ratio of the adherends (0 for plane stress)"}
ν = 0.33_
```

```calc {description="Thickness of each adherend"}
t = 0.064 in_
```

### Adhesive, bond and load

```calc {description="Shear modulus of the adhesive"}
G = 185185.185185 psi_
```

```calc {description="Young's modulus of the adhesive, across the bond line (for the peel stress)"}
E_a = 500 ksi_
```

```calc {description="Bond line thickness"}
η = 0.005 in_
```

```calc {description="Width of the bond"}
b = 1 in_
```

```calc {description="Overlap length, along the load"}
l = 0.5 in_
```

```calc {description="Load on the joint"}
P = 1000 lbf_
```

The lap-shear strength is the apparent (average) shear strength a lap-shear test (ASTM D1002) reports, from the adhesive's data sheet; Shigley Table 9-7 gives typical ranges by adhesive chemistry.

```calc {description="Lap-shear strength of the adhesive"}
S_s = 4500 psi_
```

```calc {description="Design factor on the lap-shear strength"}
n_d = 2_
```

### Bending at the ends of the overlap

```calc
> Average shear on the bond, P/(bl)
τ_av = P / (b l) = ?? psi
> Half the overlap
c = l / 2 = ?? in
> Flexural rigidity of each adherend per unit width, Eq. 11
D = E t^3 / (12 (1 - ν^2)) = ?? lbf·in
> Load parameter times half the overlap, ξ = √(P/(bD)), Eq. 5
ξc = c sqrt(P / (b D)) = ??
> End moment factor, Hart-Smith, Eq. 39
k = 1 / (1 + ξc + ξc^2 / 6) = ??
> End moment factor of Goland and Reissner, Eq. 48, for comparison
k_GR = 1 / (1 + 2 sqrt(2) tanh(ξc / (2 sqrt(2)))) = ??
> Bending moment per unit width at the end of the overlap, Eqs. 39 and 47
M_0 = k P / b × t / 2 (1 + η / t) = ?? lbf·in/in
```

### Shear in the adhesive

```calc
> Shear-lag parameter, Eq. 26
λ = sqrt(2 G / (E t η)) = ?? in^-1
> With the adherends bending, Eq. 27
λ′ = sqrt((1 + 3 (1 - ν^2)) / 4) λ = ?? in^-1
> Load per width the hyperbolic part of the shear carries, Eqs. 29 and 37
q_s = G / (E t η) (P / b + 6 (1 - ν^2) M_0 / t) / (2 λ′^2) = ?? lbf/in
> Peak shear at the ends, with bending, Eq. 28
τ_max = q_s λ′ coth(2 λ′ c) + (P / b - q_s) / (2 c) = ?? psi
> Peak over average shear by Eq. 40, a check on τ_max/τ_av
r_40 = 1 + (1 + 3 k (1 - ν^2) (1 + η / t)) λ^2 / (4 λ′^2) (2 λ′ c / tanh(2 λ′ c) - 1) = ??
> Volkersen's peak without bending, for comparison
τ_V = P / b × λ / 2 coth(λ c) = ?? psi
```

### Peel stress in the adhesive

The peel stress across the bond line is A·cos(χs)·cosh(χs) + B·sin(χs)·sinh(χs) along the overlap (Eq. 99). A and B follow from no net transverse force on the overlap (Eq. 101) and the end moment (Eq. 105), solved exactly here rather than by the long-overlap approximation; Eq. 113's long-overlap peak is shown beside it. The coefficients below are scaled by cosh(χc), so long overlaps don't overflow.

```calc
> Peel parameter, Eq. 98
χ = (6 E_a (1 - ν^2) / (E t^3 η))^0.25 = ?? in^-1
> Times half the overlap
χc = χ c = ??
> The two conditions' coefficients
a_11 = sin(χc) + cos(χc) tanh(χc) = ??
a_12 = sin(χc) - cos(χc) tanh(χc) = ??
a_21 = -2 χ^2 sin(χc) tanh(χc) = ?? in^-2
a_22 = 2 χ^2 cos(χc) = ?? in^-2
> Right-hand side, from the end moment
r_p = E_a M_0 / (η D) = ?? psi/in²
Δ_p = a_11 a_22 - a_12 a_21 = ?? in^-2
A_p = -a_12 r_p / Δ_p = ?? psi
B_p = a_11 r_p / Δ_p = ?? psi
> Peak peel stress, at the ends of the overlap
σ_max = A_p cos(χc) + B_p sin(χc) tanh(χc) = ?? psi
> Long-overlap peak peel stress, Eq. 113
σ_long = E_a M_0 / (2 η D χ^2) = ?? psi
```

### Strength

Lap-shear strengths are average stresses from coupons whose own peaks are like these, so they are compared with the average shear. Keep peel low: a peel stress near the adhesive's tensile strength, or a peak shear far above the average, calls for a longer overlap, thicker adherends, or tapered ends.

```calc {description="Lap-shear strength over the average shear"}
n_av = S_s / τ_av = ??
```

```calc
@test n_av ≥ n_d
```
