use pid::Pid;

use motor::{SetSpeed, GetSpeed};
use encoder::{GetPosition, Update};
use servo::{SetPosition, CheckTargetReached};
use gyro::GetRotation;

use rtt_target::rprintln;

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
    pub(crate) start_rotation: f32
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

    rotation: G,

    rotation_limits: (f32, f32),
    rotation_pid: Pid<f32>
}

impl<L, R, G> Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    f32: From<<L as GetPosition>::Position>,
    f32: From<<R as GetPosition>::Position>
{
    pub fn new(left: L, right: R, rotation: G, wheels_distance_cm: f32,
               rotation_limits: (f32, f32), rotation_pid: Pid<f32>) -> Self {
        Self {
            wheels_distance: wheels_distance_cm,

            speed: ChassisSpeed::default(),

            current_movement: None,
            position: ChassisPosition::default(),

            left,
            right,

            rotation,

            rotation_limits,
            rotation_pid
        }
    }

    fn get_wheel_positions(&self)
        -> (f32, f32) {
        (self.left.get_position().into(), self.right.get_position().into())
    }

    fn normalize_rotation(&self, val: f32) -> f32 {
        2.0 * ((val - self.rotation_limits.0) / (self.rotation_limits.1 - self.rotation_limits.0)) - 1.0
    }

    fn get_rotation(&mut self) -> f32 {
        let rotation =  self.rotation.get_rotation();
        self.normalize_rotation(rotation)
    }
}

impl<L, R, G> Update for Chassis<L, R, G> 
where
    L: ChassisMotor,
    R: ChassisMotor,
    G: GetRotation,
    f32: From<<L as GetPosition>::Position>,
    f32: From<<R as GetPosition>::Position>,
    <L as SetSpeed>::Speed: From<f32>, 
    <R as SetSpeed>::Speed: From<f32> 
{
    fn update(&mut self, time_delta_seconds: f32) {
        self.left.update(time_delta_seconds);
        self.right.update(time_delta_seconds);

        let wheel_positions = self.get_wheel_positions();

        let rotation =  self.get_rotation();

        if let Some(ref mut movement) = self.current_movement {
            let rotation_delta = movement.start_rotation - rotation;

            let wheels_distance = (
                (wheel_positions.0 - movement.prev_pos.0), (wheel_positions.1 - movement.prev_pos.1)
            );
            movement.prev_pos = wheel_positions;

            let control = self.rotation_pid.next_control_output(rotation_delta);
            rprintln!("{} {:?}", rotation_delta, control);
            let control = control.output;

            match movement.movement {
                AtomicMovement::Linear(_) => {
                    self.position.linear.0 += libm::cosf(self.position.angular) * wheels_distance.0;
                    self.position.linear.1 += libm::sinf(self.position.angular) * wheels_distance.1;

                    self.left.set_speed((self.speed.linear - 10.0 * control).into());
                    self.right.set_speed((self.speed.linear + 10.0 * control).into());
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
            prev_pos: wheel_positions.clone(),
            start_rotation: self.get_rotation()
        });

        match movement {
            AtomicMovement::Linear(distance) => {
                self.left.set_position((wheel_positions.0 + distance).into());
                self.right.set_position((wheel_positions.1 + distance).into());

                self.rotation_pid.setpoint = 0.0;
            },
            AtomicMovement::Angular(angle) => {
                let increment = angle.to_radians() * (self.wheels_distance / 2.0);
                self.left.set_position((wheel_positions.0 + increment).into());
                self.right.set_position((wheel_positions.1 - increment).into());
            }
        };
    }
}

