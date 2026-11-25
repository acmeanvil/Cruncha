+++
cruncha-template = 1
id = "mechanical.vdi2230.loadFactor"
version = "1.0.0"
title = "Bolted joint resilience and load factor, VDI 2230 (R3, R4)"
summary = "Elastic resilience of one cylindrical bolt and of the parts it clamps (substitutional deformation cone, with a sleeve where the interface is narrow), the load factor Φ for concentric or eccentric clamping, and the preload lost to embedding, by VDI 2230 Part 1 steps R3 and R4."
category = ["Mechanical engineering", "Fastening"]
tags = ["bolt", "bolted joint", "VDI 2230", "resilience", "stiffness", "joint stiffness", "load factor", "joint constant", "deformation cone", "embedding", "eccentric", "tapped thread", "R3", "R4"]
from = "lacewing:mechanical.vdi2230"
license = "GPL-3.0-or-later"
inputs = ["d", "P", "E_S", "l_K", "l_1", "c_SK", "c_M", "E_M", "w", "d_W", "d_h", "D_A", "D_A′", "E_P", "n", "s_sym", "a", "I_Bers", "f_Z"]
outputs = ["δ_SK", "δ_1", "δ_Gew", "δ_G", "δ_M", "δ_S", "φ", "D_AGr", "δ_P", "δ_P′", "δ_P′′", "Φ_K", "Φ", "F_Z"]

[[references]]
work = "VDI 2230 Part 1, Systematic calculation of highly stressed bolted joints: joints with one cylindrical bolt"
edition = "2015"
locator = "Sec. 5.1.1 (bolt resilience, Eqs. 19 to 31), Sec. 5.1.2 (clamped parts, Eqs. 39 to 43), Sec. 5.3 (load factor, Eqs. 67, 71, 80, 81, 83, 91, 92); calculation steps R3 and R4 (Eq. R4/1); Table 2 (load introduction factor n), Table 5 (embedding); Examples B1, B2 and B5"

