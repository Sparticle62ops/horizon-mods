//! 1:1 translation of `dev.tr7zw.waveycapes.versionless.CapeHolder`.

#![allow(non_snake_case, dead_code)]

use std::any::{Any, TypeId};

use uuid::Uuid;

use crate::versionless::config::Config;
use crate::versionless::modbase::{apply_instance_animations, config_capeMovement};
use crate::versionless::nms::MinecraftPlayer;
use crate::versionless::sim::{
    BasicSimulation, CapeMovement, StickSimulation, StickSimulation3d, StickSimulationDungeons,
};
use crate::versionless::util::{Mth, Vector3};

/// `dev.tr7zw.waveycapes.versionless.CapeHolder`
///
/// Java `default` methods become Rust default trait methods. The class-identity
/// checks use `TypeId` (`sim.getClass() != ...`).
pub trait CapeHolder {
    /// `BasicSimulation getSimulation()`
    fn getSimulation(&self) -> Option<&dyn BasicSimulation>;

    /// Borrow-shaped accessor for the same field (Rust needs `&mut` for the
    /// default methods; the Java side reuses its local variable).
    fn getSimulation_mut(&mut self) -> Option<&mut dyn BasicSimulation>;

    /// `Vector3 getLastPlayerAnimatorPosition()`
    fn getLastPlayerAnimatorPosition(&self) -> Option<&Vector3>;

    /// `void setLastPlayerAnimatorPosition(Vector3 pos)`
    fn setLastPlayerAnimatorPosition(&mut self, pos: Vector3);

    /// `void setSimulation(BasicSimulation sim)`
    fn setSimulation(&mut self, sim: Option<Box<dyn BasicSimulation>>);

    /// `UUID getWCUUID()`
    fn getWCUUID(&self) -> Option<Uuid>;

    /// `void setDirty()`
    fn setDirty(&self);

    /// `void markCapeRendered()`
    fn markCapeRendered(&self);

    /// `void setGravityVectorRequest(boolean canUpdate)`
    fn setGravityVectorRequest(&self, can_update: bool);

    /// `boolean canUpdateGravityVector()`
    fn canUpdateGravityVector(&self) -> bool;

    /// `default void updateSimulation(int partCount)`
    fn updateSimulation(&mut self, part_count: usize) {
        let incorrect = match self.getSimulation() {
            Some(simulation) => self.incorrectSimulation(simulation),
            None => true, // Java: simulation == null || incorrectSimulation(simulation)
        };
        if incorrect {
            let simulation = self.createSimulation();
            self.setSimulation(simulation);
        }
        // Java: if (simulation == null) return;
        let Some(simulation) = self.getSimulation_mut() else {
            return;
        };
        if simulation.init(part_count) {
            self.setDirty();
        }
    }

    /// `default boolean incorrectSimulation(BasicSimulation sim)`
    fn incorrectSimulation(&self, sim: &dyn BasicSimulation) -> bool {
        let style = config_capeMovement();
        let sim_any: &dyn Any = sim;
        if style == CapeMovement::BASIC_SIMULATION && sim_any.type_id() != TypeId::of::<StickSimulation>() {
            return true;
        } else if style == CapeMovement::BASIC_SIMULATION_3D
            && sim_any.type_id() != TypeId::of::<StickSimulation3d>()
        {
            return true;
        } else if style == CapeMovement::DUNGEONS && sim_any.type_id() != TypeId::of::<StickSimulationDungeons>() {
            return true;
        }
        false
    }

    /// `default BasicSimulation createSimulation()`
    fn createSimulation(&self) -> Option<Box<dyn BasicSimulation>> {
        let style = config_capeMovement();
        if style == CapeMovement::BASIC_SIMULATION {
            return Some(Box::new(StickSimulation::new()));
        }
        if style == CapeMovement::BASIC_SIMULATION_3D {
            return Some(Box::new(StickSimulation3d::new()));
        }
        if style == CapeMovement::DUNGEONS {
            return Some(Box::new(StickSimulationDungeons::new()));
        }
        None // Java returns null (VANILLA)
    }

