+++
cruncha-template = 1
id = "mechanical.hertzContact.elliptical"
version = "1.0.0"
title = "Hertz elliptical contact, general radii (Hamrock)"
summary = "Two elastic bodies with two principal radii of curvature each (crossed cylinders, a ball in a grooved race, a wheel on a rail, any pair of radii): the ellipticity from the exact Hertz solution with complete elliptic integrals, the semi-axes of the contact ellipse, the maximum and mean pressure, the elastic approach and contact stiffness, and Lundberg and Palmgren's orthogonal shear stress with its depth."
category = ["Mechanical engineering", "Bearings"]
tags = ["Hertz", "contact stress", "elliptical contact", "point contact", "ball bearing", "race", "groove", "crossed cylinders", "wheel", "rail", "elliptic integral", "orthogonal shear", "Lundberg-Palmgren", "Hamrock", "Roark"]
from = "lacewing:mechanical.hertzContact"
license = "GPL-3.0-or-later"
inputs = ["E_1", "ν_1", "E_2", "ν_2", "F", "r_1x", "r_1y", "r_2x", "r_2y", "φ"]
outputs = ["E′", "R", "R_d", "k", "ℱ", "ℰ", "a", "b", "δ", "p_max", "p_mean", "K_δ", "k_c", "t_a", "τ_0", "z_0", "x_0"]

[[references]]
work = "Fundamentals of Fluid Film Lubrication, NASA RP-1255 (Hamrock)"
edition = "1991"
locator = "Ch. 19: Eqs. 19-3 to 19-5 (curvature sum and difference), 19-7 (pmax), 19-9 to 19-11 (ellipticity, elliptic integrals), 19-14 to 19-17 (ellipse, approach, E′), 19-23 to 19-28 (orthogonal shear); Table 19-4"

[[references]]
work = "Rolling-Element Bearings, NASA RP-1105 (Hamrock, Anderson)"
edition = "1983"
locator = "Eqs. 3 and 13 to 18 (ball and race radii, conformity), 45 and 46 (contact stiffness); Tables 11 and 13"

[[references]]
work = "Roark's Formulas for Stress and Strain (Young, Budynas)"
edition = "7th"
locator = "Sec. 14.1 and its example (ball in a race); Table 14.1, cases 3 and 4 (crossed cylinders; principal planes at an angle, cos θ)"