[[tests]]
name = "VDI 2230 Example B2: M16 × 80 10.9 hexagon head bolt and nut, cast iron flanges"
source = "Lacewing Lab vdi2230_test, after VDI 2230 Part 1:2015 Example B2: δS 2.249·10⁻⁶ mm/N (the book rounds each term), tan φD 0.598, DA,Gr 58.4 mm (cones only), δP 0.781·10⁻⁶ mm/N, fZ 11 µm (Table 5, shear, Rz < 10 µm), FZ 3630 N"
inputs = { d = "16 mm", P = "2 mm", E_S = "205 GPa", l_K = "60 mm", l_1 = "42 mm", c_SK = "0.5", c_M = "0.4", E_M = "205 GPa", w = "1", d_W = "22.5 mm", d_h = "17 mm", D_A = "99 mm", "D_A′" = "99 mm", E_P = "110 GPa", n = "0.3", s_sym = "0 mm", f_Z = "11 µm" }
expect = { "δ_S" = "2.249 µm/kN", "φ" = "30.88°", D_AGr = "58.4 mm", "δ_P" = "0.781 µm/kN", F_Z = "3630 N" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "VDI 2230 Example B5: M20 × 60 10.9 socket head cap screw into a tapped cylinder, eccentric"
source = "Lacewing Lab vdi2230_test, after VDI 2230 Part 1:2015 Example B5: δM 0.1025, δS 1.157·10⁻⁶ mm/N; tan φE 0.564, DA,Gr 68.3 mm (cone and sleeve), δP 0.2458; ssym −1.7 mm, a 24 mm, IBers 112 869 mm⁴: δP* 0.250, δP** 0.184·10⁻⁶ mm/N; n 0.28: Φ*en 0.057; fZ 8 µm (Rz 16 µm, axial, one interface): FZ 5703 N (the book carries tan φ rounded to 0.564)"
inputs = { d = "20 mm", P = "2.5 mm", E_S = "205 GPa", l_K = "35 mm", l_1 = "7.5 mm", c_SK = "0.4", c_M = "0.33", E_M = "205 GPa", w = "2", d_W = "28.87 mm", d_h = "22 mm", D_A = "39.1 mm", "D_A′" = "87.1 mm", E_P = "205 GPa", n = "0.28", s_sym = "-1.7 mm", a = "24 mm", I_Bers = "112869 mm^4", f_Z = "8 µm" }
expect = { "δ_M" = "0.1025 µm/kN", "δ_S" = "1.157 µm/kN", "φ" = "29.42°", D_AGr = "68.3 mm", "δ_P" = "0.2458 µm/kN", "δ_P′" = "0.250 µm/kN", "δ_P′′" = "0.184 µm/kN", "Φ" = "0.057", F_Z = "5703 N" }
tolerance = "0.5%"
checks = "pass"

[[tests]]
name = "VDI 2230 Example B1: M12 × 60 10.9 socket head cap screw, piston to rod (resiliences)"
source = "Lacewing Lab vdi2230_test, after VDI 2230 Part 1:2015 Example B1: δS 2.95, δM 0.171·10⁻⁶ mm/N (tapped thread, 0.33d); the plates by the through-bolted cone with dW 21.11 mm, D′A 80 mm: tan φD 0.566, DA,Gr 44.9 mm, δP 0.363·10⁻⁶ mm/N; fZ 8 µm: FZ 2415 N. (The example takes δPZu from the tapped thread with the through-bolted cone, giving ΦK 0.161; this template ties both to w, so only the resiliences and FZ are compared.)"
inputs = { d = "12 mm", P = "1.75 mm", E_S = "205 GPa", l_K = "42 mm", l_1 = "24 mm", c_SK = "0.4", c_M = "0.33", E_M = "205 GPa", w = "1", d_W = "21.11 mm", d_h = "13.5 mm", D_A = "80 mm", "D_A′" = "80 mm", E_P = "205 GPa", n = "0.35", s_sym = "0 mm", f_Z = "8 µm" }
expect = { "δ_S" = "2.95 µm/kN", "δ_M" = "0.171 µm/kN", "φ" = "29.51°", D_AGr = "44.9 mm", "δ_P" = "0.363 µm/kN", F_Z = "2415 N" }
tolerance = "0.2%"
checks = "pass"

[[tests]]
name = "Hand check: Example B2's joint, concentric, n = 0.3"
source = "By hand from the B2 resiliences: ΦK = δP/(δS + δP) = 0.781065/(2.248361 + 0.781065) = 0.257826 (through-bolted, δPZu = 0); Φ = n ΦK = 0.3 × 0.257826 = 0.0773478"
inputs = { d = "16 mm", P = "2 mm", E_S = "205 GPa", l_K = "60 mm", l_1 = "42 mm", c_SK = "0.5", c_M = "0.4", E_M = "205 GPa", w = "1", d_W = "22.5 mm", d_h = "17 mm", D_A = "99 mm", "D_A′" = "99 mm", E_P = "110 GPa", n = "0.3", s_sym = "0 mm", f_Z = "11 µm" }
expect = { "δ_S" = "2.248361 µm/kN", "δ_P" = "0.781065 µm/kN", "Φ_K" = "0.257826", "Φ" = "0.0773478" }
tolerance = "1e-5"
checks = "pass"

[[tests]]
name = "Hand check: interface no wider than the bearing face, sleeve only"
source = "By hand: B2's joint with DA = 20 mm < dW = 22.5 mm: δP = 4 lK/(EP π (DA² − dh²)) = 240/(110 000 × π × 111) mm/N = 6.256705·10⁻⁶ mm/N"
inputs = { d = "16 mm", P = "2 mm", E_S = "205 GPa", l_K = "60 mm", l_1 = "42 mm", c_SK = "0.5", c_M = "0.4", E_M = "205 GPa", w = "1", d_W = "22.5 mm", d_h = "17 mm", D_A = "20 mm", "D_A′" = "20 mm", E_P = "110 GPa", n = "0.3", s_sym = "0 mm", f_Z = "11 µm" }
expect = { "δ_P" = "6.256705 µm/kN" }
tolerance = "1e-6"
checks = "pass"

[[tests]]
name = "Shank longer than the clamp length: the geometry check fails"
source = "Example B2 with l1 = 65 mm in a 60 mm clamp length"
inputs = { d = "16 mm", P = "2 mm", E_S = "205 GPa", l_K = "60 mm", l_1 = "65 mm", c_SK = "0.5", c_M = "0.4", E_M = "205 GPa", w = "1", d_W = "22.5 mm", d_h = "17 mm", D_A = "99 mm", "D_A′" = "99 mm", E_P = "110 GPa", n = "0.3", s_sym = "0 mm", f_Z = "11 µm" }
expect = { D_AGr = "58.4 mm" }
tolerance = "0.2%"
checks = "fail"
+++

## Bolted joint resilience and load factor (VDI 2230, R3 and R4)

How the axial working load on a bolted joint divides between the bolt and the parts it clamps, by VDI 2230 Part 1:2015, Sections 5.1 to 5.3 and calculation steps R3 and R4. The joint has one cylindrical bolt, either through-bolted with a nut or screwed into a tapped part. The results (the load factor Φ and the embedding loss F_Z) are inputs to the VDI 2230 preload and working-checks template.

The resiliences are in µm/kN, which is the 10⁻⁶ mm/N the guideline prints. Resilience is the inverse of stiffness.

### Bolt

The bolt is a series of cylinders: the head, a plain shank of length l₁ inside the clamp length, and the rest of the grip as free loaded thread, then the engaged thread and the nut (or the tapped thread). The head and the nut deform like a short length of the nominal section, given here as fractions of d: VDI 2230 takes 0.5d for a hexagon head and 0.4d for a socket head cap screw (Eq. 21), and 0.4d for a nut or 0.33d for a tapped thread (Eqs. 29, 30).

```calc {description="Nominal thread diameter"}
d = 16 mm_
```

```calc {description="Thread pitch"}
P = 2 mm_
```

```calc {description="Young's modulus of the bolt (about 205 GPa for steel)"}
E_S = 205 GPa_
```

```calc {description="Clamp length: under the head to the nut, or to the tapped thread"}
l_K = 60 mm_
```

```calc {description="Plain shank (at d) inside the clamp length; the rest is free thread"}
l_1 = 42 mm_
```

```calc {description="Head deformation length over d: 0.5 hexagon head, 0.4 socket head"}
c_SK = 0.5_
```

```calc {description="Nut deformation length over d: 0.4 nut, 0.33 tapped thread"}
c_M = 0.4_
```

```calc {description="Young's modulus of the nut or tapped part"}
E_M = 205 GPa_
```

```calc
> Minor diameter of the bolt thread (DIN 13)
d_3 = d - 1.226869 P = ?? mm
> Nominal cross-section
A_N = π d^2 / 4 = ?? mm²
> Minor-diameter cross-section
A_d3 = π d_3^2 / 4 = ?? mm²
```

```calc
> Head, Eqs. 20 and 21
δ_SK = c_SK d / (E_S A_N) = ?? µm/kN
> Plain shank, Eq. 22
δ_1 = l_1 / (E_S A_N) = ?? µm/kN
> Free loaded thread in the grip, Eq. 23
δ_Gew = (l_K - l_1) / (E_S A_d3) = ?? µm/kN
> Engaged thread, 0.5d at the minor section, Eqs. 25 and 26
δ_G = 0.5 d / (E_S A_d3) = ?? µm/kN
> Nut or tapped thread, Eqs. 27 to 30
δ_M = c_M d / (E_M A_N) = ?? µm/kN
> Bolt resilience, Eq. 19
δ_S = δ_SK + δ_1 + δ_Gew + δ_G + δ_M = ?? µm/kN
```

### Clamped parts

The clamped parts deform as a substitutional cone spreading from the bearing face (two cones meeting at the interface for a through-bolted joint, w = 1; one cone for a tapped thread joint, w = 2). If the parts are narrower than the cone's limiting diameter D_A,Gr, the cone is cut off by a cylindrical sleeve. D′_A is the substitutional outside diameter of the basic solid that sets the cone's angle; it is usually D_A.

```calc {description="Joint type: 1 through-bolted (bolt and nut), 2 tapped thread joint"}
w = 1_
```

```calc {description="Outside diameter of the head's bearing face"}
d_W = 22.5 mm_
```

```calc {description="Clearance hole diameter"}
d_h = 17 mm_
```

```calc {description="Outside diameter of the interface between the clamped parts"}
D_A = 99 mm_
```

```calc {description="Substitutional outside diameter of the basic solid (usually D_A)"}
D_A′ = 99 mm_
```

```calc {description="Young's modulus of the clamped parts"}
E_P = 110 GPa_
```

```calc
> Slenderness of the clamped parts
β_L = l_K / d_W = ??
> Outside diameter ratio
y = D_A′ / d_W = ??
> Angle of the substitutional deformation cone, Eq. 43 (through-bolted) or Eq. 42 (tapped thread)
φ = atan({ 0.362 + 0.032 ln(β_L / 2) + 0.153 ln(y) if w < 1.5; 0.348 + 0.013 ln(β_L) + 0.193 ln(y) otherwise }) = ?? °
> Limiting diameter of the cone, Eq. 39
D_AGr = d_W + w l_K tan(φ) = ?? mm
```

```calc {description="Resilience of the clamped parts: cones (Eq. 40), cone and sleeve (Eq. 41), or sleeve only"}
δ_P = { 2 ln((d_W + d_h) (d_W + w l_K tan(φ) - d_h) / ((d_W - d_h) (d_W + w l_K tan(φ) + d_h))) / (w E_P π d_h tan(φ)) if D_A ≥ D_AGr; (2 / (w d_h tan(φ)) ln((d_W + d_h) (D_A - d_h) / ((d_W - d_h) (D_A + d_h))) + 4 / (D_A^2 - d_h^2) (l_K - (D_A - d_W) / (w tan(φ)))) / (E_P π) if D_A > d_W; 4 l_K / (E_P π (D_A^2 - d_h^2)) otherwise } = ?? µm/kN
```

### Load factor

The load introduction factor n says where the axial load enters the clamped parts: 1 under the head and nut, 0 at the interface. Read it from VDI 2230 Table 2 for the joint type (SV 1 to SV 6 of its Figure 22) and the ratios a_k/h and l_A/h. A tapped thread joint adds the tapped part's own resilience to the plates' side (Eq. 81).

For eccentric clamping, s_sym is the bolt's distance from the symmetry axis of the interface and a the load's, with the signs of VDI 2230 Table 4 (a positive, s_sym positive on the same side as a). I_Bers is the substitutional moment of gyration of the deformation body (Eqs. 59 to 62). With s_sym = 0 the joint is concentric and a and I_Bers have no effect.

```calc {description="Load introduction factor (VDI 2230 Table 2)"}
n = 0.3_
```

```calc {description="Bolt axis from the symmetry axis of the interface (Table 4 sign)"}
s_sym = 0 mm_
```

```calc {description="Line of action of the axial load from the symmetry axis"}
a = 0 mm_
```

```calc {description="Substitutional moment of gyration of the clamped parts, Eqs. 59 to 62"}
I_Bers = 100000 mm^4_
```

```calc
> Extra plate resilience of a tapped thread joint, Eq. 81
δ_PZu = (w - 1) δ_M = ?? µm/kN
> Plate resilience with eccentric clamping, Eq. 67
δ_P′ = δ_P + s_sym^2 l_K / (E_P I_Bers) = ?? µm/kN
> Plate resilience with eccentric clamping and loading, Eq. 71
δ_P′′ = δ_P + a s_sym l_K / (E_P I_Bers) = ?? µm/kN
> Load factor for loading under the head, concentric, Eq. 80
Φ_K = (δ_P + δ_PZu) / (δ_S + δ_P) = ??
> Load factor: the share of the axial load the bolt takes, Eqs. 83, 91 and 92
Φ = n (δ_P′′ + δ_PZu) / (δ_S + δ_P′) = ??
```

### Embedding

Embedding is the permanent settling of the rough surfaces in the thread, under the head and nut and at the inner interfaces after tightening. VDI 2230 Table 5 gives guide amounts by roughness, by whether the load is axial or transverse, and per interface; it is for solid steel parts, so a stack of sheets needs measured values. The preload lost is the amount over the combined resilience.

```calc {description="Amount of embedding, the sum of thread, bearing faces and interfaces (Table 5)"}
f_Z = 11 µm_
```

```calc {description="Preload lost to embedding, Eq. R4/1"}
F_Z = f_Z / (δ_S + δ_P) = ?? N
```

### Checks

```calc
@test 0 'mm' ≤ l_1 ≤ l_K
```

```calc
@test d_h < d_W
```

```calc
@test d_h < D_A
```

```calc
@test 0 ≤ n ≤ 1
```
