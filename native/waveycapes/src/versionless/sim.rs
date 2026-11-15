//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.sim`
//! (`CapeSimulation.java`, `BasicSimulation.java`, `StickSimulation.java`,
//! `StickSimulation3d.java`, `StickSimulationDungeons.java`) plus the
//! versionless enums `CapeMovement`, `WindMode` and `CapeStyle`.
//!
//! Upstream comment kept verbatim: Java port of
//! https://www.youtube.com/watch?v=PGk0rnyTa1U by Sebastian Lague. Has some
//! changes like maximizing bends, only designed to simulate a single
//! "rope"(cape). Point 0 is the part fixed to the player.

#![allow(non_snake_case, dead_code)]

use std::any::Any;

use super::util::{CapePoint, Mth, Vector3};

/// `dev.tr7zw.waveycapes.versionless.CapeMovement`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapeMovement {
    VANILLA,
    BASIC_SIMULATION,
    BASIC_SIMULATION_3D,
    DUNGEONS,
}

impl CapeMovement {
    /// The enum constant name (Java's `name()`).
    pub fn as_str(&self) -> &'static str {
        match self {
            CapeMovement::VANILLA => "VANILLA",
            CapeMovement::BASIC_SIMULATION => "BASIC_SIMULATION",
            CapeMovement::BASIC_SIMULATION_3D => "BASIC_SIMULATION_3D",
            CapeMovement::DUNGEONS => "DUNGEONS",
        }
    }
}

/// `dev.tr7zw.waveycapes.versionless.WindMode`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WindMode {
    NONE,
    WAVES,
}

impl WindMode {
    /// The enum constant name (Java's `name()`).
    pub fn as_str(&self) -> &'static str {
        match self {
            WindMode::NONE => "NONE",
            WindMode::WAVES => "WAVES",
        }
    }
}

/// `dev.tr7zw.waveycapes.versionless.CapeStyle`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapeStyle {
    BLOCKY,
    SMOOTH,
}