[[tests]]
name = "Roark Sec. 14.1 example: 1.5 in ball in a 10 in race with a 0.80 in groove, 2000 lb"
source = "Lacewing Lab hertz_contact_test, after Roark 7th ed. pp. 694-695 (Table 14.1 case 4 with R1 = −0.80 in, R1′ = −5 in, ball 0.75 in; E = 30 Mpsi, ν = 0.3): ellipse semi-axes c = 0.144 in and d = 0.0263 in, (σc)max = 252 000 psi, approach 0.00125 in, from Roark's interpolated coefficients"
inputs = { E_1 = "30 Mpsi", "ν_1" = "0.3", E_2 = "30 Mpsi", "ν_2" = "0.3", F = "2000 lbf", r_1x = "0.75 in", r_1y = "0.75 in", r_2x = "-5 in", r_2y = "-0.80 in" }
expect = { a = "0.144 in", b = "0.0263 in", p_max = "252000 psi", "δ" = "0.00125 in" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "NASA RP-1255 Table 19-4: ball on the outer race, 222.4 N"
source = "Lacewing Lab hertz_contact_test, after Hamrock, NASA RP-1255 Table 19-4 (RP-1105 Table 13): ball 12.7 mm, race rbx = −38.9 mm, groove rby = −6.6 mm, E′ = 2.197 × 10¹¹ Pa (ν = 0): k = 7.3649, ℰ = 1.0267, ℱ = 3.3941, Dy = 1.842 mm, Dx = 0.250 mm, δ = 3.56 µm, pmax = 0.922 GPa, t* = 1.0090, τ0 = 0.229 GPa"
inputs = { E_1 = "2.197e11 Pa", "ν_1" = "0", E_2 = "2.197e11 Pa", "ν_2" = "0", F = "222.4111 N", r_1x = "6.35 mm", r_1y = "6.35 mm", r_2x = "-38.9 mm", r_2y = "-6.6 mm" }
expect = { k = "7.3649", "ℰ" = "1.0267", "ℱ" = "3.3941", a = "0.921 mm", b = "0.125 mm", "δ" = "3.56 µm", p_max = "0.922 GPa", "t_a" = "1.0090", "τ_0" = "0.229 GPa" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "NASA RP-1255 Table 19-4: ball on a plane, 222.4 N (a circle)"
source = "Lacewing Lab hertz_contact_test, after Hamrock, NASA RP-1255 Table 19-4: ball 12.7 mm on a plane, E′ = 2.197 × 10¹¹ Pa: D = 0.426 mm, δ = 7.13 µm, pmax = 2.34 GPa, t* = 1.2808, τ0 = 0.501 GPa"
inputs = { E_1 = "2.197e11 Pa", "ν_1" = "0", E_2 = "2.197e11 Pa", "ν_2" = "0", F = "222.4111 N", r_1x = "6.35 mm", r_1y = "6.35 mm", r_2x = "0 mm", r_2y = "0 mm" }
expect = { k = "1", a = "0.213 mm", "δ" = "7.13 µm", p_max = "2.34 GPa", "t_a" = "1.2808", "τ_0" = "0.501 GPa" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "NASA RP-1255 Table 19-4: wheel on rail, 100 kN (major axis along the rolling direction)"
source = "Lacewing Lab hertz_contact_test, after Hamrock, NASA RP-1255 Table 19-4: wheel rax = 0.5019 m on a rail crown rby = 0.300 m, E′ = 2.197 × 10¹¹ Pa: k = Dy/Dx = 0.7099 (here a/b = 1/0.7099), ℰ = 1.3526, ℱ = 1.8508, Dx = 15.190 mm, Dy = 10.783 mm, δ = 106 µm, pmax = 1.166 GPa"
inputs = { E_1 = "2.197e11 Pa", "ν_1" = "0", E_2 = "2.197e11 Pa", "ν_2" = "0", F = "100 kN", r_1x = "0.5019 m", r_1y = "0 m", r_2x = "0 m", r_2y = "0.300 m" }
expect = { k = "1.40865", "ℰ" = "1.3526", "ℱ" = "1.8508", a = "7.595 mm", b = "5.3915 mm", "δ" = "106 µm", p_max = "1.166 GPa" }
tolerance = "0.3%"
checks = "pass"

[[tests]]
name = "Hand check: Roark's ball in a race exactly"
source = "By hand (RP-1255 Eqs. 19-3 to 19-28, k by bisection with AGM elliptic integrals): E′ = 227.29969 GPa, R = 0.020876712 m, Rd = 0.86301370, k = 5.4564491, ℱ = 3.1008952, ℰ = 1.0438116, a = 0.14357343 in, b = 0.026312612 in, δ = 0.0012512224 in, pmax = 252774.54 psi, t* = 1.0161385, τ0 = 432.16260 MPa"
inputs = { E_1 = "30 Mpsi", "ν_1" = "0.3", E_2 = "30 Mpsi", "ν_2" = "0.3", F = "2000 lbf", r_1x = "0.75 in", r_1y = "0.75 in", r_2x = "-5 in", r_2y = "-0.80 in" }
expect = { "E′" = "227.29969 GPa", R = "0.020876712 m", R_d = "0.86301370", k = "5.4564491", "ℱ" = "3.1008952", "ℰ" = "1.0438116", a = "0.14357343 in", b = "0.026312612 in", "δ" = "0.0012512224 in", p_max = "252774.54 psi", "t_a" = "1.0161385", "τ_0" = "432.16260 MPa" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Hand check: crossed steel cylinders, 20 and 30 mm, 1 kN"
source = "By hand: E 200 GPa, ν 0.3, r1x = 10 mm, r2y = 15 mm (Roark Table 14.1 case 3): Rd = 0.2, k = 1.3101030, a = 0.49932309 mm, b = 0.38113271 mm, δ = 15.573892 µm, pmax = 2508.9009 MPa, tangent stiffness 1.5F/δ = 96.315037 kN/mm"
inputs = { E_1 = "200 GPa", "ν_1" = "0.3", E_2 = "200 GPa", "ν_2" = "0.3", F = "1 kN", r_1x = "10 mm", r_1y = "0 mm", r_2x = "0 mm", r_2y = "15 mm" }
expect = { R_d = "0.2", k = "1.3101030", a = "0.49932309 mm", b = "0.38113271 mm", "δ" = "15.573892 µm", p_max = "2508.9009 MPa", k_c = "96.315037 kN/mm" }
tolerance = "1e-6"
checks = "pass"
+++

## Hertz elliptical contact

Two elastic bodies, each with two principal radii of curvature at the point where they touch, pressed together by a force F, touch on an ellipse. This is the classical Hertz solution as Hamrock gives it (NASA RP-1255 Ch. 19), solved exactly with the complete elliptic integrals rather than their curve fits. It covers crossed cylinders, a ball in the grooved race of a ball bearing, a wheel on a rail, and a sphere on a sphere or flat (where the ellipse is a circle). For spheres and parallel cylinders, the sphere and cylinder templates also give the stresses below the surface. Hertz's theory takes smooth, frictionless, elastic surfaces and a contact small beside the radii.

### Materials

```calc {description="Young's modulus of body 1"}
E_1 = 30 Mpsi_
```

```calc {description="Poisson's ratio of body 1"}
ν_1 = 0.3_
```

```calc {description="Young's modulus of body 2"}
E_2 = 30 Mpsi_
```

```calc {description="Poisson's ratio of body 2"}
ν_2 = 0.3_
```

### Load and principal radii

Each body has a radius in the x plane (the rolling direction) and in the y plane across it. A radius is positive when its centre of curvature is inside the body (convex) and negative when outside (concave: a groove, a bore, an outer race). Enter **0 for a flat direction** (a cylinder along its axis, a plane). φ is the angle between the plane of body 1's x radius and that of body 2's (Roark Table 14.1 case 4); it is 0 in bearings.

- **Crossed cylinders** of diameters d₁ and d₂: r_1x = d₁/2, r_2y = d₂/2, the others 0.
- **Ball on a race** (RP-1105 Eqs. 13 to 18): ball diameter D, pitch diameter d_m, contact angle β and groove conformity f = r/D (usually 0.51 to 0.54): r_1x = r_1y = D/2; inner race r_2x = (d_m − D cos β)/(2 cos β), outer race r_2x = −(d_m + D cos β)/(2 cos β); groove r_2y = −f D.

```calc {description="Normal force pressing the bodies together"}
F = 2000 lbf_
```

```calc {description="Radius of body 1 in the x plane (rolling direction); negative concave, 0 flat"}
r_1x = 0.75 in_
```

```calc {description="Radius of body 1 in the y plane; negative concave, 0 flat"}
r_1y = 0.75 in_
```

```calc {description="Radius of body 2 in the x plane; negative concave, 0 flat"}
r_2x = -5 in_
```

```calc {description="Radius of body 2 in the y plane; negative concave, 0 flat"}
r_2y = -0.80 in_
```

```calc {description="Angle between the principal planes of the two bodies"}
φ = 0°_
```

### Curvatures

```calc
> Curvatures, zero for a flat direction
c_1x = {0 '1/m' if r_1x == 0 'm'; 1 / r_1x otherwise} = ?? 1/mm
c_1y = {0 '1/m' if r_1y == 0 'm'; 1 / r_1y otherwise} = ?? 1/mm
c_2x = {0 '1/m' if r_2x == 0 'm'; 1 / r_2x otherwise} = ?? 1/mm
c_2y = {0 '1/m' if r_2y == 0 'm'; 1 / r_2y otherwise} = ?? 1/mm
> Curvature sum in each plane, Eqs. 19-4 and 19-5
c_x = c_1x + c_2x = ?? 1/mm
c_y = c_1y + c_2y = ?? 1/mm
> Curvature sum, 1/R
R = 1 / (c_x + c_y) = ?? mm
> Curvature difference (Eq. 19-3, or Roark's cos θ with the planes at φ)
R_d = R sqrt((c_1x - c_1y)^2 + (c_2x - c_2y)^2 + 2 (c_1x - c_1y) (c_2x - c_2y) cos(2 φ)) = ??
> Effective modulus, Eq. 19-17
E′ = 2 / ((1 - ν_1^2) / E_1 + (1 - ν_2^2) / E_2) = ?? GPa
```

### Ellipticity

The ellipticity k, the major over the minor diameter of the ellipse, solves k² ℰ (1 − R_d) = 2ℱ − ℰ(1 + R_d) (Eq. 19-9), where ℱ and ℰ are the complete elliptic integrals of the first and second kinds with modulus √(1 − 1/k²) (Eqs. 19-10 and 19-11). The integrals come from the arithmetic-geometric mean, and k from bisection.

```calc
function ellipticKE(m)
    a = 1
    b = roundSig(sqrt(1 - m^2), 16)
    s = m^2 / 2
    p = 0.5
    n = 0
    while n < 12
        c = (a - b) / 2
        a_1 = roundSig((a + b) / 2, 16)
        b = roundSig(sqrt(a b), 16)
        a = a_1
        p = 2 p
        s = roundSig(s + p c^2, 16)
        n = n + 1
    end
    K = π / (2 a)
    return [K; K (1 - s)]
end
```

```calc
function ellipticity(r_d)
    if r_d < 1e-12
        return [1; π / 2; π / 2]
    end
    lo = 1
    hi = 2
    ke = ellipticKE(sqrt(1 - 1 / hi^2))
    while hi^2 ke[2] (1 - r_d) - (2 ke[1] - ke[2] (1 + r_d)) < 0
        hi = 2 hi
        ke = ellipticKE(sqrt(1 - 1 / hi^2))
    end
    n = 0
    while n < 100
        m = roundSig((lo + hi) / 2, 16)
        ke = ellipticKE(sqrt(1 - 1 / m^2))
        if m^2 ke[2] (1 - r_d) - (2 ke[1] - ke[2] (1 + r_d)) < 0
            lo = m
        else
            hi = m
        end
        n = n + 1
    end
    k = (lo + hi) / 2
    ke = ellipticKE(sqrt(1 - 1 / k^2))
    return [k; ke[1]; ke[2]]
end
```

```calc
> Ellipticity and the elliptic integrals, Eqs. 19-9 to 19-11
s_e = ellipticity(R_d) = ??
k = s_e[1] = ??
ℱ = s_e[2] = ??
ℰ = s_e[3] = ??
```

### Contact ellipse, pressure and approach

When the planes coincide (φ = 0), the major axis 2a lies across the rolling direction if 1/R_x > 1/R_y (c_x > c_y, as in a bearing) and along it otherwise (a wheel on a rail).

```calc
> Major semi-axis, Eq. 19-14
a = (6 k^2 ℰ F R / (π E′))^(1/3) = ?? mm
> Minor semi-axis, Eq. 19-15
b = (6 ℰ F R / (π k E′))^(1/3) = ?? mm
> Maximum contact pressure, Eq. 19-7
p_max = 3 F / (2 π a b) = ?? MPa
> Mean contact pressure, 2/3 of the maximum
p_mean = F / (π a b) = ?? MPa
> Elastic approach of the bodies, Eq. 19-16
δ = ℱ (9 / (2 ℰ R) (F / (π k E′))^2)^(1/3) = ?? µm
> Load-deflection constant of F = K δ^1.5, RP-1105 Eq. 45
K_δ = F / δ^1.5 = ?? N/m^1.5
> Tangent contact stiffness at this load
k_c = 1.5 F / δ = ?? N/mm
```

### Orthogonal shear stress

Lundberg and Palmgren based their rolling-fatigue theory on the largest half-amplitude τ_0 of the shear on planes parallel to the surface, which a rolling element reverses as it passes; it lies at depth z_0 and at x_0 either side of the centre in the rolling direction (Eqs. 19-23 to 19-28). The auxiliary t* solves b/a = √((t*² − 1)(2t* − 1)). These hold with the minor axis in the rolling direction, as in a bearing (φ = 0 and c_x ≥ c_y); for other orientations τ_0 is not given by these equations.

```calc
function auxiliaryT(r)
    lo = 1
    hi = 2
    while sqrt((hi^2 - 1) (2 hi - 1)) < r
        hi = 2 hi
    end
    n = 0
    while n < 100
        m = roundSig((lo + hi) / 2, 16)
        if sqrt((m^2 - 1) (2 m - 1)) < r
            lo = m
        else
            hi = m
        end
        n = n + 1
    end
    return (lo + hi) / 2
end
```

```calc
> Auxiliary parameter, Eq. 19-23
t_a = auxiliaryT(1 / k) = ??
> Orthogonal shear stress amplitude, Eq. 19-28
τ_0 = p_max sqrt(2 t_a - 1) / (2 t_a (t_a + 1)) = ?? MPa
> Its depth, Eqs. 19-24 and 19-26
z_0 = b / ((t_a + 1) sqrt(2 t_a - 1)) = ?? mm
> Its distance from the centre in the rolling direction, Eqs. 19-25 and 19-27
x_0 = b t_a / (t_a + 1) sqrt((2 t_a + 1) / (2 t_a - 1)) = ?? mm
```
