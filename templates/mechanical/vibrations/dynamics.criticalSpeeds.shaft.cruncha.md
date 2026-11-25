+++
cruncha-template = 1
id = "dynamics.criticalSpeeds.shaft"
version = "1.0.0"
title = "Critical speeds of a shaft carrying disks (Holzer)"
summary = "The first three flexural (whirling) critical speeds of a uniform solid round shaft on two simple bearings at its ends, carrying disks as point masses, with the shaft's own mass lumped at 20 stations, by Holzer's method for beams: the running speed as a fraction of the first critical, the whirl radius of a disk with a given eccentricity, and the separation from the nearest critical checked."
category = ["Mechanical engineering", "Vibrations"]
tags = ["critical speed", "whirling", "shaft", "rotor", "rotordynamics", "Holzer", "Myklestad", "disk", "eccentricity", "whirl radius", "resonance", "Den Hartog"]
from = "lacewing:dynamics.criticalSpeeds"
license = "GPL-3.0-or-later"
inputs = ["L", "d", "E", "ρ", "x", "m_d", "ω_max", "n", "e", "s_min"]
outputs = ["I", "m_sh", "ω_c", "ω_c1", "n_c1", "ω_c2", "q", "r_w", "s"]

[[references]]
work = "Den Hartog, Mechanical Vibrations"
edition = "4th (1956; Dover reprint 1985)"
locator = "§46 and §46a (critical speeds; Holzer's method for flexural vibration of a shaft with several disks), Eq. 149b (whirl radius of an eccentric disk); §51 and Fig. 172 (gyroscopic and bearing effects, left out)"