impl CapeStyle {
    /// The enum constant name (Java's `name()`).
    pub fn as_str(&self) -> &'static str {
        match self {
            CapeStyle::BLOCKY => "BLOCKY",
            CapeStyle::SMOOTH => "SMOOTH",
        }
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.BasicSimulation`
pub trait BasicSimulation: Any {
    /// `BasicSimulation.simulate()`
    fn simulate(&mut self);
    /// `BasicSimulation.setGravityDirection(Vector3)`
    fn setGravityDirection(&self, gravity_direction: &Vector3);
    /// `BasicSimulation.getGravity()`
    fn getGravity(&self) -> f32;
    /// `BasicSimulation.setGravity(float)`
    fn setGravity(&mut self, gravity: f32);
    /// `BasicSimulation.isSneaking()`
    fn isSneaking(&self) -> bool;
    /// `BasicSimulation.setSneaking(boolean)`
    fn setSneaking(&mut self, sneaking: bool);
    /// `BasicSimulation.init(int)`; returns true if it was re-initialized
    fn init(&mut self, part_count: usize) -> bool;
    /// `BasicSimulation.empty()`
    fn empty(&self) -> bool;
    /// `BasicSimulation.applyMovement(Vector3)`
    fn applyMovement(&mut self, movement: &Vector3);
    /// `BasicSimulation.getPoints()`
    fn getPoints(&self) -> Vec<PointView<'_>>;
}

/// `CapeSimulation` base class. Java inheritance becomes composition: the
/// concrete simulations embed a `CapeSimulation` and forward to it.
pub struct CapeSimulation {
    /// Java `protected float[] x`
    pub x: Vec<f32>,
    /// Java `protected float[] y`
    pub y: Vec<f32>,
    /// Java `protected float[] z`
    pub z: Vec<f32>,
    /// Java `private float[] prevX`
    prev_x: Vec<f32>,
    /// Java `private float[] prevY`
    prev_y: Vec<f32>,
    /// Java `private float[] prevZ`
    prev_z: Vec<f32>,
    /// Java `private final int numIterations`
    num_iterations: usize,
    /// Java `protected float gravityX`
    pub gravity_x: std::cell::Cell<f32>,
    /// Java `protected float gravityY`
    pub gravity_y: std::cell::Cell<f32>,
    /// Java `protected float gravityZ`
    pub gravity_z: std::cell::Cell<f32>,
    /// Java `@Getter @Setter private float gravity`
    gravity: f32,
    /// Java `@Getter @Setter private boolean sneaking`
    sneaking: bool,
    /// Java `private float dirX, dirY, dirZ`
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
}

impl CapeSimulation {
    /// `protected CapeSimulation(int numIterations)`
    pub fn new(num_iterations: usize) -> Self {
        CapeSimulation {
            x: Vec::new(),
            y: Vec::new(),
            z: Vec::new(),
            prev_x: Vec::new(),
            prev_y: Vec::new(),
            prev_z: Vec::new(),
            num_iterations,
            gravity_x: std::cell::Cell::new(0.0),
            gravity_y: std::cell::Cell::new(-1.0),
            gravity_z: std::cell::Cell::new(0.0),
            gravity: 0.0,
            sneaking: false,
            dir_x: 0.0,
            dir_y: 0.0,
            dir_z: 0.0,
        }
    }

    /// `CapeSimulation.init(int)` (the `@Override` of `BasicSimulation.init`)
    pub fn init(&mut self, part_count: usize) -> bool {
        if self.x.len() == part_count {
            return false;
        }
        self.x = vec![0.0; part_count];
        self.y = vec![0.0; part_count];
        self.z = vec![0.0; part_count];
        self.prev_x = vec![0.0; part_count];
        self.prev_y = vec![0.0; part_count];
        self.prev_z = vec![0.0; part_count];
        for i in 0..part_count {
            self.y[i] = -(i as f32);
        }
        // Java builds the List<CapePoint> views here; the views borrow the
        // sim on demand in Rust (see `PointView` / `BasicSimulation::getPoints`).
        true
    }

    /// `CapeSimulation.empty()`
    pub fn empty(&self) -> bool {
        self.x.len() < 2
    }

    /// `CapeSimulation.setGravityDirection(Vector3)`
    pub fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.gravity_x.set(gravity_direction.x);
        self.gravity_y.set(gravity_direction.y);
        self.gravity_z.set(gravity_direction.z);
    }

    /// `CapeSimulation.applyMovement(Vector3)`
    pub fn applyMovement(&mut self, movement: &Vector3) {
        self.r#move(movement.x, movement.y, movement.z);
    }

    /// `protected void move(float dx, float dy, float dz)`
    pub fn r#move(&mut self, dx: f32, dy: f32, dz: f32) {
        self.prev_x[0] = self.x[0];
        self.prev_y[0] = self.y[0];
        self.prev_z[0] = self.z[0];
        self.x[0] += dx;
        self.y[0] += dy;
        self.z[0] += dz;
    }

    /// `protected void applyGravity()`
    pub fn applyGravity(&mut self) {
        let fall = self.gravity * (50.0f32 / 1000.0); // fixed timescale
        for i in 1..self.x.len() {
            self.prev_x[i] = self.x[i];
            self.prev_y[i] = self.y[i];
            self.prev_z[i] = self.z[i];
            self.x[i] += self.gravity_x.get() * fall;
            self.y[i] += self.gravity_y.get() * fall;
            self.z[i] += self.gravity_z.get() * fall;
        }
    }

    /// `protected void preventClipping()`
    pub fn preventClipping(&mut self) {
        // prevent the cape from clipping into the player
        for i in 1..self.x.len() {
            if self.x[i] - self.x[0] > 0.0 {
                self.x[i] = self.x[0];
            }
            let max_z = ((i as f32 / self.x.len() as f32) * (i as f32 / self.x.len() as f32)) * 5.0;
            let offset = self.z[0] - self.z[i];
            if offset > max_z {
                self.z[i] = self.z[0] - max_z;
            }
            if offset < -max_z {
                self.z[i] = self.z[0] + max_z;
            }
        }
    }

    /// `protected void applyMotion()`
    pub fn applyMotion(&mut self) {
        // move into correct direction
        for _ in 0..self.num_iterations {
            for a in (0..self.x.len().saturating_sub(1)).rev() {
                self.direction(a, a + 1);
                self.separate(a, a + 1);
            }
        }
    }

    /// `protected void limitLength()`
    pub fn limitLength(&mut self) {
        // fix in the position/length, this prevents it from acting like a
        // spring/stretchy
        for a in 0..self.x.len().saturating_sub(1) {
            self.direction(a, a + 1);
            self.x[a + 1] = self.x[a] - self.dir_x;
            self.y[a + 1] = self.y[a] - self.dir_y;
            self.z[a + 1] = self.z[a] - self.dir_z;
        }
    }

    /// `protected void preventSelfClipping(int maxRuns)`
    pub fn preventSelfClipping(&mut self, max_runs: i32) {
        let mut clipped;
        let mut runs = 0;
        loop {
            clipped = false;
            // check the cape parts against each other. Bad implementation
            for a in 0..self.x.len() {
                for b in a + 1..self.x.len() {
                    let dx = self.x[a] - self.x[b];
                    let dy = self.y[a] - self.y[b];
                    let dz = self.z[a] - self.z[b];
                    if dx * dx + dy * dy + dz * dz < 0.99 {
                        clipped = true;
                        runs += 1;
                        self.direction(a, b);
                        self.separate(a, b);
                    }
                }
            }
            if !(clipped && runs < max_runs) {
                break;
            }
        }
    }

    /// `protected void preventHardBend(int i, float maxBend)`
    /// Doesnt work like it should at all, but it prevents some folding into
    /// itself, so it stays for now
    pub fn preventHardBend(&mut self, i: usize, max_bend: f32) {
        let abx = self.x[i - 1] - self.x[i];
        let aby = self.y[i - 1] - self.y[i];
        let cbx = self.x[i - 1] - self.x[i + 1];
        let cby = self.y[i - 1] - self.y[i + 1];

        let dot = abx * cbx + aby * cby; // dot product
        let cross = abx * cby - aby * cbx; // cross product

        let angle = Mth::atan2(cross as f64, dot as f64) * 180.0 / std::f64::consts::PI;
        if angle < -(max_bend as f64) {
            self.swing(i, -max_bend * 2.0);
        }
        if angle > max_bend as f64 {
            self.swing(i, max_bend * 2.0);
        }
    }

    /// `private void swing(int i, float deg)`
    fn swing(&mut self, i: usize, deg: f32) {
        let ox = self.x[i] - self.x[i - 1];
        let oy = self.y[i] - self.y[i - 1];
        let oz = self.z[i] - self.z[i - 1];
        let deg = (deg as f64).to_radians() as f32;
        self.x[i + 1] = Mth::cos(deg) * ox - Mth::sin(deg) * oy + self.x[i];
        self.y[i + 1] = Mth::sin(deg) * ox + Mth::cos(deg) * oy + self.y[i];
        self.z[i + 1] = oz + self.z[i];
    }

    /// `private void separate(int a, int b)`
    fn separate(&mut self, a: usize, b: usize) {
        let centre_x = (self.x[a] + self.x[b]) / 2.0;
        let centre_y = (self.y[a] + self.y[b]) / 2.0;
        let centre_z = (self.z[a] + self.z[b]) / 2.0;
        if a != 0 {
            self.x[a] = centre_x + self.dir_x / 2.0;
            self.y[a] = centre_y + self.dir_y / 2.0;
            self.z[a] = centre_z + self.dir_z / 2.0;
        }
        self.x[b] = centre_x - self.dir_x / 2.0;
        self.y[b] = centre_y - self.dir_y / 2.0;
        self.z[b] = centre_z - self.dir_z / 2.0;
    }

    /// `private void direction(int a, int b)`
    fn direction(&mut self, a: usize, b: usize) {
        self.dir_x = self.x[a] - self.x[b];
        self.dir_y = self.y[a] - self.y[b];
        self.dir_z = self.z[a] - self.z[b];
        let f = Mth::sqrt(self.dir_x * self.dir_x + self.dir_y * self.dir_y + self.dir_z * self.dir_z);
        if f < 1.0E-4 {
            self.dir_x = 0.0;
            self.dir_y = 0.0;
            self.dir_z = 0.0;
        } else {
            self.dir_x /= f;
            self.dir_y /= f;
            self.dir_z /= f;
        }
    }

    /// `CapeSimulation.getGravity()` (lombok `@Getter`)
    pub fn getGravity(&self) -> f32 {
        self.gravity
    }

    /// `CapeSimulation.setGravity(float)` (lombok `@Setter`)
    pub fn setGravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    /// `CapeSimulation.isSneaking()` (lombok `@Getter`)
    pub fn isSneaking(&self) -> bool {
        self.sneaking
    }

    /// `CapeSimulation.setSneaking(boolean)` (lombok `@Setter`)
    pub fn setSneaking(&mut self, sneaking: bool) {
        self.sneaking = sneaking;
    }

    /// `Point.getLerpX(float)` on the inner `Point` view for `index`
    pub fn pointLerpX(&self, index: usize, delta: f32) -> f32 {
        Mth::lerp(delta, self.prev_x[index], self.x[index])
    }

    /// `Point.getLerpY(float)`
    pub fn pointLerpY(&self, index: usize, delta: f32) -> f32 {
        Mth::lerp(delta, self.prev_y[index], self.y[index])
    }

    /// `Point.getLerpZ(float)`
    pub fn pointLerpZ(&self, index: usize, delta: f32) -> f32 {
        Mth::lerp(delta, self.prev_z[index], self.z[index])
    }
}

impl BasicSimulation for CapeSimulation {
    fn simulate(&mut self) {
        // abstract upstream; concrete sims override
    }

    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        CapeSimulation::setGravityDirection(self, gravity_direction)
    }

    fn getGravity(&self) -> f32 {
        self.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.sneaking = sneaking;
    }

    fn init(&mut self, part_count: usize) -> bool {
        CapeSimulation::init(self, part_count)
    }

    fn empty(&self) -> bool {
        self.x.len() < 2
    }

    fn applyMovement(&mut self, movement: &Vector3) {
        CapeSimulation::applyMovement(self, movement)
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        (0..self.x.len()).map(|index| PointView { sim: self, index }).collect()
    }
}

/// The inner `private final class Point implements CapePoint`, as a borrow of
/// its owning simulation.
pub struct PointView<'a> {
    sim: &'a CapeSimulation,
    index: usize,
}

impl CapePoint for PointView<'_> {
    fn getLerpX(&self, delta: f32) -> f32 {
        self.sim.pointLerpX(self.index, delta)
    }

    fn getLerpY(&self, delta: f32) -> f32 {
        self.sim.pointLerpY(self.index, delta)
    }

    fn getLerpZ(&self, delta: f32) -> f32 {
        self.sim.pointLerpZ(self.index, delta)
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.StickSimulation`
pub struct StickSimulation {
    /// Java `extends CapeSimulation`
    pub base: CapeSimulation,
}

impl StickSimulation {
    /// `private static final float MAX_BEND = 5`
    pub const MAX_BEND: f32 = 5.0;

