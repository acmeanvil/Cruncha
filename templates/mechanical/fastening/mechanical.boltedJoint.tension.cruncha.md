+++
cruncha-template = 1
id = "mechanical.boltedJoint.tension"
version = "1.0.0"
title = "Preloaded bolted joint in tension (Shigley)"
summary = "One preloaded bolt or cap screw in a tension joint: bolt stiffness from its shank and threaded lengths in the grip, member stiffness of a stack of layers by the pressure-cone (frustum) method, the joint constant C, the preload and tightening torque, and the proof, load and separation factors of safety."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt", "bolted joint", "cap screw", "preload", "joint constant", "stiffness", "frustum", "pressure cone", "torque", "proof load", "separation", "load factor", "Shigley"]
from = "lacewing:mechanical.boltedJoint"
license = "GPL-3.0-or-later"
inputs = ["d", "A_t", "L", "L_T", "E_b", "d_w", "α", "t_m", "E_m", "t_tap", "E_tap", "S_p", "f_p", "K", "P"]
outputs = ["l", "l_d", "l_t", "k_b", "k_m", "C", "F_p", "F_i", "T", "σ_i", "F_b", "σ_b", "n_p", "n_L", "n_0"]

[[references]]
work = "Shigley's Mechanical Engineering Design (Budynas, Nisbett)"
edition = "10th"
locator = "Secs. 8-4 to 8-9: Eqs. 8-13, 8-14 (thread length), 8-17 (bolt stiffness), Table 8-7 (grip and effective grip), 8-18, 8-19 and Fig. 8-15 (frusta), Fig. 8-21 (tapped member), 8-24 (C), 8-27 (torque), 8-28 to 8-31 (factors, preload); Examples 8-2, 8-4, 8-5"

