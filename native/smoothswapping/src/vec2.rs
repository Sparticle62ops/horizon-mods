//! Upstream `dev.shwg.smoothswapping.Vec2.java`, 1:1.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

/// `public class Vec2 implements Comparable<Vec2>`.
///
/// Upstream quirks kept verbatim, including the `v[0] =- b.v[0]` instance
/// `diff` (Java parses it as `v[0] = -b.v[0]`, a negation, not a
/// subtraction) and the degree-based `radg` trig.
#[derive(Debug, Clone, PartialEq)]
pub struct Vec2 {
    /// `public double[] v; // coordinates of the Point`
    pub v: [f64; 2],
}

/// `static double Resolution = 72;`
pub static RESOLUTION: f64 = 72.0;
/// `static int xoffset = 120;`
pub static XOFFSET: i32 = 120;
/// `static int yoffset = 120;`
pub static YOFFSET: i32 = 120;
/// `final static double radg = Math.atan(1) / 45.0;` (pi/180).
pub fn radg() -> f64 {
    1f64.atan() / 45.0
}

impl Vec2 {
    /// No-argument constructor `Vec2()` -> `this(0, 0)`.
    pub fn new() -> Self {
        Self::with_xy(0.0, 0.0)
    }

    /// Constructor `Vec2(double x, double y)`.
    pub fn with_xy(x: f64, y: f64) -> Self {
        Self { v: [0.0; 2] }.set(x, y)
    }

    /// Copy constructor `Vec2(Vec2 src)`.
    pub fn from_src(src: &Vec2) -> Self {
        Self::with_xy(src.v[0], src.v[1])
    }

    /// Constructor (angle theta) `Vec2(double theta)`: theta in degrees.
    pub fn from_angle(theta: f64) -> Self {
        let theta = theta * radg();
        Self {
            v: [0.0; 2],
        }
        .set(theta.cos(), theta.sin())
    }

    /// `public int length()`.
    pub fn length(&self) -> usize {
        self.v.len()
    }

    /// Set x and y coordinates of Point.
    pub fn set(mut self, x: f64, y: f64) -> Self {
        self.v[0] = x;
        self.v[1] = y;
        self
    }

    /// `public int[] convert()`.
    pub fn convert(&self) -> [i32; 2] {
        let mut iv = [0; 2];
        self.convert_into(&mut iv);
        iv
    }

    /// `public void convert(int[] iv)`.
    pub fn convert_into(&self, iv: &mut [i32; 2]) {
        iv[0] = (XOFFSET as f64 + self.v[0] * RESOLUTION) as i32;
        iv[1] = (YOFFSET as f64 - self.v[1] * RESOLUTION) as i32;
    }

    /// `static public Vec2 sum(Vec2 a, Vec2 b)`.
    pub fn sum(a: &Vec2, b: &Vec2) -> Vec2 {
        Vec2::with_xy(a.v[0] + b.v[0], a.v[1] + b.v[1])
    }

    /// `public Vec2 sum(Vec2 b)` (in place).
    pub fn sum_in_place(&mut self, b: &Vec2) -> &mut Self {
        self.v[0] += b.v[0];
        self.v[1] += b.v[1];
        self
    }