[[tests]]
name = "Lacewing Lab: one disk at mid-span on a massless shaft"
source = "Lacewing Lab vibration_test: a single mass at mid-span of a massless simply supported shaft whirls at √(48EI/(m l³)). Here 10 kg on a 1 m, 40 mm steel shaft: EI = 200e9 × π 0.04⁴/64 = 25132.741 N·m², ωc = √(48 × 25132.741/10) = 347.32860 rad/s"
inputs = { L = "1 m", d = "40 mm", E = "200 GPa", "ρ" = "0 kg/m³", x = "[500] mm", m_d = "[10] kg", "ω_max" = "20000 rad/s", n = "1500 rpm" }
expect = { "ω_c1" = "347.32860 rad/s" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Lacewing Lab: two equal disks at the thirds, massless shaft"
source = "Lacewing Lab vibration_test: the 2 × 2 flexibility solution, a11 = a22 = 4l³/(243EI), a12 = 7l³/(486EI), ω = 1/√(m(a11 ± a12)). Here 10 kg each at 0.3 and 0.6 m of a 0.9 m, 40 mm steel shaft: ωc1 = 334.21710, ωc2 = 1294.4173 rad/s; running at 3000 rpm (314.16 rad/s) is 6 % below the first"
inputs = { L = "0.9 m", d = "40 mm", E = "200 GPa", "ρ" = "0 kg/m³", x = "[300; 600] mm", m_d = "[10; 10] kg", "ω_max" = "20000 rad/s", n = "3000 rpm" }
expect = { "ω_c1" = "334.21710 rad/s", "ω_c2" = "1294.4173 rad/s", s = "0.0600144" }
tolerance = "1e-6"
checks = "fail"

[[tests]]
name = "Hand check: the uniform shaft alone against the exact π²√(EI/(μl⁴))"
source = "By hand: a uniform simply supported 1 m, 40 mm steel shaft (μ = 7850 × π 0.04²/4 = 9.8646 kg/m) whirls at (kπ)² √(EI/μ) = 498.17269, 1992.6908 rad/s exactly; the 20 lumps give 498.17248 and 1992.6770 (a Python run of Lacewing Lab's method), within 0.001 %. A zero-mass disk stands in for none"
inputs = { L = "1 m", d = "40 mm", E = "200 GPa", "ρ" = "7850 kg/m³", x = "[500] mm", m_d = "[0] kg", "ω_max" = "20000 rad/s", n = "1500 rpm" }
expect = { "ω_c1" = "498.17269 rad/s", "ω_c2" = "1992.6908 rad/s" }
tolerance = "0.01%"
checks = "pass"

[[tests]]
name = "Lacewing Lab page's opening case: 20 and 10 kg disks on a 1 m, 40 mm steel shaft at 3000 rpm"
source = "A Python run of Lacewing Lab's method (shaft mass in 20 lumps): ωc = 203.09560, 939.22415 rad/s (1939.4201, 8968.9300 rpm); 3000 rpm is 1.5469 times the first critical, a flexible shaft; with e = 0.02 mm the whirl radius is −0.03436 mm (the heavy side inside)"
inputs = { L = "1 m", d = "40 mm", E = "200 GPa", "ρ" = "7850 kg/m³", x = "[400; 700] mm", m_d = "[20; 10] kg", "ω_max" = "20000 rad/s", n = "3000 rpm", e = "0.02 mm", s_min = "0.2" }
expect = { "ω_c1" = "203.09560 rad/s", n_c1 = "1939.4201 rpm", "ω_c2" = "939.22415 rad/s", q = "1.5468541", r_w = "-0.0343600 mm", s = "0.5468541" }
tolerance = "1e-6"
checks = "pass"
+++

## Critical speeds of a shaft carrying disks

The speeds at which a shaft whirls in resonance: its bending natural frequencies, with the disks it carries (gears, pulleys, impellers, rotors) as concentrated masses. The shaft is uniform, solid and round, and runs in two bearings at its ends that are rigid radially and let it rotate freely in bending (simple supports). The disks' rotary inertia and gyroscopic stiffening, flexible bearings and overhangs are left out (Den Hartog §51); they can move a critical speed appreciably. After Den Hartog, Mechanical Vibrations, §46a.

Holzer's method for beams: the shaft becomes weightless spans between the stations, and shear, moment, slope and deflection are carried from one bearing to the other. At a critical speed the deflection and the moment at the far bearing can both vanish; the template scans ω from ω_max/1000 up to ω_max in 240 geometric steps (3 % apart) for sign changes of that condition, bisects each, and stops at the third.

### Shaft and disks

```calc {description="Bearing span (the shaft's ends)"}
L = 1 m_
```

```calc {description="Shaft diameter"}
d = 40 mm_
```

```calc {description="Young's modulus of the shaft"}
E = 200 GPa_
```

```calc {description="Density of the shaft (0 treats it as massless)"}
ρ = 7850 kg/m³_
```

List each disk's position from the left bearing, in ascending order and strictly inside the span, and its mass.

```calc {description="Disk positions from the left bearing, ascending"}
x = [400; 700] mm_
```

```calc {description="Disk masses, in the same order"}
m_d = [20; 10] kg_
```

```calc
> Second moment of area
I = π d^4 / 64 = ?? mm⁴
> Mass of the shaft, lumped at 20 stations
m_sh = ρ π d^2 / 4 L = ?? kg
```

### Holzer's method

The working is dimensionless: positions over L, masses over the total M_t, and λ = ω² M_t L³/(E I). The first function builds the stations (bearings, disks and the shaft's 20 lumps at the middles of twentieths of the span, in order); the second carries two starts along the shaft at once (unit slope, and unit shear, at the left bearing) and returns the determinant of their deflections and moments at the right bearing, zero at a critical speed; the third scans. Each step is rounded to 15 significant figures (`roundSig`), which keeps the engine's exact fractions from growing without bound along the shaft.

```calc
function stations(ξ, μ, ms, which)
    N = 20
    nd = length(ξ)
    ξ_e = vcat(ξ, 2)
    μ_e = vcat(μ, 0)
    p = [0]
    q = [0]
    k = 1
    for j in 0:(N - 1)
        ξ_l = (j + 0.5) / N
        while ξ_e[k] ≤ ξ_l
            p = vcat(p, ξ_e[k])
            q = vcat(q, μ_e[k])
            k = k + 1
        end
        p = vcat(p, ξ_l)
        q = vcat(q, ms / N)
    end
    while k ≤ nd
        p = vcat(p, ξ_e[k])
        q = vcat(q, μ_e[k])
        k = k + 1
    end
    p = vcat(p, 1)
    q = vcat(q, 0)
    if which == 1
        return p
    end
    return q
end
```

```calc
function shaftRemainder(p, q, ν)
    λ = ν^2
    y_a = 0
    θ_a = 1
    M_a = 0
    S_a = 0
    y_b = 0
    θ_b = 0
    M_b = 0
    S_b = 1
    n = length(p)
    for i in 1:n
        S_a = roundSig(S_a + q[i] λ y_a, 15)
        S_b = roundSig(S_b + q[i] λ y_b, 15)
        if i < n
            h = p[i + 1] - p[i]
            y_a = roundSig(y_a + θ_a h + M_a h^2 / 2 + S_a h^3 / 6, 15)
            θ_a = roundSig(θ_a + M_a h + S_a h^2 / 2, 15)
            M_a = roundSig(M_a + S_a h, 15)
            y_b = roundSig(y_b + θ_b h + M_b h^2 / 2 + S_b h^3 / 6, 15)
            θ_b = roundSig(θ_b + M_b h + S_b h^2 / 2, 15)
            M_b = roundSig(M_b + S_b h, 15)
        end
    end
    return y_a M_b - M_a y_b
end
```

```calc
function shaftCriticals(p, q, νmax)
    steps = 240
    out = [0]
    count = 0
    v_0 = νmax / 1000
    r_0 = shaftRemainder(p, q, v_0)
    for i in 1:steps
        if count < 3
            v_1 = νmax 1000^(i / steps - 1)
            r_1 = shaftRemainder(p, q, v_1)
            if sign(r_0) ≠ sign(r_1)
                a = v_0
                b = v_1
                r_a = r_0
                for j in 1:40
                    c = (a + b) / 2
                    r_c = shaftRemainder(p, q, c)
                    if sign(r_a) == sign(r_c)
                        a = c
                        r_a = r_c
                    else
                        b = c
                    end
                end
                count = count + 1
                if count == 1
                    out = [(a + b) / 2]
                else
                    out = vcat(out, (a + b) / 2)
                end
            end
            v_0 = v_1
            r_0 = r_1
        end
    end
    return out
end
```

```calc {description="Highest speed to search up to (the search starts at a thousandth of it)"}
ω_max = 20000 rad/s_
```

```calc
> Total mass, the scale of the working
M_t = sum(m_d) + m_sh = ?? kg
> Speed scale
ω_s = sqrt(E I / (M_t L^3)) = ?? rad/s
> Station positions over L
p_s = stations(x / L, m_d / M_t, m_sh / M_t, 1) = !
> Station masses over M_t
q_s = stations(x / L, m_d / M_t, m_sh / M_t, 2) = !
```

```calc {description="The first three critical speeds between ω_max/1000 and ω_max, lowest first"}
ω_c = shaftCriticals(p_s, q_s, ω_max / ω_s) ω_s = ?? rad/s
```

```calc
> First critical speed
ω_c1 = ω_c[1] = ?? rad/s
n_c1 = ω_c1 = ?? rpm
> Second critical speed
ω_c2 = ω_c[2] = ?? rad/s
```

### Running speed and whirl

Below about 70 % of the first critical a shaft is "stiff"; above about 140 % it is "flexible" and passes through the critical on run-up, which needs enough damping or a quick passage. In between it is near resonance. The whirl radius of a disk whose centre of mass is e off the shaft axis is r_w = e q²/(1 − q²), q being the running speed over the first critical, taking the first mode as a single disk (Eq. 149b): positive below the critical (the heavy side flies out), negative above it (the heavy side flies in, and the whirl tends to e). A balancing grade gives e: ISO 21940-11's G 6.3 at 3000 rpm is about 0.02 mm.

```calc {description="Running speed"}
n = 3000 rpm_
```

```calc {description="Eccentricity of the disk's centre of mass (its unbalance over its mass)"}
e = 0.02 mm_
```

```calc
> Running speed over the first critical
q = n / ω_c1 = ??
> Whirl radius of the disk, Eq. 149b
r_w = e q^2 / (1 - q^2) = ?? mm
```

### Separation

```calc {description="Smallest acceptable separation of the running speed from a critical, as a fraction of it"}
s_min = 0.2_
```

```calc {description="Separation of the running speed from the nearest critical found"}
s = min(abs(n / ω_c - 1)) = ??
```

```calc
@test s ≥ s_min
```
