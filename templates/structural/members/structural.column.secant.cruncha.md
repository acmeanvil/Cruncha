+++
cruncha-template = 1
id = "structural.column.secant"
version = "1.0.0"
title = "Eccentrically loaded column: secant formula and short strut"
summary = "Largest compressive stress of a pin-ended column whose load acts at an eccentricity, by the secant formula; the load at which it first reaches yield and the margin on the applied load; and the short-strut stress with its slenderness limit."
category = ["Structural engineering", "Members"]
tags = ["column", "eccentric load", "secant formula", "short strut", "first yield", "buckling", "beam-column", "Shigley", "Juvinall", "stability"]
from = "lacewing:structural.columnEccentric"
license = "GPL-3.0-or-later"
inputs = ["P", "e_P", "A", "I", "c", "I_o", "L_e", "E", "S_y"]
outputs = ["k", "ε", "σ_max", "P_y", "n_y", "σ_strut", "λ_2", "P_(E,o)"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 4-11 to 4-15: Eqs. 4-49 and 4-50 (secant formula), 4-55 and 4-56 (short strut); Example 4-20, pp. 202-203"

[[references]]
work = "Juvinall and Marshek, Fundamentals of Machine Component Design"
edition = "5th"
locator = "Sec. 5.13, Eq. 5.14, pp. 234-236 (secant formula; assumed eccentricity Le/400 or ec/r² = 0.025 for nominally central loads)"

[[tests]]
name = "W8×31, 4 m pinned, 200 kN at 10 mm about the strong axis"
source = "Lacewing Lab's opening case (help, structural.columnEccentric), hand-checked by script: A 9.13 in², Ix 110 in⁴, c = Ix/Sx = 4.00 in, Iy 37.1 in⁴ (AISC Manual Table 1-1), E 200 GPa, Sy 345 MPa: k = 88.165 mm, ec/k² = 0.13071, σmax = 38.593 MPa, load to first yield 1693.0 kN (n 8.465), weak-axis Euler 1905.1 kN"
inputs = { P = "200 kN", e_P = "10 mm", A = "9.13 in^2", I = "110 in^4", c = "4.00 in", I_o = "37.1 in^4", L_e = "4 m", E = "200 GPa", S_y = "345 MPa" }
expect = { k = "88.1647 mm", "ε" = "0.130709", "σ_max" = "38.5933 MPa", P_y = "1693.00 kN", n_y = "8.46499", "P_(E,o)" = "1905.10 kN", "σ_strut" = "38.3922 MPa" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 4-20: 1 in. square steel block, 4 in. long, 1000 lbf at 0.10 in."
source = "Shigley 10th ed. Example 4-20, pp. 202-203 (checked against the PDF): l/k = 13.9, (l/k)₂ = 0.282 √(AE/P) = 48.8, so the strut formula applies; σc = (P/A)(1 + ec/k²) = 1000 × (1 + 0.1 × 0.5/0.0833) = 1600 psi. (Lacewing Lab's columns_test quotes this example as 2000 lbf and 3200 psi: 2000 lbf is the bolt tension, the block carries 1000 lbf.)"
inputs = { P = "1000 lbf", e_P = "0.1 in", A = "1 in^2", I = "0.0833333333 in^4", c = "0.5 in", I_o = "0.0833333333 in^4", L_e = "4 in", E = "30000000 psi", S_y = "36000 psi" }
expect = { "σ_strut" = "1600 psi", "λ_2" = "48.8", "λ" = "13.86" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Slender round bar near its Euler load: the stress passes yield"
source = "By hand: d 30 mm, A 706.858 mm², I 39 760.78 mm⁴, k 7.5 mm, L 1.2 m, E 200 GPa; PE = π² × 200e3 × 39 760.78 / 1200² = 54.503 kN; at P 45 kN, arg = (1200/15) √(45 000/(706.858 × 200 000)) = 1.42730, sec = 6.99276; ec/k² = 2 × 15/56.25 = 0.53333; σ = 63.662 × (1 + 0.53333 × 6.99276) = 301.09 MPa > 250 MPa"
inputs = { P = "45 kN", e_P = "2 mm", A = "706.858347 mm^2", I = "39760.7820 mm^4", c = "15 mm", I_o = "39760.7820 mm^4", L_e = "1.2 m", E = "200 GPa", S_y = "250 MPa" }
expect = { "ε" = "0.533333", "σ_max" = "301.09 MPa", "P_(E,o)" = "54.503 kN" }
tolerance = "0.1%"
checks = "fail"
+++

## Eccentrically loaded column: secant formula

A straight, pin-ended column of uniform section carries a compressive load P whose line of action is offset by e_P from the centroid, in one principal plane. Bending deflection amplifies the end moment P e_P, and the secant formula gives the largest compressive stress at mid-length (Shigley Eq. 4-49):

σ_max = (P/A) [1 + (e_P c/k²) sec((L_e/2k) √(P/(A E)))]

The load to first yield is the P at which σ_max reaches S_y, found here by bisection below the Euler load (Eq. 4-50). It is a limit, not a working load: apply your own factor of safety, and note that because the formula is not linear in P, a margin on the load is not the same as a margin on the stress.

The formula works in the plane of the eccentricity only. A column bent about its strong axis can still buckle about the weak one, so the template also gives the Euler load about the other axis (I_o). For a nominally central load, Juvinall suggests an assumed eccentricity of L_e/400, or an eccentricity ratio e_P c/k² of 0.025.

### Load and section

```calc {description="Compressive load"}
P = 200 kN_
```

```calc {description="Eccentricity of the load from the centroid, in the plane of bending"}
e_P = 10 mm_
```

```calc {description="Cross-section area"}
A = 9.13 in²_
```

```calc {description="Second moment of area about the bending axis"}
I = 110 in⁴_
```

```calc {description="Distance from the bending axis to the most compressed fibre"}
c = 4.00 in_
```

```calc {description="Second moment of area about the other principal axis"}
I_o = 37.1 in⁴_
```

```calc {description="Effective length, K L"}
L_e = 4 m_
```

```calc {description="Modulus of elasticity"}
E = 200 GPa_
```

```calc {description="Yield strength"}
S_y = 345 MPa_
```

### Secant formula

```calc
> Radius of gyration in the plane of bending
k = sqrt(I / A) = ?? mm
> Eccentricity ratio
ε = e_P c / k^2 = ??
> Euler load in the plane of bending
P_E = π^2 E I / L_e^2 = ?? kN
```

```calc {description="Largest compressive stress, Eq. 4-49"}
σ_max = P / A (1 + ε sec(L_e / (2 k) sqrt(P / (A E)))) = ?? MPa
```

The load to first yield solves Eq. 4-50 for P by bisection between zero and the lesser of the Euler load and the squash load S_y A.

```calc
function secantYieldLoad(S_y, A, k, ε, L_e, E, P_E)
    lo = 0 P_E
    hi = 0.999999999 min(P_E, S_y A)
    for i in 1:60
        m = (lo + hi) / 2
        if m / A (1 + ε sec(L_e / (2 k) sqrt(m / (A E)))) < S_y
            lo = m
        else
            hi = m
        end
    end
    return (lo + hi) / 2
end
```

```calc
> Load at which σ_max reaches S_y, Eq. 4-50
P_y = secantYieldLoad(S_y, A, k, ε, L_e, E, P_E) = ?? kN
> Margin on the applied load
n_y = P_y / P = ??
```

### Short strut

Without the deflection, the stress is (P/A)(1 + e_P c/k²) (Eq. 4-55). That is close enough only for a strut shorter than the slenderness (l/k)₂ (Eq. 4-56).

```calc
> Short-strut stress, Eq. 4-55
σ_strut = P / A (1 + ε) = ?? MPa
> Slenderness below which the strut formula applies, Eq. 4-56
λ_2 = 0.282 sqrt(A E / P) = ??
> Slenderness of this column
λ = L_e / k = ??
```

### Checks

```calc {description="Euler load about the other principal axis"}
P_(E,o) = π^2 E I_o / L_e^2 = ?? kN
```

```calc
@test σ_max ≤ S_y
```

```calc
@test P < P_(E,o)
```
