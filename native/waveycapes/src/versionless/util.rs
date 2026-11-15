//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.util`
//! (`Mth.java`, `Vector3.java`, `Vector4.java`, `CapePoint.java`,
//! `CapeStick.java`). Identifiers keep the upstream camelCase names;
//! Java method overloads keep their arity via the `2`/`xyz` suffix form.

#![allow(non_snake_case, dead_code)]

#[allow(non_snake_case)]
pub mod Mth {
    /// `Mth.PI`
    pub const PI: f32 = 3.1415927;
    /// `Mth.HALF_PI`
    pub const HALF_PI: f32 = 1.5707964;
    /// `Mth.TWO_PI`
    pub const TWO_PI: f32 = 6.2831855;
    /// `Mth.DEG_TO_RAD`
    pub const DEG_TO_RAD: f32 = 0.017453292;
    /// `Mth.RAD_TO_DEG`
    pub const RAD_TO_DEG: f32 = 57.295776;
    /// `Mth.EPSILON`
    pub const EPSILON: f32 = 1.0E-5;
    /// `Mth.SQRT_OF_TWO` (Java evaluates `sqrt(2.0F)` at class init)
    pub const SQRT_OF_TWO: f32 = 1.4142135;

    /// `Mth.truncate(float f, float g)`
    pub fn truncate(f: f32, g: f32) -> f32 {
        let h = 10.0_f64.powf(g as f64);
        ((f as f64 * h) as i64) as f32 / h as f32
    }

    /// `Mth.sqrt(float)`
    pub fn sqrt(f: f32) -> f32 {
        (f as f64).sqrt() as f32
    }

    /// `Mth.floor(float)`
    pub fn floor(f: f32) -> i32 {
        let i = f as i32;
        if f < i as f32 { i - 1 } else { i }
    }

    /// `Mth.fastFloor(double)`
    pub fn fastFloor(d: f64) -> i32 {
        (d + 1024.0) as i32 - 1024
    }

    /// `Mth.floor(double)`
    pub fn floor_d(d: f64) -> i32 {
        let i = d as i32;
        if d < i as f64 { i - 1 } else { i }
    }

    /// `Mth.lfloor(double)`
    pub fn lfloor(d: f64) -> i64 {
        let l = d as i64;
        if d < l as f64 { l - 1 } else { l }
    }

    /// `Mth.absFloor(double)`
    pub fn absFloor(d: f64) -> i32 {
        (if d >= 0.0 { d } else { -d + 1.0 }) as i32
    }

    /// `Mth.abs(float)`
    pub fn abs(f: f32) -> f32 {
        f.abs()
    }

    /// `Mth.abs(int)`
    pub fn abs_i(i: i32) -> i32 {
        i.abs()
    }

    /// `Mth.ceil(float)`
    pub fn ceil(f: f32) -> i32 {
        let i = f as i32;
        if f > i as f32 { i + 1 } else { i }
    }

    /// `Mth.ceil(double)`
    pub fn ceil_d(d: f64) -> i32 {
        let i = d as i32;
        if d > i as f64 { i + 1 } else { i }
    }

    /// `Mth.clamp(byte, byte, byte)`
    pub fn clamp_b(b: i8, c: i8, d: i8) -> i8 {
        if b < c {
            return c;
        }
        if b > d {
            return d;
        }
        b
    }

    /// `Mth.clamp(int, int, int)`
    pub fn clamp(i: i32, j: i32, k: i32) -> i32 {
        if i < j {
            return j;
        }
        if i > k {
            return k;
        }
        i
    }

    /// `Mth.clamp(long, long, long)`
    pub fn clamp_l(l: i64, m: i64, n: i64) -> i64 {
        if l < m {
            return m;
        }
        if l > n {
            return n;
        }
        l
    }

    /// `Mth.clamp(float, float, float)`
    pub fn clamp_f(f: f32, g: f32, h: f32) -> f32 {
        if f < g {
            return g;
        }
        if f > h {
            return h;
        }
        f
    }

    /// `Mth.clamp(double, double, double)`
    pub fn clamp_d(d: f64, e: f64, f: f64) -> f64 {
        if d < e {
            return e;
        }
        if d > f {
            return f;
        }
        d
    }

