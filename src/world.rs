#![allow(dead_code)]
#![allow(unused_variables)]

use crate::input::Input;
use crate::vec::Vec2;

pub struct World {
    pub camera: Camera,
    pub lift: Lift,
    pub player: Player,

    pub n_floors: u32,
    pub floor_size: Vec2<f32>,
}

impl World {
    pub fn create(n_floors: u32, floor_size: Vec2<f32>) -> Self {
        let floor_size = Vec2::new(500.0, 3.0);

        let lift_floor = 0;
        let lift_max_speed = 4.0;
        let lift =
            Lift::from_floor(lift_floor, floor_size.y, lift_max_speed);
        let camera = Camera::create(lift.get_primitive_position());

        let player = Player::from_lift(&lift);

        Self {
            camera,
            lift,
            player,
            n_floors,
            floor_size,
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        self.update_lift(dt, input);
        self.update_player(dt);
        self.update_free_camera(input);
        // self.camera.position = self.lift.get_position();
    }

    fn update_lift(&mut self, dt: f32, input: &Input) {
        let cursor_world_pos = screen_to_world(
            &self.camera,
            input.window_size,
            input.cursor_pos,
        );

        let mut lift_floor = self.get_lift_floor();
        if (lift_floor.floor() - lift_floor).abs() < 1.0e-5 {
            if let Some(_) = input.lmb_press_pos {
                lift_floor += 1.0;
            } else if let Some(_) = input.rmb_press_pos {
                lift_floor -= 1.0;
            }
            lift_floor = lift_floor
                .min(self.get_max_floor() as f32)
                .max(self.get_min_floor() as f32);
            self.lift.target_y = lift_floor.floor() * self.floor_size.y;
        }

        let diff = self.lift.target_y - self.lift.y;
        let step = dt * self.lift.max_speed;
        if step >= diff.abs() {
            self.lift.y = self.lift.target_y;
        } else {
            self.lift.y += step * diff.signum();
        }
    }

    fn update_player(&mut self, dt: f32) {
        self.player.position.y = self.lift.y;
    }

    fn update_free_camera(&mut self, input: &Input) {
        self.camera.aspect =
            input.window_size.x as f32 / input.window_size.y as f32;
        if input.wheel_d != 0 {
            let diff = self.camera.view_width * 0.1 * input.wheel_d as f32;
            self.camera.view_width -= diff;
        }

        if input.mmb_is_down {
            let cursor_world_pos = screen_to_world(
                &self.camera,
                input.window_size,
                input.cursor_pos,
            );
            let cursor_world_prev_pos = screen_to_world(
                &self.camera,
                input.window_size,
                input.cursor_prev_pos,
            );
            let mut cursor_world_diff =
                cursor_world_pos - cursor_world_prev_pos;
            cursor_world_diff.rotate_inplace(
                Vec2::new(0.0, 0.0),
                -self.camera.orientation,
            );
            self.camera.position -= cursor_world_diff;
        }
    }

    pub fn get_lift_floor(&self) -> f32 {
        self.lift.y / self.floor_size.y
    }

    pub fn get_max_floor(&self) -> i32 {
        (self.n_floors / 2) as i32
    }

    pub fn get_min_floor(&self) -> i32 {
        -(self.n_floors as i32 - self.get_max_floor() - 1)
    }

    pub fn get_shaft_primitive_xywh(&self) -> [f32; 4] {
        let min_y = self.get_min_floor() as f32 * self.floor_size.y;
        let max_y = (self.get_max_floor() + 1) as f32 * self.floor_size.y;
        let y = (max_y + min_y) / 2.0;
        let height = max_y - min_y;

        [0.0, y, self.lift.size.x * 1.2, height]
    }

    pub fn get_floor_primitive_xywh(&self, floor: i32) -> [f32; 4] {
        let y = self.floor_size.y * (floor as f32 + 0.5);

        [0.0, y, self.floor_size.x, self.floor_size.y]
    }
}

pub struct Camera {
    pub position: Vec2<f32>,
    pub orientation: f32,

    pub view_width: f32,
    pub aspect: f32,
}

impl Camera {
    fn create(position: Vec2<f32>) -> Self {
        Self {
            position: position,
            orientation: 0.0,
            view_width: 20.0,
            aspect: 1.77,
        }
    }

    pub fn get_view_size(&self) -> Vec2<f32> {
        let view_height = self.view_width / self.aspect;

        Vec2::new(self.view_width, view_height)
    }
}

pub struct Lift {
    pub size: Vec2<f32>,
    pub y: f32,
    pub target_y: f32,
    pub max_speed: f32,
    speed: f32,
}

impl Lift {
    pub fn create(size: Vec2<f32>, y: f32, max_speed: f32) -> Self {
        Self {
            size: size,
            y: y,
            target_y: y,
            max_speed: max_speed,
            speed: 0.0,
        }
    }

    pub fn from_floor(
        floor: i32,
        floor_height: f32,
        max_speed: f32,
    ) -> Self {
        let size = Vec2::new(floor_height * 0.6, floor_height);
        let y = floor as f32 * floor_height;

        Lift::create(size, y, max_speed)
    }

    pub fn get_primitive_xywh(&self) -> [f32; 4] {
        let y = self.y + 0.5 * self.size.y;

        [0.0, y, self.size.x, self.size.y]
    }

    pub fn get_primitive_position(&self) -> Vec2<f32> {
        let xywh = self.get_primitive_xywh();

        Vec2::new(xywh[0], xywh[1])
    }
}

pub struct Player {
    pub size: Vec2<f32>,
    pub position: Vec2<f32>,
}

impl Player {
    pub fn from_lift(lift: &Lift) -> Self {
        let size = lift.size * Vec2::new(0.25, 0.55);
        let position = Vec2::new(0.0, lift.y);

        Self { size, position }
    }

    pub fn get_primitive_xywh(&self) -> [f32; 4] {
        let y = self.position.y + 0.5 * self.size.y;

        [self.position.x, y, self.size.x, self.size.y]
    }
}

fn screen_to_world(
    camera: &Camera,
    window_size: Vec2<i32>,
    screen_pos: Vec2<i32>,
) -> Vec2<f32> {
    let width = window_size.x as f32;
    let height = window_size.y as f32;
    let window_size = Vec2::new(width, height);
    let aspect = width / height;

    let view_size = camera.get_view_size();

    let screen_pos = Vec2::<f32>::new(
        screen_pos.x as f32,
        height - screen_pos.y as f32,
    );
    let bot_left = camera.position - view_size.scale(0.5);
    let mut world_pos = bot_left + view_size * screen_pos / window_size;
    world_pos = world_pos.rotate(Vec2::zeros(), camera.orientation);
    return world_pos;
}
