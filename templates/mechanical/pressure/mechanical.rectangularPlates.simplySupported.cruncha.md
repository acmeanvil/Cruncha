+++
cruncha-template = 1
id = "mechanical.rectangularPlates.simplySupported"
version = "1.0.0"
title = "Simply supported rectangular plate, uniform pressure (Navier)"
summary = "Centre deflection, bending moments and bending stresses of a flat rectangular plate simply supported on all four edges under uniform pressure, by Navier's double series summed in a function for any side ratio and Poisson's ratio, with the factor of safety on yield and the small-deflection limit checked."
category = ["Mechanical engineering", "Pressure vessels"]
tags = ["plate", "flat plate", "rectangular plate", "simply supported", "uniform pressure", "Navier", "plate bending", "tank wall", "cover plate", "Timoshenko", "Den Hartog"]
from = "lacewing:mechanical.rectangularPlates"
license = "GPL-3.0-or-later"
inputs = ["a", "b", "t", "p_0", "E", "ν", "S_y", "n_d"]
outputs = ["D", "α", "β_x", "β_y", "w", "M_x", "M_y", "σ", "n_y"]

[[references]]
work = "Timoshenko & Woinowsky-Krieger, Theory of Plates and Shells"
edition = "2nd"
locator = "Sec. 28 (Navier solution for a simply supported plate), Table 8 (deflection and moments at the centre, uniform load, ν = 0.3)"

[[references]]
work = "Den Hartog, Advanced Strength of Materials"
edition = "1952"
locator = "§19 Catalogue of Results, Case 19 (simply supported, uniform pressure); Eq. 68 (σ = 6M/t²); §20 (limit of small deflections)"

[[tests]]
name = "Square plate: Timoshenko Table 8, b/a = 1"
source = "Timoshenko & Woinowsky-Krieger 2nd ed. Table 8 (ν = 0.3), b/a = 1: w = 0.00406 qa⁴/D, Mx = My = 0.0479 qa²; Den Hartog Case 19 gives 0.044 and 0.048 in his form (Lacewing Lab rectangular_plates_test sums the same series and agrees with his table)"
inputs = { a = "1 m", b = "1 m", t = "10 mm", p_0 = "10 kPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "α" = "0.00406", "β_x" = "0.0479", "β_y" = "0.0479" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Timoshenko Table 8, b/a = 2"
source = "Timoshenko & Woinowsky-Krieger 2nd ed. Table 8 (ν = 0.3), b/a = 2: w = 0.01013 qa⁴/D, Mx = 0.1017 qa², My = 0.0464 qa²"
inputs = { a = "0.5 m", b = "1 m", t = "10 mm", p_0 = "10 kPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { "α" = "0.01013", "β_x" = "0.1017", "β_y" = "0.0464" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: 500 × 1000 × 10 mm steel at 1 bar, the series to 99 × 99 terms"
source = "By hand (Python, the same 2500 odd terms): D = 200e9 × 0.01³/(12 × 0.91) = 18 315.018 N·m, α = 0.010128663, βx = 0.10168294, βy = 0.046350019, w = α p a⁴/D = 3.4564063 mm, Mx = 2542.0736 N·m/m, My = 1158.7505 N·m/m, σ = 6Mx/t² = 152.52442 MPa, n = 1.6390818. The deflection is a third of t, under the t/2 small-deflection limit"
inputs = { a = "500 mm", b = "1000 mm", t = "10 mm", p_0 = "100 kPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { D = "18315.018 N·m", "α" = "0.010128663", "β_x" = "0.10168294", "β_y" = "0.046350019", w = "3.4564063 mm", M_x = "2542.0736 N", M_y = "1158.7505 N", "σ" = "152.52442 MPa", n_y = "1.6390818" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "The same plate at 2 bar: too much deflection and too little margin"
source = "By hand: w = 6.9128 mm, more than t/2, and n = 0.81954"
inputs = { a = "500 mm", b = "1000 mm", t = "10 mm", p_0 = "200 kPa", E = "200 GPa", "ν" = "0.3", S_y = "250 MPa", n_d = "1.5" }
expect = { w = "6.9128125 mm", n_y = "0.81954092" }
tolerance = "1e-6"
checks = "fail"
+++

## Simply supported rectangular plate under uniform pressure

The largest deflection and bending stresses of a flat rectangular plate a × b, thickness t, simply supported on all four edges (free to rotate, corners held down) under a uniform pressure p₀. Navier's double sine series is the exact small-deflection solution for any side ratio; its terms fall off quickly, and 50 odd terms each way (2500 in all) give the centre values to better than 0.01 % up to b/a = 10. The coefficients are those of Timoshenko's Table 8 (w = α p₀a⁴/D, M = β p₀a²) for any Poisson's ratio.

The plate must be thin (t much less than a) and the deflection well under its thickness: as w approaches t/2 the plate starts to carry load as a membrane too, and these results overstate both the deflection and the stress (Den Hartog §20). Corners and joints of a real vessel are not included. For clamped or mixed edges, or other loads, use the tabulated-coefficient template.

### Plate

```calc {description="Span a, along x (the shorter side gives the largest moment Mx)"}
a = 500 mm_
```

```calc {description="Span b, along y"}
b = 1000 mm_
```

```calc {description="Plate thickness"}
t = 10 mm_
```

```calc {description="Uniform pressure"}
p_0 = 100 kPa_
```

### Material

```calc {description="Young's modulus"}
E = 200 GPa_
```

```calc {description="Poisson's ratio"}
ν = 0.3_
```

```calc {description="Yield strength"}
S_y = 250 MPa_
```

```calc {description="Design factor the factor of safety on yield must reach"}
n_d = 1.5_
```

### Navier's series

With m and n odd, the sign of each term is sin(mπ/2) sin(nπ/2), and r = a/b. The function returns α, βx and βy. Each term is rounded to 16 significant digits before it is added, which keeps the engine's exact fractions short without changing the sum.

```calc
function navierUniform(r, nu)
    w = 0
    mx = 0
    my = 0
    for i in 1:50
        m = 2 i - 1
        for j in 1:50
            n = 2 j - 1
            s = (-1)^((m + n) / 2 - 1)
            d = m n (m^2 + n^2 r^2)^2
            w = w + roundSig(s / d, 16)
            mx = mx + roundSig(s (m^2 + nu n^2 r^2) / d, 16)
            my = my + roundSig(s (nu m^2 + n^2 r^2) / d, 16)
        end
    end
    return [16 w / π^6, 16 mx / π^4, 16 my / π^4]
end
```

```calc
> Flexural rigidity
D = E t^3 / (12 (1 - ν^2)) = ?? N·m
> Navier coefficients at the centre
c_N = navierUniform(a / b, ν) = !
> Deflection coefficient, w = α p₀a⁴/D
α = c_N[1] = ??
> Moment coefficient across x, Mx = βx p₀a²
β_x = c_N[2] = ??
> Moment coefficient across y, My = βy p₀a²
β_y = c_N[3] = ??
```

### Deflection and stress at the centre

```calc
> Largest deflection, at the centre
w = α p_0 a^4 / D = ?? mm
> Bending moments per unit length at the centre
M_x = β_x p_0 a^2 = ?? N·m/m
M_y = β_y p_0 a^2 = ?? N·m/m
> Largest bending stress, Den Hartog Eq. 68
σ = 6 max(M_x, M_y) / t^2 = ?? MPa
> Factor of safety on yield
n_y = S_y / σ = ??
```

### Checks

Small deflections (w under half the thickness), and the factor of safety.

```calc
@test w ≤ t / 2
```

```calc
@test n_y ≥ n_d
```