[[tests]]
name = "Shigley Ex. 8-4: 5/8-11 UNC grade 5 through 1.5 in of cast iron, 6 kip per bolt"
source = "Lacewing Lab bolted_joint_test, after Shigley 10th Example 8-4: kb = 5.21, km = 8.95 Mlbf/in, C = 0.368, Fi = 14.4 kip, nL = 2.18, np = 1.16, n0 = 3.80"
inputs = { d = "0.625 in", A_t = "0.226 in²", L = "2.25 in", L_T = "1.5 in", E_b = "30 Mpsi", d_w = "0.9375 in", t_m = "[1.5] in", E_m = "[14] Mpsi", t_tap = "0 in", S_p = "85 ksi", f_p = "0.75", P = "6 kip" }
expect = { k_b = "5210 kip/in", k_m = "8950 kip/in", C = "0.368", F_i = "14.4 kip", n_L = "2.18", n_p = "1.16", n_0 = "3.80" }
tolerance = "0.4%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 8-5: 5/8-11 cap screw through a washer and steel cover into cast iron, 5 kip"
source = "Lacewing Lab bolted_joint_test, after Shigley 10th Example 8-5: 1/16 in washer and 5/8 in cover (30 Mpsi) on cast iron (16 Mpsi), effective grip 1.0 in, screw threaded through the grip (kb = At·E/l = 6.78 Mlbf/in), km = 17.40 Mlbf/in, C = 0.280, np = 1.22, nL = 3.44 (3.43 unrounded), n0 = 4.00"
inputs = { d = "0.625 in", A_t = "0.226 in²", L = "1.25 in", L_T = "1.25 in", E_b = "30 Mpsi", d_w = "0.9375 in", t_m = "[0.0625; 0.625] in", E_m = "[30; 30] Mpsi", t_tap = "0.625 in", E_tap = "16 Mpsi", S_p = "85 ksi", f_p = "0.75", P = "5 kip" }
expect = { l = "1.0 in", k_b = "6780 kip/in", k_m = "17400 kip/in", C = "0.280", n_p = "1.22", n_L = "3.43", n_0 = "4.00" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "Shigley Ex. 8-2: 1/2-20 UNF × 1.5 in, steel plate and washer on cast iron"
source = "Lacewing Lab bolted_joint_test, after Shigley 10th Example 8-2: 0.095 in washer and 1/2 in plate (30 Mpsi) on 3/4 in cast iron (14.5 Mpsi), dw = 0.75 in: grip 1.345 in, LT = 1.25 in, km = 9.378 Mlbf/in, kb = 3.69 Mlbf/in"
inputs = { d = "0.5 in", A_t = "0.1599 in²", L = "1.5 in", L_T = "1.25 in", E_b = "30 Mpsi", d_w = "0.75 in", t_m = "[0.595; 0.75] in", E_m = "[30; 14.5] Mpsi", t_tap = "0 in" }
expect = { l = "1.345 in", l_d = "0.25 in", k_m = "9378 kip/in", k_b = "3690 kip/in" }
tolerance = "0.15%"
checks = "pass"

[[tests]]
name = "Hand check: Ex. 8-4 bolt stiffness, preload and torque"
source = "By hand: Ad = π(0.625)²/4 = 0.306796 in², ld = lt = 0.75 in, kb = 0.306796 × 0.226 × 30e6 / (0.306796 × 0.75 + 0.226 × 0.75) = 5.205438e6 lbf/in; Fp = 0.226 × 85 = 19.21 kip, Fi = 14.4075 kip, T = 0.2 × 14.4075 × 0.625 = 1.80094 kip·in"
inputs = { d = "0.625 in", A_t = "0.226 in²", L = "2.25 in", L_T = "1.5 in", E_b = "30 Mpsi", d_w = "0.9375 in", t_m = "[1.5] in", E_m = "[14] Mpsi", t_tap = "0 in", S_p = "85 ksi", f_p = "0.75", K = "0.2", P = "6 kip" }
expect = { l_d = "0.75 in", l_t = "0.75 in", k_b = "5205.438 kip/in", F_p = "19.21 kip", F_i = "14.4075 kip", T = "1.800938 kip·in", "σ_i" = "63.75 ksi" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Ex. 8-4 at 25 kip per bolt: the joint opens"
source = "As Ex. 8-4 with four times the load and more: n0 = 14.4 / (25 × 0.632) = 0.91, under 1"
inputs = { d = "0.625 in", A_t = "0.226 in²", L = "2.25 in", L_T = "1.5 in", E_b = "30 Mpsi", d_w = "0.9375 in", t_m = "[1.5] in", E_m = "[14] Mpsi", t_tap = "0 in", S_p = "85 ksi", f_p = "0.75", P = "25 kip" }
expect = { n_0 = "0.912" }
tolerance = "0.5%"
checks = "fail"
+++

## Preloaded bolted joint in tension

One bolt (with a nut) or cap screw (into a tapped member) clamping a stack of members, loaded by an external tension P that tries to separate them (Shigley Secs. 8-4 to 8-9). The bolt and the members act as springs in parallel: the bolt takes the share C of P and the members lose the rest of their clamping. The members are taken as elastic pressure cones; bending of the bolt, prying and gaskets are not modelled. For fatigue of the bolt under a fluctuating P, use the bolted joint fatigue template with this C and F_i; for a pattern of bolts, the bolt group templates.

### Bolt

The thread size gives d and the tensile-stress area A_t (ASME B1.1 for inch threads, ISO 898-1 or Shigley Tables 8-1 and 8-2 for metric). The threaded length of a standard bolt is L_T = 2d + 1/4 in for L ≤ 6 in and 2d + 1/2 in above (Eq. 8-13); for metric bolts 2d + 6, 12 or 25 mm for L ≤ 125, ≤ 200 and above 200 mm (Eq. 8-14). Enter L_T ≥ L for a bolt threaded its whole length.

```calc {description="Nominal (major) diameter"}
d = 0.625 in_
```

```calc {description="Tensile-stress area of the thread"}
A_t = 0.226 in²_
```

```calc {description="Bolt length under the head"}
L = 1.25 in_
```

```calc {description="Threaded length of the bolt, Eqs. 8-13 and 8-14"}
L_T = 1.25 in_
```

```calc {description="Bolt Young's modulus"}
E_b = 30 Mpsi_
```

### Members

List the clamped layers from the head down as column vectors of thicknesses and moduli; a washer is a layer. A single layer is a one-entry vector, `[1.5] in`. For a cap screw, the tapped member is entered separately by its full thickness t_tap: only half of min(t_tap, d) takes part in the grip (Table 8-7, Fig. 8-21). For a bolt and nut, enter t_tap = 0.

The bearing faces under the head and the nut (or washer) have diameter d_w, 1.5d for a standard hex head or washer face. The pressure-cone half-angle α is usually 30°.

```calc {description="Thickness of each clamped layer, head side first"}
t_m = [0.0625; 0.625] in_
```

```calc {description="Young's modulus of each clamped layer"}
E_m = [30; 30] Mpsi_
```

```calc {description="Full thickness of the tapped member (0 for a bolt and nut)"}
t_tap = 0.625 in_
```

```calc {description="Young's modulus of the tapped member"}
E_tap = 16 Mpsi_
```

```calc {description="Bearing face diameter of head and nut (1.5d for a washer face)"}
d_w = 0.9375 in_
```

```calc {description="Pressure-cone half-angle"}
α = 30°_
```

### Grip and bolt stiffness

```calc
> Layers in the grip, with the tapped member's effective depth last
t_g = vcat(t_m, min(t_tap, d) / 2) = ?? in
E_g = vcat(E_m, E_tap) = ?? Mpsi
> Effective grip, Table 8-7
l = sum(t_g) = ?? in
> Unthreaded shank in the grip, Table 8-7
l_d = min(l, max(L - L_T, 0 'in')) = ?? in
> Threaded length in the grip
l_t = l - l_d = ?? in
> Major-diameter area of the shank
A_d = π d^2 / 4 = ?? in²
> Bolt stiffness, shank and thread in series, Eq. 8-17
k_b = A_d A_t E_b / (A_d l_t + A_t l_d) = ?? kip/in
```

### Member stiffness

Two cones of half-angle α grow from the bearing faces at both ends of the grip and meet at mid-grip (Fig. 8-15). Each layer's part of each cone is a frustum of stiffness πEd tan α / ln[(2t tan α + D − d)(D + d) / ((2t tan α + D + d)(D − d))], where D is the frustum's smaller diameter (Eq. 8-19), and the frusta act in series (Eq. 8-18). The function returns the compliance of one frustum, and the stack's stiffness is the reciprocal of their sum.

```calc
function frustumCompliance(E, d, t, D, ta)
    return ln((2 t ta + D - d) (D + d) / ((2 t ta + D + d) (D - d))) / (π E d ta)
end
```

```calc
function coneStiffness(t, E, d, dw, α)
    l = sum(t)
    h = l / 2
    ta = tan(α)
    x = 0 t[1]
    c = 0 / (E[1] d)
    for i in 1:length(t)
        x_1 = x + t[i]
        if x < h
            c = c + frustumCompliance(E[i], d, min(x_1, h) - x, dw + 2 x ta, ta)
        end
        if x_1 > h
            c = c + frustumCompliance(E[i], d, x_1 - max(x, h), dw + 2 (l - x_1) ta, ta)
        end
        x = x_1
    end
    return 1 / c
end
```

```calc {description="Member stiffness, frusta in series, Eqs. 8-18 and 8-19"}
k_m = coneStiffness(t_g, E_g, d, d_w, α) = ?? kip/in
```

```calc {description="Joint constant, the bolt's share of the external load, Eq. 8-24"}
C = k_b / (k_b + k_m) = ??
```

### Preload and torque

Shigley recommends a preload of 0.75 of the proof load for a reusable joint and 0.90 for a permanent one (Eq. 8-31). The proof strength S_p is the grade's (SAE J429 or ASTM for inch bolts, ISO 898-1 for metric property classes; Shigley Tables 8-9 to 8-11). The torque coefficient K is about 0.2 for an unlubricated zinc-plated or as-received bolt (Shigley Table 8-15 lists others); torque sets preload only roughly, so use bolt stretch or turn of the nut where preload matters.

```calc {description="Minimum proof strength of the bolt grade"}
S_p = 85 ksi_
```

```calc {description="Preload as a fraction of the proof load (0.75 reusable, 0.90 permanent)"}
f_p = 0.75_
```

```calc {description="Torque coefficient"}
K = 0.2_
```

```calc
> Proof load
F_p = A_t S_p = ?? kip
> Preload, Eq. 8-31
F_i = f_p F_p = ?? kip
> Tightening torque, Eq. 8-27
T = K F_i d = ?? lbf·in
> Preload stress on the tensile-stress area
σ_i = F_i / A_t = ?? ksi
```

### External load and factors of safety

```calc {description="External tensile load per bolt"}
P = 5 kip_
```

```calc
> Bolt load at P, joint closed
F_b = F_i + C P = ?? kip
> Bolt stress at P
σ_b = F_b / A_t = ?? ksi
> Yielding (proof) factor of safety, Eq. 8-28
n_p = S_p A_t / (C P + F_i) = ??
> Load factor against overload to proof, Eq. 8-29
n_L = (S_p A_t - F_i) / (C P) = ??
> Load factor against joint separation, Eq. 8-30
n_0 = F_i / (P (1 - C)) = ??
```

### Checks

```calc
@test n_p ≥ 1
```

```calc
@test n_L ≥ 1
```

```calc
@test n_0 ≥ 1
```