    /// `default void simulate(MinecraftPlayer abstractClientPlayer)`
    fn simulate(&mut self, abstract_client_player: &dyn MinecraftPlayer) {
        let empty = match self.getSimulation() {
            Some(simulation) => simulation.empty(),
            None => true, // Java: simulation == null || simulation.empty()
        };
        if empty {
            return; // no cape, nothing to update
        }

        let d = abstract_client_player.getXCloak() - abstract_client_player.getX();
        let m = abstract_client_player.getZCloak() - abstract_client_player.getZ();
        let n = abstract_client_player.getYBodyRotO() + abstract_client_player.getYBodyRot()
            - abstract_client_player.getYBodyRotO();
        let o = Mth::sin(n * 0.017_453_292_f32) as f64;
        let p = -(Mth::cos(n * 0.017_453_292_f32) as f64);
        let mut height_mul = Config::heightMultiplier() as f32;
        let strave_mul = Config::straveMultiplier() as f32;
        if abstract_client_player.isUnderWater() {
            height_mul *= 2.0; // let the cape have more drag than the player underwater
        }
        // gives the cape a small swing when jumping/falling to not clip with
        // itself/simulate some air getting under it
        let fall_hack = Mth::clamp_d(
            (abstract_client_player.getYo() - abstract_client_player.getY()) * 10.0,
            0.0,
            1.0,
        );
        if abstract_client_player.isUnderWater() {
            self.with_simulation(|simulation| {
                simulation.setGravity(Config::gravity() as f32 / 10.0f32);
            });
        } else {
            self.with_simulation(|simulation| {
                simulation.setGravity(Config::gravity() as f32);
            });
        }

        let mut gravity = Vector3::new(0.0, -1.0, 0.0);
        let yaw = (-abstract_client_player.getYRot() as f64).to_radians() as f32;
        let strave = Mth::cos(yaw) * (abstract_client_player.getX() - abstract_client_player.getXo()) as f32
            - Mth::sin(yaw) * (abstract_client_player.getZ() - abstract_client_player.getZo()) as f32;
        let change_x = (d * o + m * p)
            + fall_hack
            + if abstract_client_player.isCrouching() && !self.simulation_isSneaking() {
                3.0
            } else {
                0.0
            };
        let change_y = ((abstract_client_player.getY() - abstract_client_player.getYo()) * height_mul as f64)
            + if abstract_client_player.isCrouching() && !self.simulation_isSneaking() {
                1.0
            } else {
                0.0
            };
        let change_z = -strave * strave_mul;
        self.with_simulation(|simulation| {
            simulation.setSneaking(abstract_client_player.isCrouching());
        });
        let mut change = Vector3::new(change_x as f32, change_y as f32, change_z);
        if !Config::computeGravityVector() {
            if abstract_client_player.isVisuallySwimming() {
                let mut rotation = abstract_client_player.getXRot(); // -90 = swimming up, 0 = straight, 90 = down
                // the simulation has the body as reference, so if the player is swimming
                // straight down, gravity needs to point up(the cape should move into the
                // direction of the head, not the feet)
                // offset the rotation to swimming up doesn't rotate the vector at all
                rotation += 90.0;
                // apply rotation
                gravity.rotateDegrees(rotation);

                change.rotateDegrees(rotation);
            }
            self.with_simulation(|simulation| {
                simulation.setGravityDirection(&gravity);
            });
        }

        let change = apply_instance_animations(abstract_client_player, change);
        self.with_simulation(|simulation| {
            simulation.applyMovement(&change);
            simulation.simulate();
        });
    }

    /// Borrow-shaped read of `isSneaking()` on the current simulation (Java
    /// calls `simulation.isSneaking()` on its local).
    fn simulation_isSneaking(&self) -> bool {
        self.getSimulation().map_or(false, |simulation| simulation.isSneaking())
    }

    /// Apply `f` to the current simulation (Java mutates its local `simulation`
    /// variable; Rust goes back through the holder's accessor).
    fn with_simulation(&mut self, f: impl FnOnce(&mut dyn BasicSimulation)) {
        if let Some(simulation) = self.getSimulation_mut() {
            f(simulation);
        }
    }
}