    /// `public StickSimulation() { super(3); }`
    pub fn new() -> Self {
        StickSimulation { base: CapeSimulation::new(3) }
    }

    /// `private void preventHardBends()`
    /// Doesnt work like it should at all, but it prevents some folding into
    /// itself, so it stays for now
    fn preventHardBends(&mut self) {
        let len = self.base.x.len();
        for i in (1..len.saturating_sub(1)).rev() {
            let mut angle = ((self.base.y[i + 1] - self.base.y[i]) as f64).atan2((self.base.x[i + 1] - self.base.x[i]) as f64)
                - ((self.base.y[i - 1] - self.base.y[i]) as f64).atan2((self.base.x[i - 1] - self.base.x[i]) as f64);
            angle *= 57.2958;
            if angle > 360.0 {
                angle -= 360.0;
            }
            if angle < -360.0 {
                angle += 360.0;
            }
            let abs = angle.abs();
            if abs < 180.0 - Self::MAX_BEND as f64 {
                self.replace(i, angle, 180.0 - Self::MAX_BEND as f64 + 1.0);
            }
            if abs > 180.0 + Self::MAX_BEND as f64 {
                self.replace(i, angle, 180.0 + Self::MAX_BEND as f64 - 1.0);
            }
        }
    }

    /// `private void replace(int i, double angle, double target)`
    fn replace(&mut self, i: usize, angle: f64, target: f64) {
        let mut theta = target / 57.2958;
        let px = self.base.x[i - 1] - self.base.x[i];
        let py = self.base.y[i - 1] - self.base.y[i];
        if angle < 0.0 {
            theta *= -1.0;
        }
        let cs = theta.cos();
        let sn = theta.sin();
        self.base.x[i + 1] = ((px as f64 * cs) - (py as f64 * sn) + self.base.x[i] as f64) as f32;
        self.base.y[i + 1] = ((px as f64 * sn) + (py as f64 * cs) + self.base.y[i] as f64) as f32;
    }
}

impl Default for StickSimulation {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicSimulation for StickSimulation {
    fn simulate(&mut self) {
        self.base.applyGravity();
        self.base.preventClipping();
        self.preventHardBends();
        self.base.applyMotion();
        self.base.limitLength();
    }

