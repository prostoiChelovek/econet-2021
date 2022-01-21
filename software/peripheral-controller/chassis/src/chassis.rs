use motor::{SetSpeed, GetSpeed};
use encoder::{GetPosition, Update};
use servo::{SetPosition, CheckTargetReached};
use gyro::GetRotation;

pub trait ChassisMotor = SetSpeed + GetSpeed + GetPosition + SetPosition + CheckTargetReached + Update;

#[derive(Debug, Default)]
pub struct ChassisSpeed {
    pub linear: f32,
    pub angular: f32
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ChassisPosition {
    pub linear: (f32, f32),
    pub angular: f32
}

#[derive(Debug, Clone, Copy)]
pub enum AtomicMovement {
    Linear(f32),
    Angular(f32)
}

pub trait MoveAtomic {
    fn move_atomic(&mut self, movement: AtomicMovement);
}

struct ChassisMovement {
    pub movement: AtomicMovement,
    pub prev_pos: (f32, f32),
}

pub struct Chassis<L, R, G>
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation
{
    wheels_distance: f32,

    speed: ChassisSpeed,

    current_movement: Option<ChassisMovement>,
    position: ChassisPosition,

    left: L,
    right: R,

    rotation: G
}

impl<L, R, G> Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    f32: From<<L as GetPosition>::Position>,
    f32: From<<R as GetPosition>::Position>
{
    pub fn new(left: L, right: R, rotation: G, wheels_distance_cm: f32) -> Self {
        Self {
            wheels_distance: wheels_distance_cm,

            speed: ChassisSpeed::default(),

            current_movement: None,
            position: ChassisPosition::default(),

            left,
            right,

            rotation
        }
    }

    fn get_wheel_positions(&self)
        -> (f32, f32) {
        (self.left.get_position().into(), self.right.get_position().into())
    }
}

impl<L, R, G> Update for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    f32: From<<L as GetPosition>::Position>,
    f32: From<<R as GetPosition>::Position>
{
    fn update(&mut self, time_delta_seconds: f32) {
        self.left.update(time_delta_seconds);
        self.right.update(time_delta_seconds);

        let wheel_positions = self.get_wheel_positions();

        if let Some(ref mut movement) = self.current_movement {
            let wheels_distance = (
                (wheel_positions.0 - movement.prev_pos.0), (wheel_positions.1 - movement.prev_pos.1)
            );
            movement.prev_pos = wheel_positions;

            match movement.movement {
                AtomicMovement::Linear(_) => {
                    self.position.linear.0 += libm::cosf(self.position.angular) * wheels_distance.0;
                    self.position.linear.1 += libm::sinf(self.position.angular) * wheels_distance.1;
                },
                AtomicMovement::Angular(_) => {
                    // TODO: it assumes that wheels have travelled an equal distance,
                    //       but it is probably incorrect
                    self.position.angular += (wheels_distance.0 * 2.0) / self.wheels_distance;
                },
            }

            if self.is_target_reached() {
                self.current_movement = None;
            }
        }
    }
}

impl<L, R, G> SetSpeed for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation
{
    type Speed = ChassisSpeed;

    fn set_speed(&mut self, speed: Self::Speed) {
        self.speed = speed;
    }
}

impl<L, R, G> GetSpeed for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    <L as GetSpeed>::Speed: Into<f32>,
    <R as GetSpeed>::Speed: Into<f32>
{
    type Speed = (f32, f32);

    fn get_speed(&mut self) -> Self::Speed {
        (self.left.get_speed().into(), self.right.get_speed().into())
    }
}

impl<L, R, G> GetPosition for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation
{
    type Position = ChassisPosition;

    fn get_position(&self) -> Self::Position {
        self.position
    }
}

impl<L, R, G> CheckTargetReached for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    f32: From<<L as GetPosition>::Position>,
    f32: From<<R as GetPosition>::Position>
{
    fn is_target_reached(&self) -> bool {
        self.left.is_target_reached() && self.right.is_target_reached()
    }
}

impl<L, R, G> MoveAtomic for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    f32: From<<L as GetPosition>::Position>,
    f32: From<<R as GetPosition>::Position>,
    <L as SetPosition>::Position: From<f32>,
    <R as SetPosition>::Position: From<f32>
{
    fn move_atomic(&mut self, movement: AtomicMovement) {
        let wheel_positions = self.get_wheel_positions();

        self.current_movement = Some(ChassisMovement {
            movement: movement.clone(),
            prev_pos: wheel_positions.clone()
        });

        match movement {
            AtomicMovement::Linear(distance) => {
                self.left.set_position((wheel_positions.0 + distance).into());
                self.right.set_position((wheel_positions.1 + distance).into());
            },
            AtomicMovement::Angular(angle) => {
                let increment = angle.to_radians() * (self.wheels_distance / 2.0);
                self.left.set_position((wheel_positions.0 + increment).into());
                self.right.set_position((wheel_positions.1 - increment).into());
            }
        };
    }
}