    /// `Mth.clampedLerp(double, double, double)`
    pub fn clampedLerp_d(d: f64, e: f64, f: f64) -> f64 {
        if f < 0.0 {
            return d;
        }
        if f > 1.0 {
            return e;
        }
        lerp_d(f, d, e)
    }

    /// `Mth.clampedLerp(float, float, float)`
    pub fn clampedLerp(f: f32, g: f32, h: f32) -> f32 {
        if h < 0.0 {
            return f;
        }
        if h > 1.0 {
            return g;
        }
        lerp(h, f, g)
    }

    /// `Mth.absMax(double, double)`
    pub fn absMax(d: f64, e: f64) -> f64 {
        let d = if d < 0.0 { -d } else { d };
        let e = if e < 0.0 { -e } else { e };
        if d > e { d } else { e }
    }

    /// `Mth.intFloorDiv(int, int)`
    pub fn intFloorDiv(i: i32, j: i32) -> i32 {
        i.div_euclid(j)
    }

    /// `Mth.average(long[])`
    pub fn average(ls: &[i64]) -> f64 {
        let mut l: i64 = 0;
        for m in ls {
            l += *m;
        }
        l as f64 / ls.len() as f64
    }

    /// `Mth.equal(float, float)`
    pub fn equal(f: f32, g: f32) -> bool {
        (g - f).abs() < 1.0E-5
    }

    /// `Mth.equal(double, double)`
    pub fn equal_d(d: f64, e: f64) -> bool {
        (e - d).abs() < 9.999_999_747_378_752E-6
    }

    /// `Mth.positiveModulo(int, int)`
    pub fn positiveModulo(i: i32, j: i32) -> i32 {
        i.rem_euclid(j)
    }

    /// `Mth.positiveModulo(float, float)`
    pub fn positiveModulo_f(f: f32, g: f32) -> f32 {
        (f % g + g) % g
    }

    /// `Mth.positiveModulo(double, double)`
    pub fn positiveModulo_d(d: f64, e: f64) -> f64 {
        (d % e + e) % e
    }

    /// `Mth.isDivisionInteger(int, int)`
    pub fn isDivisionInteger(i: i32, j: i32) -> bool {
        i / j * j == i
    }

    /// `Mth.wrapDegrees(int)`
    pub fn wrapDegrees(i: i32) -> i32 {
        let mut j = i % 360;
        if j >= 180 {
            j -= 360;
        }
        if j < -180 {
            j += 360;
        }
        j
    }

    /// `Mth.wrapDegrees(float)`
    pub fn wrapDegrees_f(f: f32) -> f32 {
        let mut g = f % 360.0;
        if g >= 180.0 {
            g -= 360.0;
        }
        if g < -180.0 {
            g += 360.0;
        }
        g
    }

    /// `Mth.wrapDegrees(double)`
    pub fn wrapDegrees_d(d: f64) -> f64 {
        let mut e = d % 360.0;
        if e >= 180.0 {
            e -= 360.0;
        }
        if e < -180.0 {
            e += 360.0;
        }
        e
    }

    /// `Mth.degreesDifference(float, float)`
    pub fn degreesDifference(f: f32, g: f32) -> f32 {
        wrapDegrees_f(g - f)
    }

    /// `Mth.degreesDifferenceAbs(float, float)`
    pub fn degreesDifferenceAbs(f: f32, g: f32) -> f32 {
        abs(degreesDifference(f, g))
    }

    /// `Mth.rotateIfNecessary(float, float, float)`
    pub fn rotateIfNecessary(f: f32, g: f32, h: f32) -> f32 {
        let i = degreesDifference(f, g);
        let j = clamp_f(i, -h, h);
        g - j
    }

    /// `Mth.approach(float, float, float)`
    pub fn approach(f: f32, g: f32, h: f32) -> f32 {
        let h = abs(h);
        if f < g {
            return clamp_f(f + h, f, g);
        }
        clamp_f(f - h, g, f)
    }

    /// `Mth.approachDegrees(float, float, float)`
    pub fn approachDegrees(f: f32, g: f32, h: f32) -> f32 {
        let i = degreesDifference(f, g);
        approach(f, f + i, h)
    }

    /// `Mth.getDouble(String, double)`
    pub fn getDouble(string: &str, d: f64) -> f64 {
        string.parse::<f64>().unwrap_or(d)
    }

    /// `Mth.getDouble(String, double, double)`
    pub fn getDouble2(string: &str, d: f64, e: f64) -> f64 {
        e.max(getDouble(string, d))
    }

