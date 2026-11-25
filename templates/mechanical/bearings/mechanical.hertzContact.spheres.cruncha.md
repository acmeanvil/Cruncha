+++
cruncha-template = 1
id = "mechanical.hertzContact.spheres"
version = "1.0.0"
title = "Hertz contact of spheres (Shigley)"
summary = "Two elastic spheres pressed together, or a sphere on a flat or in a spherical seat: the radius of the circular contact, the maximum and mean contact pressure, the elastic approach and contact stiffness, and the largest shear and von Mises stresses below the surface on the load axis with their depth."
category = ["Mechanical engineering", "Bearings"]
tags = ["Hertz", "contact stress", "sphere", "ball", "point contact", "contact pressure", "approach", "subsurface shear", "seat", "Shigley", "Roark"]
from = "lacewing:mechanical.hertzContact"
license = "GPL-3.0-or-later"
inputs = ["E_1", "ν_1", "E_2", "ν_2", "F", "d_1", "d_2", "body_2", "body_z"]
outputs = ["Δ", "a", "p_max", "p_mean", "δ", "k_c", "ζ_τ", "τ_max", "z_τ", "σ_vm"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Sec. 3-19, Spherical Contact: Eqs. 3-68 to 3-72, Fig. 3-37; Eq. 5-12; Problem 3-133"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 14.1; Table 14.1, case 1 (approach of two spheres)"

[[references]]
work = "Rolling-Element Bearings, NASA RP-1105 (Hamrock, Anderson)"
edition = "1983"
locator = "Eqs. 45 and 46 (point-contact stiffness F = K·δ^1.5); Table 13 (ball on plane)"

[[tests]]
name = "Shigley Problem 3-133: two 30 mm carbon steel balls, 1 N"
source = "Lacewing Lab hertz_contact_test, after Shigley 10th Problem 3-133 (answer p. 1068): σmax = 352·F^⅓ MPa with F in newtons (E = 207 GPa, ν = 0.292, Table A-5); Eqs. 3-70 to 3-72 put τmax at 0.312·pmax at 0.48a (the answer's 106 MPa takes Fig. 3-37's 0.3·pmax)"
inputs = { E_1 = "207 GPa", "ν_1" = "0.292", E_2 = "207 GPa", "ν_2" = "0.292", F = "1 N", d_1 = "30 mm", d_2 = "30 mm", body_2 = "1" }
expect = { p_max = "352 MPa", "ζ_τ" = "0.48" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "NASA RP-1105 Table 13: 12.7 mm ball on a plane, 222.4 N, E′ = 2.197 × 10¹¹ Pa"
source = "Lacewing Lab hertz_contact_test, after Hamrock and Anderson, NASA RP-1105 Table 13 (and RP-1255 Table 19-4), ball on plane: contact diameter 0.000426 m, approach 7.13 µm, pmax 2.34 GPa (ν = 0 makes E′ = E)"
inputs = { E_1 = "2.197e11 Pa", "ν_1" = "0", E_2 = "2.197e11 Pa", "ν_2" = "0", F = "222.4111 N", d_1 = "12.7 mm", d_2 = "1 m", body_2 = "2" }
expect = { a = "0.000213 m", "δ" = "7.13e-6 m", p_max = "2.34 GPa" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Problem 3-133 exactly"
source = "By hand (Eqs. 3-68 to 3-72, scanned and refined for the peak): Δ = 2(1 − 0.292²)/207e9 = 8.838029e-12 1/Pa, a = ∛(3/8 × Δ/(2/0.03)) = 3.6769917e-5 m, pmax = 3/(2πa²) = 353.14746 MPa, pmean = 235.43164 MPa, δ = 1.040∛(Δ² × 66.667) = 1.8026297e-7 m, k = 1.5/δ = 8.3211763e6 N/m, τmax = 110.13439 MPa at z/a = 0.478212, σ′max = 220.26878 MPa"
inputs = { E_1 = "207 GPa", "ν_1" = "0.292", E_2 = "207 GPa", "ν_2" = "0.292", F = "1 N", d_1 = "30 mm", d_2 = "30 mm", body_2 = "1" }
expect = { "Δ" = "8.838029e-12 1/Pa", a = "3.6769917e-5 m", p_max = "353.14746 MPa", p_mean = "235.43164 MPa", "δ" = "1.8026297e-7 m", k_c = "8.3211763e6 N/m", "τ_max" = "110.13439 MPa", "ζ_τ" = "0.478212", "σ_vm" = "220.26878 MPa" }
tolerance = "2e-5"
checks = "pass"

[[tests]]
name = "Hand check: a 25 mm steel ball in a 40 mm spherical seat, 1 kN"
source = "By hand: E = 200 GPa, ν = 0.3 both, 1/d1 − 1/d2 = 1/0.025 − 1/0.04 = 15 1/m: a = 0.61046457 mm, pmax = 1281.2104 MPa, δ = 11.179559 µm, τmax = 397.20149 MPa at z/a = 0.480864"
inputs = { E_1 = "200 GPa", "ν_1" = "0.3", E_2 = "200 GPa", "ν_2" = "0.3", F = "1 kN", d_1 = "25 mm", d_2 = "40 mm", body_2 = "3" }
expect = { a = "0.61046457 mm", p_max = "1281.2104 MPa", "δ" = "11.179559 µm", "τ_max" = "397.20149 MPa", "ζ_τ" = "0.480864" }
tolerance = "2e-5"
checks = "pass"

+++

## Hertz contact of spheres

Two elastic spheres pressed together by a force F touch on a small circle (Shigley Sec. 3-19). Body 1 is a sphere of diameter d_1; body 2 is another sphere, a flat, or a spherical seat (a concave socket) of diameter d_2 larger than d_1. Hertz's theory takes the surfaces as smooth, frictionless and elastic, with the contact small beside the radii; friction from sliding or rolling raises the stresses above these. For two cylinders use the cylinder template; for bodies with two different curvatures each (crossed cylinders, a ball in a grooved race) the elliptical contact template.

### Materials

Young's modulus and Poisson's ratio of each body, e.g. from Shigley Table A-5 (carbon steel 207 GPa and 0.292).

```calc {description="Young's modulus of body 1 (the sphere)"}
E_1 = 207 GPa_
```

```calc {description="Poisson's ratio of body 1"}
ν_1 = 0.292_
```

```calc {description="Young's modulus of body 2"}
E_2 = 207 GPa_
```

```calc {description="Poisson's ratio of body 2"}
ν_2 = 0.292_
```

### Load and geometry

Enter body 2's kind as a number: 1 a convex sphere, 2 a flat (d_2 is then not used), 3 a spherical seat, d_2 being the seat's diameter entered positive and larger than d_1 (a smaller seat makes no contact, and the contact size cannot be computed).

```calc {description="Normal force pressing the bodies together"}
F = 1000 N_
```

```calc {description="Diameter of body 1, the sphere"}
d_1 = 30 mm_
```

```calc {description="Diameter of body 2 (sphere, or seat entered positive; not used for a flat)"}
d_2 = 30 mm_
```

```calc {description="Body 2: 1 sphere, 2 flat, 3 spherical seat"}
body_2 = 1_
```

```calc
> Curvature sum 1/d1 + 1/d2: a flat has d2 = ∞, a seat a negative diameter
s_d = {1 / d_1 + 1 / d_2 if body_2 == 1; 1 / d_1 if body_2 == 2; 1 / d_1 - 1 / d_2 if body_2 == 3} = ?? 1/mm
> Elastic compliance sum
Δ = (1 - ν_1^2) / E_1 + (1 - ν_2^2) / E_2 = ?? 1/GPa
```

### Contact

```calc
> Radius of the circular contact, Eq. 3-68
a = (3 F / 8 · Δ / s_d)^(1/3) = ?? mm
> Maximum contact pressure, Eq. 3-69
p_max = 3 F / (2 π a^2) = ?? MPa
> Mean contact pressure, 2/3 of the maximum
p_mean = F / (π a^2) = ?? MPa
```

The approach δ is how much closer two points far from the contact come (Roark Table 14.1 case 1, with K_D = 1/s_d and C_E = Δ). The contact stiffens as the load grows, F ∝ δ^1.5 (RP-1105 Eq. 45), so its tangent stiffness at this load is 1.5F/δ.

```calc
> Approach of the two bodies, Roark Table 14.1 case 1
δ = 1.040 (F^2 Δ^2 s_d)^(1/3) = ?? µm
> Tangent contact stiffness at this load
k_c = 1.5 F / δ = ?? N/mm
```

### Stresses below the surface

On the load axis at depth z the radial and tangential stresses are equal and, with σ_z, principal (Shigley Eqs. 3-70 and 3-71). The largest shear (σ_x − σ_z)/2 lies below the surface, about 0.3 p_max near 0.48a for ν = 0.3 (Fig. 3-37), and is often taken as where surface fatigue starts; the von Mises stress, twice that shear on the axis, is the one to compare with yield. The stresses are in the body whose Poisson's ratio is used (Shigley p. 138).

```calc {description="Body whose stresses are found below the surface (1 or 2)"}
body_z = 1_
```

```calc
function sphereShear(ζ, ν)
    r = max(ζ, 1e-12)
    σ_x = -((1 - r atan(1 / r)) (1 + ν) - 1 / (2 (1 + r^2)))
    σ_z = -1 / (1 + r^2)
    return abs(σ_x - σ_z) / 2
end
```

```calc
function peakDepth(ν)
    best = 0
    ζ_b = 0
    k = 0
    while k ≤ 600
        g = sphereShear(3 k / 600, ν)
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
        if sphereShear(m_1, ν) > sphereShear(m_2, ν)
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
> Depth of the largest shear, as a fraction of a
ζ_τ = peakDepth(ν_z) = ??
> Largest shear stress on the axis, Eq. 3-72
τ_max = p_max sphereShear(ζ_τ, ν_z) = ?? MPa
> Its depth below the surface
z_τ = ζ_τ a = ?? mm
> Largest von Mises stress on the axis, Eq. 5-12 with σ_x = σ_y
σ_vm = 2 τ_max = ?? MPa
```
