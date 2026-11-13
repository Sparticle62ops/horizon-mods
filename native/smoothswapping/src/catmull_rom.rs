//! Upstream `config/CatmullRomWidget.java`, math half 1:1:
//! `splinesFromPoints`, `getProgress`, `getSegmentForT`, and the
//! `CatmullRomSpline`/`Segment` inner classes. The widget UI half
//! (extractWidgetRenderState, mouse handling, tooltip state) lands with
//! the config-screen batch.
/*
 * Copyright (c) 2026 Schauweg. LGPL-3.0-only; see the crate root.
 */

use crate::swap_util;
use crate::vec2::Vec2;

/// `static float tension = 0f`.
pub const TENSION: f32 = 0.0;
/// `static float alpha = 0f`.
pub const ALPHA: f32 = 0.0;

/// `public static class CatmullRomSpline`.
#[derive(Debug, Clone)]
pub struct CatmullRomSpline {
    segment: Segment,
    /// `float x` (package-private).
    pub x: f32,
    /// `float oldX` (package-private).
    pub old_x: f32,
}

/// Private `record Segment(Vec2 a, Vec2 b, Vec2 c, Vec2 d)`.
#[derive(Debug, Clone)]
pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
    pub d: Vec2,
}

impl Segment {
    /// `getPoint(double t)`.
    pub fn get_point(&self, t: f64) -> Vec2 {
        let mut a = self.a.copy();
        a.multiply_scalar(t * t * t);
        let mut b = self.b.copy();
        b.multiply_scalar(t * t);
        let mut c = self.c.copy();
        c.multiply_scalar(t);
        let bc = Vec2::sum(&b, &c);
        let abc = Vec2::sum(&a, &bc);
        Vec2::sum(&abc, &self.d)
    }
}

impl CatmullRomSpline {
    /// `new CatmullRomSpline(Vec2 p0, Vec2 p1, Vec2 p2, Vec2 p3)`.
    pub fn new(p0: &Vec2, p1: &Vec2, p2: &Vec2, p3: &Vec2) -> Self {
        let x = p2.v[0] as f32;
        let old_x = p1.v[0] as f32;

        let t01 = Vec2::distance(p0, p1).powf(ALPHA as f64);
        let t12 = Vec2::distance(p1, p2).powf(ALPHA as f64);
        let t23 = Vec2::distance(p2, p3).powf(ALPHA as f64);

        // m1 = Vec2.sum(Vec2.diff(p2, p1), (Vec2.diff(
        //     Vec2.diff(p1, p0).divideScalar(t01),
        //     Vec2.diff(p2, p0).divideScalar(t01 + t12))).multiplyScalar(t12))
        //     .multiplyScalar(1.0f - tension);
        let mut d10 = Vec2::diff_of(p1, p0);
        d10.divide_scalar(t01);
        let mut d20 = Vec2::diff_of(p2, p0);
        d20.divide_scalar(t01 + t12);
        let mut inner = Vec2::diff_of(&d10, &d20);
        inner.multiply_scalar(t12);
        let mut m1 = Vec2::diff_of(p2, p1);
        m1.sum_in_place(&inner);
        m1.multiply_scalar((1.0 - TENSION) as f64);

        // m2 = Vec2.sum(Vec2.diff(p2, p1), (Vec2.diff(
        //     Vec2.diff(p3, p2).divideScalar(t23),
        //     Vec2.diff(p3, p1).divideScalar(t12 + t23))).multiplyScalar(t12))
        //     .multiplyScalar(1.0f - tension);
        let mut d32 = Vec2::diff_of(p3, p2);
        d32.divide_scalar(t23);
        let mut d31 = Vec2::diff_of(p3, p1);
        d31.divide_scalar(t12 + t23);
        let mut inner2 = Vec2::diff_of(&d32, &d31);
        inner2.multiply_scalar(t12);
        let mut m2 = Vec2::diff_of(p2, p1);
        m2.sum_in_place(&inner2);
        m2.multiply_scalar((1.0 - TENSION) as f64);

        // segment = new Segment(
        //     Vec2.sum(Vec2.diff(p1, p2).multiplyScalar(2.0d), Vec2.sum(m1, m2)),
        //     Vec2.diff(Vec2.diff(Vec2.diff(Vec2.diff(p1, p2).multiplyScalar(-3), m1), m1), m2),
        //     m1, p1);
        let mut twice = Vec2::diff_of(p1, p2);
        twice.multiply_scalar(2.0);
        let m12 = Vec2::sum(&m1, &m2);
        let a = Vec2::sum(&twice, &m12);

        let mut neg_three = Vec2::diff_of(p1, p2);
        neg_three.multiply_scalar(-3.0);
        let s1 = Vec2::diff_of(&neg_three, &m1);
        let s2 = Vec2::diff_of(&s1, &m1);
        let b = Vec2::diff_of(&s2, &m2);

        let c = m1;
        let d = p1.copy();

        Self { segment: Segment { a, b, c, d }, x, old_x }
    }

    /// `private Segment getSegment()` (crate-visible for getProgress and
    /// the widget batch).
    pub fn segment(&self) -> &Segment {
        &self.segment
    }
}

/// `public static List<CatmullRomSpline> splinesFromPoints(List<Vec2> points)`.
pub fn splines_from_points(points: &[Vec2]) -> Vec<CatmullRomSpline> {
    let mut splines = Vec::new();
    // Java: for (int i = 1; i < points.size() - 2; i++) — an int bound,
    // so short lists just produce no iterations.
    let end = (points.len() as isize - 2).max(0) as usize;
    for i in 1..end {
        let p0 = &points[i - 1];
        let p1 = &points[i];
        let p2 = &points[i + 1];
        let p3 = &points[i + 2];
        splines.push(CatmullRomSpline::new(p0, p1, p2, p3));
    }
    splines
}

/// `public static double getProgress(double t, List<CatmullRomSpline> segments)`.
pub fn get_progress(t: f64, segments: &[CatmullRomSpline]) -> f64 {
    let current_segment = get_segment_for_t(t, segments);
    // SwapUtil.map(t, currentSegment.oldX, currentSegment.x, 1, 0)
    let progress = swap_util::map(t, current_segment.old_x as f64, current_segment.x as f64, 1.0, 0.0);
    current_segment.segment().get_point(progress).v[1]
}

/// `private static CatmullRomSpline getSegmentForT(double t,
/// List<CatmullRomSpline> segments)`. Falls back to `segments.get(0)`
/// (which throws when empty, as upstream does).
fn get_segment_for_t(t: f64, segments: &[CatmullRomSpline]) -> &CatmullRomSpline {
    for spline in segments {
        if t >= spline.old_x as f64 && t < spline.x as f64 {
            return spline;
        }
    }
    &segments[0]
}