    /// `Mth.smallestEncompassingPowerOfTwo(int)`
    pub fn smallestEncompassingPowerOfTwo(i: i32) -> i32 {
        let mut j = i - 1;
        j |= j >> 1;
        j |= j >> 2;
        j |= j >> 4;
        j |= j >> 8;
        j |= j >> 16;
        j + 1
    }

    /// `Mth.isPowerOfTwo(int)`
    pub fn isPowerOfTwo(i: i32) -> bool {
        i != 0 && (i & (i - 1)) == 0
    }

    /// `Mth.MULTIPLY_DE_BRUIJN_BIT_POSITION`
    const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
        0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7,
        26, 12, 18, 6, 11, 5, 10, 9,
    ];

    /// `Mth.ceillog2(int)`
    pub fn ceillog2(i: i32) -> i32 {
        let i = if isPowerOfTwo(i) { i } else { smallestEncompassingPowerOfTwo(i) };
        MULTIPLY_DE_BRUIJN_BIT_POSITION[(((i as i64 * 125613361) >> 27) & 0x1F) as usize]
    }

    /// `Mth.log2(int)`
    pub fn log2(i: i32) -> i32 {
        ceillog2(i) - if isPowerOfTwo(i) { 0 } else { 1 }
    }

    /// `Mth.color(float, float, float)`
    pub fn color_f(f: f32, g: f32, h: f32) -> i32 {
        color(floor(f * 255.0), floor(g * 255.0), floor(h * 255.0))
    }

    /// `Mth.color(int, int, int)`
    pub fn color(i: i32, j: i32, k: i32) -> i32 {
        let mut l = i;
        l = (l << 8) + j;
        l = (l << 8) + k;
        l
    }

    /// `Mth.colorMultiply(int, int)`
    pub fn colorMultiply(i: i32, j: i32) -> i32 {
        let k = (i & 0xFF0000) >> 16;
        let l = (j & 0xFF0000) >> 16;
        let m = (i & 0xFF00) >> 8;
        let n = (j & 0xFF00) >> 8;
        let o = i & 0xFF;
        let p = j & 0xFF;
        let q = (k as f32 * l as f32 / 255.0) as i32;
        let r = (m as f32 * n as f32 / 255.0) as i32;
        let s = (o as f32 * p as f32 / 255.0) as i32;
        i & 0xFF000000u32 as i32 | q << 16 | r << 8 | s
    }

    /// `Mth.colorMultiply(int, float, float, float)`
    pub fn colorMultiply_f(i: i32, f: f32, g: f32, h: f32) -> i32 {
        let j = (i & 0xFF0000) >> 16;
        let k = (i & 0xFF00) >> 8;
        let l = i & 0xFF;
        let m = (j as f32 * f) as i32;
        let n = (k as f32 * g) as i32;
        let o = (l as f32 * h) as i32;
        i & 0xFF000000u32 as i32 | m << 16 | n << 8 | o
    }

    /// `Mth.frac(float)`
    pub fn frac(f: f32) -> f32 {
        f - floor(f) as f32
    }

    /// `Mth.frac(double)`
    pub fn frac_d(d: f64) -> f64 {
        d - lfloor(d) as f64
    }

    /// `Mth.getSeed(int, int, int)`
    pub fn getSeed(i: i32, j: i32, k: i32) -> i64 {
        let mut l = (i as i64 * 3129871) ^ (k as i64 * 116129781) ^ j as i64;
        l = l * l * 42317861 + l * 11;
        l >> 16
    }

    /// `Mth.inverseLerp(double, double, double)`
    pub fn inverseLerp_d(d: f64, e: f64, f: f64) -> f64 {
        (d - e) / (f - e)
    }

    /// `Mth.inverseLerp(float, float, float)`
    pub fn inverseLerp(f: f32, g: f32, h: f32) -> f32 {
        (f - g) / (h - g)
    }

    /// `Mth.FRAC_BIAS`
    const FRAC_BIAS: f64 = f64::from_bits(4805340802404319232);

    /// The `ASIN_TAB`/`COS_TAB` pair Java builds in its static initializer.
    fn asin_tables() -> &'static ([f64; 257], [f64; 257]) {
        use std::sync::OnceLock;
        static TABLES: OnceLock<([f64; 257], [f64; 257])> = OnceLock::new();
        TABLES.get_or_init(|| {
            let mut asin_tab = [0.0_f64; 257];
            let mut cos_tab = [0.0_f64; 257];
            for (i, slot) in asin_tab.iter_mut().enumerate().take(257) {
                let d = i as f64 / 256.0;
                let e = d.asin();
                cos_tab[i] = e.cos();
                *slot = e;
            }
            (asin_tab, cos_tab)
        })
    }

    /// `Mth.atan2(double, double)`
    pub fn atan2(mut d: f64, mut e: f64) -> f64 {
        let f = e * e + d * d;
        if f.is_nan() {
            return f64::NAN;
        }
        let bl = d < 0.0;
        if bl {
            d = -d;
        }
        let bl2 = e < 0.0;
        if bl2 {
            e = -e;
        }
        let bl3 = d > e;
        if bl3 {
            let d1 = e;
            e = d;
            d = d1;
        }
        let g = fastInvSqrt_d(f);
        e *= g;
        d *= g;
        let h = FRAC_BIAS + d;
        let i = h.to_bits() as i64 as i32;
        let (asin_tab, cos_tab) = asin_tables();
        let j = asin_tab[i as usize];
        let k = cos_tab[i as usize];
        let l = h - FRAC_BIAS;
        let m = d * k - e * l;
        let n = (6.0 + m * m) * m * 0.16666666666666666;
        let mut o = j + n;
        if bl3 {
            o = 1.5707963267948966 - o;
        }
        if bl2 {
            o = std::f64::consts::PI - o;
        }
        if bl {
            o = -o;
        }
        o
    }

    /// `Mth.fastInvSqrt(float)`
    pub fn fastInvSqrt(f: f32) -> f32 {
        let g = 0.5 * f;
        let mut i = f.to_bits() as i32;
        i = 1597463007 - (i >> 1);
        let mut f = f32::from_bits(i as u32);
        f *= 1.5 - g * f * f;
        f
    }

    /// `Mth.fastInvSqrt(double)`
    pub fn fastInvSqrt_d(d: f64) -> f64 {
        let e = 0.5 * d;
        let mut l = d.to_bits() as i64;
        l = 6910469410427058090 - (l >> 1);
        let mut d = f64::from_bits(l as u64);
        d *= 1.5 - e * d * d;
        d
    }

    /// `Mth.fastInvCubeRoot(float)`
    pub fn fastInvCubeRoot(f: f32) -> f32 {
        let mut i = f.to_bits() as i32;
        i = 1419967116 - i / 3;
        let mut g = f32::from_bits(i as u32);
        g = 0.6666667 * g + 1.0 / 3.0 * g * g * f;
        g = 0.6666667 * g + 1.0 / 3.0 * g * g * f;
        g
    }

    /// `Mth.hsvToRgb(float, float, float)`
    pub fn hsvToRgb(f: f32, g: f32, h: f32) -> i32 {
        let (n, o, p);
        let i = ((f * 6.0) as i32) % 6;
        let j = f * 6.0 - i as f32;
        let k = h * (1.0 - g);
        let l = h * (1.0 - j * g);
        let m = h * (1.0 - (1.0 - j) * g);
        match i {
            0 => {
                n = h;
                o = m;
                p = k;
            }
            1 => {
                n = l;
                o = h;
                p = k;
            }
            2 => {
                n = k;
                o = h;
                p = m;
            }
            3 => {
                n = k;
                o = l;
                p = h;
            }
            4 => {
                n = m;
                o = k;
                p = h;
            }
            5 => {
                n = h;
                o = k;
                p = l;
            }
            _ => panic!(
                "Something went wrong when converting from HSV to RGB. Input was {}, {}, {}",
                f, g, h
            ),
        }
        let q = clamp((n * 255.0) as i32, 0, 255);
        let r = clamp((o * 255.0) as i32, 0, 255);
        let s = clamp((p * 255.0) as i32, 0, 255);
        q << 16 | r << 8 | s
    }

    /// `Mth.murmurHash3Mixer(int)`
    pub fn murmurHash3Mixer(mut i: i32) -> i32 {
        i ^= i >> 16;
        i = i.wrapping_mul(-2048144789);
        i ^= i >> 13;
        i = i.wrapping_mul(-1028477387);
        i ^= i >> 16;
        i
    }

    /// `Mth.murmurHash3Mixer(long)`
    pub fn murmurHash3Mixer_l(mut l: i64) -> i64 {
        l ^= l >> 33;
        l = l.wrapping_mul(-49064778989728563);
        l ^= l >> 33;
        l = l.wrapping_mul(-4265267296055464877);
        l ^= l >> 33;
        l
    }

    /// `Mth.cumulativeSum(double...)`
    pub fn cumulativeSum(ds: &mut [f64]) {
        let mut d = 0.0;
        for e in ds.iter() {
            d += e;
        }
        for e in ds.iter_mut() {
            *e /= d;
        }
        for i in 0..ds.len() {
            ds[i] = if i == 0 { 0.0 } else { ds[i - 1] } + ds[i];
        }
    }

    /// `Mth.binNormalDistribution(double, double, double, int, int)`
    pub fn binNormalDistribution(d: f64, e: f64, f: f64, i: i32, j: i32) -> Vec<f64> {
        let mut ds = vec![0.0; (j - i + 1) as usize];
        let mut k = 0;
        for l in i..=j {
            ds[k] = 0.0_f64.max(d * ((l as f64 - f) * (l as f64 - f) / 2.0 * e * e).exp());
            k += 1;
        }
        ds
    }

    /// `Mth.binBiModalNormalDistribution(double, double, double, double, double, double, int, int)`
    pub fn binBiModalNormalDistribution(
        d: f64,
        e: f64,
        f: f64,
        g: f64,
        h: f64,
        i: f64,
        j: i32,
        k: i32,
    ) -> Vec<f64> {
        let mut ds = vec![0.0; (k - j + 1) as usize];
        let mut l = 0;
        for m in j..=k {
            ds[l] = 0.0_f64.max(
                d * ((m as f64 - f) * (m as f64 - f) / 2.0 * e * e).exp()
                    + g * ((m as f64 - i) * (m as f64 - i) / 2.0 * h * h).exp(),
            );
            l += 1;
        }
        ds
    }

    /// `Mth.binLogDistribution(double, double, int, int)`
    pub fn binLogDistribution(d: f64, e: f64, i: i32, j: i32) -> Vec<f64> {
        let mut ds = vec![0.0; (j - i + 1) as usize];
        let mut k = 0;
        for l in i..=j {
            ds[k] = (d * (l as f64).ln() + e).max(0.0);
            k += 1;
        }
        ds
    }

    /// `Mth.binarySearch(int, int, IntPredicate)`
    pub fn binarySearch(mut i: i32, j: i32, int_predicate: impl Fn(i32) -> bool) -> i32 {
        let mut k = j - i;
        while k > 0 {
            let l = k / 2;
            let m = i + l;
            if int_predicate(m) {
                k = l;
                continue;
            }
            i = m + 1;
            k -= l + 1;
        }
        i
    }

    /// `Mth.lerp(float, float, float)`
    pub fn lerp(f: f32, g: f32, h: f32) -> f32 {
        g + f * (h - g)
    }

    /// `Mth.lerp(double, double, double)`
    pub fn lerp_d(d: f64, e: f64, f: f64) -> f64 {
        e + d * (f - e)
    }

    /// `Mth.lerp2(double, double, double, double, double, double)`
    pub fn lerp2(d: f64, e: f64, f: f64, g: f64, h: f64, i: f64) -> f64 {
        lerp_d(e, lerp_d(d, f, g), lerp_d(d, h, i))
    }

    /// `Mth.lerp3(double x 11)`
    #[allow(clippy::too_many_arguments)]
    pub fn lerp3(d: f64, e: f64, f: f64, g: f64, h: f64, i: f64, j: f64, k: f64, l: f64, m: f64, n: f64) -> f64 {
        lerp_d(f, lerp2(d, e, g, h, i, j), lerp2(d, e, k, l, m, n))
    }

    /// `Mth.catmullrom(float x 5)`
    pub fn catmullrom(f: f32, g: f32, h: f32, i: f32, j: f32) -> f32 {
        0.5
            * (2.0 * h
                + (i - g) * f
                + (2.0 * g - 5.0 * h + 4.0 * i - j) * f * f
                + (3.0 * h - g - 3.0 * i + j) * f * f * f)
    }

    /// `Mth.smoothstep(double)`
    pub fn smoothstep(d: f64) -> f64 {
        d * d * d * (d * (d * 6.0 - 15.0) + 10.0)
    }

    /// `Mth.smoothstepDerivative(double)`
    pub fn smoothstepDerivative(d: f64) -> f64 {
        30.0 * d * d * (d - 1.0) * (d - 1.0)
    }

    /// `Mth.sign(double)`
    pub fn sign(d: f64) -> i32 {
        if d == 0.0 {
            return 0;
        }
        if d > 0.0 { 1 } else { -1 }
    }

    /// `Mth.rotLerp(float, float, float)`
    pub fn rotLerp(f: f32, g: f32, h: f32) -> f32 {
        g + f * wrapDegrees_f(h - g)
    }

    /// `Mth.diffuseLight(float, float, float)`
    pub fn diffuseLight(f: f32, g: f32, h: f32) -> f32 {
        (f * f * 0.6 + g * g * (3.0 + g) / 4.0 + h * h * 0.8).min(1.0)
    }

    /// `Mth.rotlerp(float, float, float)` (deprecated upstream)
    pub fn rotlerp(f: f32, g: f32, h: f32) -> f32 {
        let mut i = g - f;
        while i < -180.0 {
            i += 360.0;
        }
        while i >= 180.0 {
            i -= 360.0;
        }
        f + h * i
    }

    /// `Mth.rotWrap(double)` (deprecated upstream)
    pub fn rotWrap(d: f64) -> f32 {
        let mut d = d;
        while d >= 180.0 {
            d -= 360.0;
        }
        while d < -180.0 {
            d += 360.0;
        }
        d as f32
    }

    /// `Mth.triangleWave(float, float)`
    pub fn triangleWave(f: f32, g: f32) -> f32 {
        ((f % g - g * 0.5).abs() - g * 0.25) / g * 0.25
    }

    /// `Mth.square(float)`
    pub fn square(f: f32) -> f32 {
        f * f
    }

    /// `Mth.square(double)`
    pub fn square_d(d: f64) -> f64 {
        d * d
    }

    /// `Mth.square(int)`
    pub fn square_i(i: i32) -> i32 {
        i * i
    }

    /// `Mth.square(long)`
    pub fn square_l(l: i64) -> i64 {
        l * l
    }

    /// `Mth.cube(float)`
    pub fn cube(f: f32) -> f32 {
        f * f * f
    }

    /// `Mth.clampedMap(double x 5)`
    pub fn clampedMap_d(d: f64, e: f64, f: f64, g: f64, h: f64) -> f64 {
        clampedLerp_d(g, h, inverseLerp_d(d, e, f))
    }

    /// `Mth.clampedMap(float x 5)`
    pub fn clampedMap(f: f32, g: f32, h: f32, i: f32, j: f32) -> f32 {
        clampedLerp(i, j, inverseLerp(f, g, h))
    }

    /// `Mth.map(double x 5)`
    pub fn map_d(d: f64, e: f64, f: f64, g: f64, h: f64) -> f64 {
        lerp_d(inverseLerp_d(d, e, f), g, h)
    }

    /// `Mth.map(float x 5)`
    pub fn map(f: f32, g: f32, h: f32, i: f32, j: f32) -> f32 {
        lerp(inverseLerp(f, g, h), i, j)
    }

    /// `Mth.roundToward(int, int)`
    pub fn roundToward(i: i32, j: i32) -> i32 {
        positiveCeilDiv(i, j) * j
    }

    /// `Mth.positiveCeilDiv(int, int)`
    pub fn positiveCeilDiv(i: i32, j: i32) -> i32 {
        -(-i).div_euclid(j)
    }

    /// `Mth.lengthSquared(double, double)`
    pub fn lengthSquared(d: f64, e: f64) -> f64 {
        d * d + e * e
    }

    /// `Mth.length(double, double)`
    pub fn length(d: f64, e: f64) -> f64 {
        lengthSquared(d, e).sqrt()
    }

    /// `Mth.lengthSquared(double, double, double)`
    pub fn lengthSquared3(d: f64, e: f64, f: f64) -> f64 {
        d * d + e * e + f * f
    }

    /// `Mth.length(double, double, double)`
    pub fn length3(d: f64, e: f64, f: f64) -> f64 {
        lengthSquared3(d, e, f).sqrt()
    }

    /// `Mth.quantize(double, int)`
    pub fn quantize(d: f64, i: i32) -> i32 {
        floor_d(d / i as f64) * i
    }

    /// `Mth.cos(float)` (takes radians upstream, despite the name)
    pub fn cos(deg: f32) -> f32 {
        (deg as f64).cos() as f32
    }

    /// `Mth.sin(float)` (takes radians upstream, despite the name)
    pub fn sin(deg: f32) -> f32 {
        (deg as f64).sin() as f32
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.Vector3`
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// `new Vector3(x, y, z)`
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vector3 { x, y, z }
    }

    /// `Vector3.clone()`
    pub fn clone_vec(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }

    /// `Vector3.copy(Vector3)`
    pub fn copy(&mut self, vec: &Vector3) {
        self.x = vec.x;
        self.y = vec.y;
        self.z = vec.z;
    }

    /// `Vector3.add(Vector3)`
    pub fn add(&mut self, vec: &Vector3) -> &mut Self {
        self.x += vec.x;
        self.y += vec.y;
        self.z += vec.z;
        self
    }

    /// `Vector3.add(float, float, float)`
    pub fn add_xyz(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.x += x;
        self.y += y;
        self.z += z;
        self
    }

    /// `Vector3.subtract(Vector3)`
    pub fn subtract(&mut self, vec: &Vector3) -> &mut Self {
        self.x -= vec.x;
        self.y -= vec.y;
        self.z -= vec.z;
        self
    }

    /// `Vector3.div(float)`
    pub fn div(&mut self, amount: f32) -> &mut Self {
        self.x /= amount;
        self.y /= amount;
        self.z /= amount;
        self
    }

    /// `Vector3.mul(float)`
    pub fn mul(&mut self, amount: f32) -> &mut Self {
        self.x *= amount;
        self.y *= amount;
        self.z *= amount;
        self
    }

    /// `Vector3.cross(Vector3)`
    pub fn cross(&mut self, vec: &Vector3) -> &mut Self {
        let f = self.x;
        let g = self.y;
        let h = self.z;
        let i = vec.x;
        let j = vec.y;
        let k = vec.z;
        self.x = g * k - h * j;
        self.y = h * i - f * k;
        self.z = f * j - g * i;
        self
    }

    /// `Vector3.normalize()`
    pub fn normalize(&mut self) -> &mut Self {
        let f = Mth::sqrt(self.x * self.x + self.y * self.y + self.z * self.z);
        if f < 1.0E-4 {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 0.0;
        } else {
            self.x /= f;
            self.y /= f;
            self.z /= f;
        }
        self
    }

    /// `Vector3.rotateDegrees(float)`
    pub fn rotateDegrees(&mut self, mut deg: f32) -> &mut Self {
        let ox = self.x;
        let oy = self.y;
        deg = (deg as f64).to_radians() as f32;
        self.x = Mth::cos(deg) * ox - Mth::sin(deg) * oy;
        self.y = Mth::sin(deg) * ox + Mth::cos(deg) * oy;
        self
    }

    /// `Vector3.sqrMagnitude()`
    pub fn sqrMagnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.Vector4`
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    /// `new Vector4(x, y, z, w)`
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vector4 { x, y, z, w }
    }

    /// `Vector4.clone()`
    pub fn clone_vec(&self) -> Vector4 {
        Vector4::new(self.x, self.y, self.z, self.w)
    }

    /// `Vector4.toVec3()`
    pub fn toVec3(&self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.CapePoint`
pub trait CapePoint {
    /// `CapePoint.getLerpX(float)`
    fn getLerpX(&self, delta: f32) -> f32;
    /// `CapePoint.getLerpY(float)`
    fn getLerpY(&self, delta: f32) -> f32;
    /// `CapePoint.getLerpZ(float)`
    fn getLerpZ(&self, delta: f32) -> f32;

    /// `CapePoint.getLerpedPos(float)`
    fn getLerpedPos(&self, delta: f32) -> Vector3 {
        Vector3::new(self.getLerpX(delta), self.getLerpY(delta), self.getLerpZ(delta))
    }
}

/// `dev.tr7zw.waveycapes.versionless.util.CapeStick`
pub trait CapeStick {
    /// `CapeStick.getPointA()`
    fn getPointA(&self) -> &dyn CapePoint;
    /// `CapeStick.getPointB()`
    fn getPointB(&self) -> &dyn CapePoint;
    /// `CapeStick.getLength()`
    fn getLength(&self) -> f32;
}
