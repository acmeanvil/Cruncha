+++
cruncha-template = 1
id = "mechanical.hertzContact.cylinders"
version = "1.0.0"
title = "Hertz contact of parallel cylinders (Shigley)"
summary = "Two parallel elastic cylinders pressed together along a length, or a cylinder on a flat or in a cylindrical seat: the half-width of the rectangular contact, the maximum and mean contact pressure, the principal stresses at a chosen depth below the surface, and the largest shear and von Mises stresses on the load axis with their depths."
category = ["Mechanical engineering", "Bearings"]
tags = ["Hertz", "contact stress", "cylinder", "roller", "line contact", "contact pressure", "subsurface shear", "wheel", "rail", "cam", "Shigley"]
from = "lacewing:mechanical.hertzContact"
license = "GPL-3.0-or-later"
inputs = ["E_1", "ν_1", "E_2", "ν_2", "F", "l", "d_1", "d_2", "body_2", "body_z", "ζ_z"]
outputs = ["Δ", "b", "p_max", "p_mean", "σ_x", "σ_y", "σ_z", "ζ_τ", "τ_max", "z_τ", "ζ_vm", "σ_vm"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-19, Cylindrical Contact: Eqs. 3-73 to 3-77, Fig. 3-39; Eq. 5-12; Problems 3-138 and 3-141"

[[tests]]
name = "Shigley Problem 3-141: 150 mm steel wheel, 40 mm wide, on a cast iron rail, 2 kN"
source = "Lacewing Lab hertz_contact_test, after Shigley 10th Problem 3-141 (answer p. 1068): steel 207 GPa, ν 0.292; gray cast iron 100 GPa, ν 0.211 (Table A-5); at the critical depth 0.786b, σx = −35.0, σy = −22.9, σz = −96.9 MPa, τmax = 37.0 MPa"
inputs = { E_1 = "207 GPa", "ν_1" = "0.292", E_2 = "100 GPa", "ν_2" = "0.211", F = "2 kN", l = "40 mm", d_1 = "150 mm", body_2 = "2", body_z = "1", "ζ_z" = "0.786" }
expect = { "σ_x" = "-35.0 MPa", "σ_y" = "-22.9 MPa", "σ_z" = "-96.9 MPa", "τ_max" = "37.0 MPa", "ζ_τ" = "0.786" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Shigley Problem 3-138: aluminium roller 1.25 in × 2 in in a 6 in radius cast iron ring, 117.4 lbf"
source = "Lacewing Lab hertz_contact_test, after Shigley 10th Problem 3-138 (answer p. 1068): the load that makes τmax = 4000 psi is 117.4 lbf (aluminium 71.7 GPa, ν 0.333; cast iron 100 GPa, ν 0.211; the book reads 0.3·pmax off Fig. 3-39)"
inputs = { E_1 = "71.7 GPa", "ν_1" = "0.333", E_2 = "100 GPa", "ν_2" = "0.211", F = "117.4 lbf", l = "2 in", d_1 = "1.25 in", d_2 = "12 in", body_2 = "3", body_z = "1" }
expect = { "τ_max" = "4000 psi" }
tolerance = "0.6%"
checks = "pass"

[[tests]]
name = "Hand check: Problem 3-141 exactly"
source = "By hand (Eqs. 3-73 to 3-77, scanned and refined for the peaks): Δ = (1 − 0.292²)/207e9 + (1 − 0.211²)/100e9 = 1.3973804e-11 1/Pa, b = √(2 × 2000/(π × 0.04) × Δ × 0.15) = 0.25830215 mm, pmax = 123.23160 MPa, pmean = 96.785876 MPa; at 0.786b σx = −34.970768, σy = −22.877084, σz = −96.885821 MPa; τmax = 37.004369 MPa at 0.786151b; σ′max = 69.093151 MPa at 0.697315b"
inputs = { E_1 = "207 GPa", "ν_1" = "0.292", E_2 = "100 GPa", "ν_2" = "0.211", F = "2 kN", l = "40 mm", d_1 = "150 mm", body_2 = "2", body_z = "1", "ζ_z" = "0.786" }
expect = { "Δ" = "1.3973804e-11 1/Pa", b = "0.25830215 mm", p_max = "123.23160 MPa", p_mean = "96.785876 MPa", "σ_x" = "-34.970768 MPa", "σ_y" = "-22.877084 MPa", "σ_z" = "-96.885821 MPa", "τ_max" = "37.004369 MPa", "ζ_τ" = "0.786151", "σ_vm" = "69.093151 MPa", "ζ_vm" = "0.697315" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Hand check: two steel cylinders, 20 and 40 mm, 20 mm long, 10 kN"
source = "By hand: E 207 GPa, ν 0.3 both, 1/d1 + 1/d2 = 75 1/m: b = 0.19317235 mm, pmax = 1647.8026 MPa; at 0.5b σx = −611.03879, σy = −562.95656, σz = −1473.8394 MPa; τmax = 494.80727 MPa; σ′max = 918.67665 MPa at 0.704292b"
inputs = { E_1 = "207 GPa", "ν_1" = "0.3", E_2 = "207 GPa", "ν_2" = "0.3", F = "10 kN", l = "20 mm", d_1 = "20 mm", d_2 = "40 mm", body_2 = "1", body_z = "1", "ζ_z" = "0.5" }
expect = { b = "0.19317235 mm", p_max = "1647.8026 MPa", "σ_x" = "-611.03879 MPa", "σ_y" = "-562.95656 MPa", "σ_z" = "-1473.8394 MPa", "τ_max" = "494.80727 MPa", "σ_vm" = "918.67665 MPa", "ζ_vm" = "0.704292" }
tolerance = "2e-5"
checks = "pass"
+++

## Hertz contact of parallel cylinders

Two elastic cylinders with parallel axes pressed together by a force F spread evenly along a length l touch on a narrow rectangle of half-width b (Shigley Sec. 3-19). Body 1 is a cylinder of diameter d_1 (a roller, a wheel, a cam follower); body 2 is another cylinder, a flat, or a cylindrical seat (a bore) larger than d_1. The theory takes smooth, frictionless, elastic surfaces and a contact small beside the radii, with the load uniform along l: end effects and crowning are not modelled, and friction from sliding (gears, cams) raises the stresses above these. The approach of a line contact depends on how the bodies are supported away from the contact and is not given here.

### Materials

Young's modulus and Poisson's ratio of each body, e.g. from Shigley Table A-5.

```calc {description="Young's modulus of body 1 (the cylinder)"}
E_1 = 207 GPa_
```

```calc {description="Poisson's ratio of body 1"}
ν_1 = 0.292_
```

```calc {description="Young's modulus of body 2"}
E_2 = 100 GPa_
```

```calc {description="Poisson's ratio of body 2"}
ν_2 = 0.211_
```

### Load and geometry

Enter body 2's kind as a number: 1 a convex cylinder, 2 a flat (d_2 is then not used), 3 a cylindrical seat, d_2 being the bore's diameter entered positive and larger than d_1.

```calc {description="Normal force on the whole contact length"}
F = 2 kN_
```

```calc {description="Length of the line contact (roller length, face width)"}
l = 40 mm_
```

```calc {description="Diameter of body 1, the cylinder"}
d_1 = 150 mm_
```

```calc {description="Diameter of body 2 (cylinder, or seat entered positive; not used for a flat)"}
d_2 = 150 mm_
```

```calc {description="Body 2: 1 cylinder, 2 flat, 3 cylindrical seat"}
body_2 = 2_
```

```calc
> Curvature sum 1/d1 + 1/d2: a flat has d2 = ∞, a seat a negative diameter
s_d = {1 / d_1 + 1 / d_2 if body_2 == 1; 1 / d_1 if body_2 == 2; 1 / d_1 - 1 / d_2 if body_2 == 3} = ?? 1/mm
> Elastic compliance sum
Δ = (1 - ν_1^2) / E_1 + (1 - ν_2^2) / E_2 = ?? 1/GPa
```

### Contact

```calc
> Half-width of the rectangular contact, Eq. 3-73
b = sqrt(2 F / (π l) · Δ / s_d) = ?? mm
> Maximum contact pressure, Eq. 3-74
p_max = 2 F / (π b l) = ?? MPa
> Mean contact pressure, π/4 of the maximum
p_mean = F / (2 b l) = ?? MPa
```

### Stresses below the surface

On the load axis at depth z = ζb the stresses along the axes of the cylinders (σ_x), across the contact (σ_y) and normal to it (σ_z) are principal (Shigley Eqs. 3-75 to 3-77; σ_x is a plane-strain stress and depends on Poisson's ratio). The largest shear is (σ_x − σ_z)/2 near the surface and (σ_y − σ_z)/2 deeper; its greatest value, about 0.30 p_max at 0.786b (Fig. 3-39), is often taken as where surface fatigue starts. The largest von Mises stress (Eq. 5-12) is the one to compare with yield. The stresses are in the body whose Poisson's ratio is used.

```calc {description="Body whose stresses are found below the surface (1 or 2)"}
body_z = 1_
```

```calc {description="Depth at which to give the principal stresses, as a fraction of b"}
ζ_z = 0.786_
```

```calc
function lineStresses(ζ, ν)
    s = sqrt(1 + ζ^2)
    return [-2 ν (s - ζ); -((1 + 2 ζ^2) / s - 2 ζ); -1 / s]
end
```

```calc
function lineStress(ζ, ν, kind)
    σ = lineStresses(ζ, ν)
    if kind == 1
        return max(abs(σ[1] - σ[3]), abs(σ[2] - σ[3]), abs(σ[1] - σ[2])) / 2
    end
    return sqrt(((σ[1] - σ[2])^2 + (σ[2] - σ[3])^2 + (σ[3] - σ[1])^2) / 2)
end
```

```calc
function peakDepth(ν, kind)
    best = 0
    ζ_b = 0
    k = 0
    while k ≤ 600
        g = lineStress(3 k / 600, ν, kind)
        if g > best
            best = g
            ζ_b = 3 k / 600
        end
        k = k + 1
    end
    lo = max(ζ_b - 0.005, 0)
    hi = ζ_b + 0.005
    k = 0
    while k < 60
        m_1 = roundSig(hi - 0.6180339887 (hi - lo), 15)
        m_2 = roundSig(lo + 0.6180339887 (hi - lo), 15)
        if lineStress(m_1, ν, kind) > lineStress(m_2, ν, kind)
            hi = m_2
        else
            lo = m_1
        end
        k = k + 1
    end
    return (lo + hi) / 2
end
```

```calc
> Poisson's ratio of the stressed body
ν_z = {ν_1 if body_z == 1; ν_2 if body_z == 2} = ??
> Principal stresses at z = ζ_z b, as fractions of p_max, Eqs. 3-75 to 3-77
s_z = lineStresses(ζ_z, ν_z) = ??
> Along the cylinders' axes, Eq. 3-75
σ_x = p_max s_z[1] = ?? MPa
> Across the contact, Eq. 3-76
σ_y = p_max s_z[2] = ?? MPa
> Normal to the surface, Eq. 3-77
σ_z = p_max s_z[3] = ?? MPa
```

```calc
> Depth of the largest shear, as a fraction of b
ζ_τ = peakDepth(ν_z, 1) = ??
> Largest shear stress on the axis
τ_max = p_max lineStress(ζ_τ, ν_z, 1) = ?? MPa
> Its depth below the surface
z_τ = ζ_τ b = ?? mm
> Depth of the largest von Mises stress, as a fraction of b
ζ_vm = peakDepth(ν_z, 2) = ??
> Largest von Mises stress on the axis, Eq. 5-12
σ_vm = p_max lineStress(ζ_vm, ν_z, 2) = ?? MPa
```
