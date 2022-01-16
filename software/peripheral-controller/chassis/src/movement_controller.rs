use num_traits::float::FloatCore;

use crate::chassis::{MoveAtomic, ChassisPosition, AtomicMovement};

use servo::CheckTargetReached;
use encoder::{Update, GetPosition};

// TODO: this associated type specification should probably not be here
pub trait MovementControlled = MoveAtomic + Update + GetPosition<Position = ChassisPosition> + CheckTargetReached;

pub trait MoveRelative {
    // TODO: it is semantically wrong to use ChassisPosition here
    fn move_relative(&mut self, movement: ChassisPosition);
}

#[derive(Debug, Clone, Copy)]
enum MovementStage {
    InitialRotarion,
    Translation,
    FinalRotation
}

struct Movement {
    pub movement: ChassisPosition,
    pub stage: MovementStage
}

impl Movement {
    pub fn new(movement: ChassisPosition) -> Self {
        Self {
            movement,
            stage: MovementStage::InitialRotarion,
        }
    }
}

pub struct MovementController<T: MovementControlled> {
    atomic: T,
    movement: Option<Movement>
}

impl<T: MovementControlled> MovementController<T> {
    pub fn new(controlled: T) -> Self {
        Self {
            atomic: controlled,
            movement: None
        }
    }

    fn next_stage(&mut self) {
        let mut movement = self.movement.as_mut().unwrap();
        match movement.stage.clone() {
            MovementStage::InitialRotarion => {
                movement.stage = MovementStage::Translation;
            },
            MovementStage::Translation => {
                movement.stage = MovementStage::FinalRotation;
            },
            MovementStage::FinalRotation => {
                self.movement = None;
            }
        }
    }

    fn start_stage(&mut self) {
        let movement = self.movement.as_mut().unwrap();
        let (linear, angular) = (movement.movement.linear, movement.movement.angular);
        let current_pos = self.atomic.get_position();

        match movement.stage {
            MovementStage::InitialRotarion => {
                let angle = libm::atan2f(
                    linear.1, linear.0,
                    );

                self.atomic.move_atomic(AtomicMovement::Angular(angle.to_degrees() - current_pos.angular));
            },
            MovementStage::Translation => {
                let distance = libm::sqrtf(linear.0.powi(2) + linear.1.powi(2));

                self.atomic.move_atomic(AtomicMovement::Linear(distance));
            },
            MovementStage::FinalRotation => {
                self.atomic.move_atomic(AtomicMovement::Angular(angular));
            }
        }
    }
}

impl<T: MovementControlled> MoveRelative for MovementController<T> {
    fn move_relative(&mut self, movement: ChassisPosition) {
        self.movement = Some(Movement::new(movement))
    }
}

impl<T: MovementControlled> Update for MovementController<T> {
    fn update(&mut self, time_delta_seconds: f32) {
        self.atomic.update(time_delta_seconds);

        if self.movement.is_some() {
            if self.atomic.is_target_reached() {
                self.next_stage();
                self.start_stage();
            }
        }
    }
}

impl<T: MovementControlled> CheckTargetReached for MovementController<T> {
    fn is_target_reached(&self) -> bool {
        self.movement.is_none() && self.atomic.is_target_reached()
    }
}