    /// `public Vec2 sumScalar(double b)` (in place).
    pub fn sum_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] += b;
        self.v[1] += b;
        self
    }

    /// `public Vec2 diffScalar(double b)` (in place).
    pub fn diff_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] -= b;
        self.v[1] -= b;
        self
    }

    /// `public Vec2 divideScalar(double b)` (in place).
    pub fn divide_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] /= b;
        self.v[1] /= b;
        self
    }

    /// `public Vec2 multiplyScalar(double b)` (in place).
    pub fn multiply_scalar(&mut self, b: f64) -> &mut Self {
        self.v[0] *= b;
        self.v[1] *= b;
        self
    }

    /// `public Vec2 diff(Vec2 b)`. Upstream body is `v[0] =- b.v[0];
    /// v[1] =- b.v[1]`, a negation (the assignment operator `=-`), kept
    /// 1:1; see also the static [`Vec2::diff_of`].
    pub fn diff_negate(&mut self, b: &Vec2) -> &mut Self {
        self.v[0] = -b.v[0];
        self.v[1] = -b.v[1];
        self
    }

    /// `static public Vec2 diff(Vec2 a, Vec2 b)`.
    pub fn diff_of(a: &Vec2, b: &Vec2) -> Vec2 {
        Vec2::with_xy(a.v[0] - b.v[0], a.v[1] - b.v[1])
    }

    /// Returns magnitude of 2D vector.
    pub fn magn(&self) -> f64 {
        (self.v[0] * self.v[0] + self.v[1] * self.v[1]).sqrt()
    }

    /// Multiply point by a scalar.
    pub fn scale(mut self, s: f64) -> Self {
        self.v[0] *= s;
        self.v[1] *= s;
        self
    }

    /// `public Vec2 copy()`.
    pub fn copy(&self) -> Vec2 {
        Vec2::from_src(self)
    }

    /// Offset (translate) point by the amount (tx, ty).
    pub fn translate(&mut self, tx: f64, ty: f64) -> &mut Self {
        self.v[0] += tx;
        self.v[1] += ty;
        self
    }

    /// Offset (translate) point by a Vec2.
    pub fn translate_by(&mut self, b: &Vec2) -> &mut Self {
        self.v[0] += b.v[0];
        self.v[1] += b.v[1];
        self
    }

    /// Rotate point about the origin (theta in degrees).
    pub fn rotate(&mut self, theta: f64) -> &mut Self {
        let theta = theta * radg();
        let c = theta.cos();
        let s = theta.sin();
        let t = self.v[1] * c + self.v[0] * s;
        self.v[0] = self.v[0] * c - self.v[1] * s;
        self.v[1] = t;
        self
    }

    /// `public Vec2 perp()` (in place).
    pub fn perp(&mut self) -> &mut Self {
        let t = self.v[0];
        self.v[0] = -self.v[1];
        self.v[1] = t;
        self
    }

    /// `static public Vec2 perp(Vec2 a)`.
    pub fn perp_of(a: &Vec2) -> Vec2 {
        Vec2::with_xy(-a.v[1], a.v[0])
    }

    /// `static public double perpdot(Vec2 a, Vec2 b)`.
    pub fn perpdot(a: &Vec2, b: &Vec2) -> f64 {
        a.v[0] * b.v[1] - a.v[1] * b.v[0]
    }

    /// `static public double dot(Vec2 a, Vec2 b)`.
    pub fn dot(a: &Vec2, b: &Vec2) -> f64 {
        a.v[0] * b.v[0] + a.v[1] * b.v[1]
    }

    /// `static public double distance(Vec2 a, Vec2 b)`.
    pub fn distance(a: &Vec2, b: &Vec2) -> f64 {
        Vec2::diff_of(a, b).magn()
    }

    /// Returns angle (in degrees) between two vectors.
    pub fn angle(a: &Vec2, b: &Vec2) -> f64 {
        let c = Vec2::dot(a, b);
        let s = Vec2::perpdot(a, b);
        s.atan2(c) / radg()
    }

    /// `public void setSize(double sz)`.
    pub fn set_size(&mut self, sz: f64) {
        let vnorm = self.magn();
        if vnorm == 0.0 {
            return;
        }
        let sz = sz / vnorm;
        self.v[0] *= sz;
        self.v[1] *= sz;
    }

    /// Convert the point into a String representation (`String.format
    /// ("[%.3g %.3g]", ...)` with the default locale's decimal point).
    pub fn to_display_string(&self) -> String {
        format!("[{} {}]", java_format_g(self.v[0]), java_format_g(self.v[1]))
    }
}

/// Java `String.format("%.3g", x)` (ROUND_HALF_UP, significant digits,
/// trailing zeros stripped, exponent notation above ~10^-4 and below
/// 10^precision-1).
pub fn java_format_g(x: f64) -> String {
    if x.is_nan() {
        return "NaN".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let neg = x < 0.0;
    let a = x.abs();
    let exp10 = a.log10().floor() as i32;
    let decimals = 2 - exp10;
    if decimals < 0 {
        // %g switches to scientific notation when exp >= precision.
        let mantissa = a / 10f64.powi(exp10);
        format!("{}{}e+{:02}", if neg { "-" } else { "" }, java_g_digits(mantissa), exp10)
    } else if decimals > 20 {
        // Very small magnitudes: %g switches to scientific notation.
        let mantissa = a / 10f64.powi(exp10);
        format!("{}{}e-{:02}", if neg { "-" } else { "" }, java_g_digits(mantissa), -exp10)
    } else {
        let scaled = (a * 10f64.powi(decimals) + 0.5).floor() / 10f64.powi(decimals);
        let mut s = format!("{:.*}", decimals.max(0) as usize, scaled);
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        format!("{}{}", if neg { "-" } else { "" }, s)
    }
}

fn java_g_digits(m: f64) -> String {
    let s = format!("{:.2}", m);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    s
}

/// `Comparable<Vec2>`: `compareTo` compares `v[0]` with `Double.compare`
/// semantics (used by `Collections.sort` in the config widget).
pub fn compare_to(a: &Vec2, b: &Vec2) -> std::cmp::Ordering {
    a.v[0].total_cmp(&b.v[0])
}