    /// `@Override setGravityDirection`: only gravityX and gravityY
    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.base.gravity_x.set(gravity_direction.x);
        self.base.gravity_y.set(gravity_direction.y);
    }

    /// `@Override applyMovement`: `move(movement.x, movement.y, 0)`
    fn applyMovement(&mut self, movement: &Vector3) {
        self.base.r#move(movement.x, movement.y, 0.0);
    }

    fn getGravity(&self) -> f32 {
        self.base.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.base.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.base.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.base.sneaking = sneaking;
    }

    fn init(&mut self, part_count: usize) -> bool {
        self.base.init(part_count)
    }

    fn empty(&self) -> bool {
        self.base.empty()
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        self.base.getPoints()
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.StickSimulation3d`
pub struct StickSimulation3d {
    /// Java `extends CapeSimulation`
    pub base: CapeSimulation,
}

impl StickSimulation3d {
    /// `private static final float MAX_BEND = 20`
    pub const MAX_BEND: f32 = 20.0;

    /// `public StickSimulation3d() { super(3); }`
    pub fn new() -> Self {
        StickSimulation3d { base: CapeSimulation::new(3) }
    }
}

impl Default for StickSimulation3d {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicSimulation for StickSimulation3d {
    /// `@Override init`: super.init then `x[i] = -i`
    fn init(&mut self, part_count: usize) -> bool {
        if !self.base.init(part_count) {
            return false;
        }
        for i in 0..part_count {
            self.base.x[i] = -(i as f32);
        }
        true
    }

    /// `@Override simulate`
    fn simulate(&mut self) {
        self.base.applyGravity();
        self.base.preventClipping();
        self.base.preventSelfClipping(32);
        self.base.applyMotion();
        self.base.preventSelfClipping(32);
        for i in 1..self.base.x.len().saturating_sub(2) {
            self.base.preventHardBend(i, Self::MAX_BEND);
        }
        self.base.limitLength();
    }

    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.base.setGravityDirection(gravity_direction)
    }

    fn getGravity(&self) -> f32 {
        self.base.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.base.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.base.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.base.sneaking = sneaking;
    }

    fn empty(&self) -> bool {
        self.base.empty()
    }

    /// `@Override applyMovement` (inherited upstream)
    fn applyMovement(&mut self, movement: &Vector3) {
        self.base.applyMovement(movement)
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        self.base.getPoints()
    }
}

/// `dev.tr7zw.waveycapes.versionless.sim.StickSimulationDungeons`
pub struct StickSimulationDungeons {
    /// Java `extends CapeSimulation`
    pub base: CapeSimulation,
}

impl StickSimulationDungeons {
    /// `private static final float MAX_BEND = 20`
    pub const MAX_BEND: f32 = 20.0;

    /// `public StickSimulationDungeons() { super(30); }`
    pub fn new() -> Self {
        StickSimulationDungeons { base: CapeSimulation::new(30) }
    }
}

impl Default for StickSimulationDungeons {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicSimulation for StickSimulationDungeons {
    /// `@Override simulate`
    fn simulate(&mut self) {
        self.base.applyGravity();
        self.base.preventClipping();
        self.base.applyMotion();
        for i in 1..self.base.x.len().saturating_sub(1) {
            let max_bend = if i == self.base.x.len() / 2 { Self::MAX_BEND } else { 0.0 };
            self.base.preventHardBend(i, max_bend);
        }
        self.base.preventSelfClipping(10);
        self.base.limitLength();
    }

    fn setGravityDirection(&self, gravity_direction: &Vector3) {
        self.base.setGravityDirection(gravity_direction)
    }

    fn getGravity(&self) -> f32 {
        self.base.gravity
    }

    fn setGravity(&mut self, gravity: f32) {
        self.base.gravity = gravity;
    }

    fn isSneaking(&self) -> bool {
        self.base.sneaking
    }

    fn setSneaking(&mut self, sneaking: bool) {
        self.base.sneaking = sneaking;
    }

    fn init(&mut self, part_count: usize) -> bool {
        self.base.init(part_count)
    }

    fn empty(&self) -> bool {
        self.base.empty()
    }

    fn applyMovement(&mut self, movement: &Vector3) {
        self.base.applyMovement(movement)
    }

    fn getPoints(&self) -> Vec<PointView<'_>> {
        self.base.getPoints()
    }
}
