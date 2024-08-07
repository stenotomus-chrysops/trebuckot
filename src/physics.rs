use crate::{utils::*, Game, GameState};
use macroquad::prelude::*;
// use std::f32::consts;

const PHYSICS_TICK: f32 = 0.001;

pub struct Physics {
    time_acc:           f32,
    float_displacement: Vec2,
}

impl Physics {
    pub async fn init() -> Physics {
        Physics {
            time_acc:           0.0,
            float_displacement: Vec2::ZERO,
        }
    }

    pub fn update(&mut self, game: &mut Game) {
        if game.state != GameState::Launched {
            return;
        }

        if is_key_down(KeyCode::Escape) {
            game.state = GameState::Paused;
        }

        self.time_acc += get_frame_time();
        while self.time_acc > PHYSICS_TICK {
            self.time_acc -= PHYSICS_TICK;

            // Basic movement
            if is_key_down(KeyCode::W) {
                game.player.acceleration +=
                    Vec2::from_angle(game.player.rotation) * game.player.move_speed;
            }
            if is_key_down(KeyCode::S) {
                game.player.acceleration -=
                    Vec2::from_angle(game.player.rotation) * game.player.move_speed;
            }
            if is_key_down(KeyCode::A) {
                game.player.rotation += 0.001;
            }
            if is_key_down(KeyCode::D) {
                game.player.rotation -= 0.001;
            }

            if !game.trebuchet.run(PHYSICS_TICK) {
                game.player.position = game.trebuchet.projectile_position();
                game.player.velocity = game.trebuchet.v_projectile();
                game.player.rotation =
                    (game.trebuchet.sling_point() - game.trebuchet.armsling_point()).to_angle();
                continue;
            }

            game.player.acceleration += game.world.grativy_at(game.player.position);
            let displacement = (game.player.velocity * PHYSICS_TICK)
                + 0.5 * game.player.acceleration * PHYSICS_TICK.powi(2);
            self.float_displacement += displacement;

            let next_position = game.player.position + to_i64coords(self.float_displacement);
            let next_gravity = game.world.grativy_at(next_position);
            game.player.velocity += 0.5 * (game.player.acceleration + next_gravity) * PHYSICS_TICK;
            game.player.acceleration = Vec2::ZERO;

            game.stats.time += PHYSICS_TICK;
            game.stats.distance += displacement.length();
            game.stats.max_altitude = game
                .stats
                .max_altitude
                .max(game.world.altitude_at(game.player.position));
            game.stats.max_speed = game.stats.max_speed.max(game.player.velocity.length());

            if let Some(point) = ground_collision(game, displacement) {
                game.player.position = point;
                game.player.velocity = Vec2::ZERO;
                game.state = GameState::Landed;
                self.float_displacement = Vec2::ZERO;
                break;
            }
        }

        let i64_displacement: I64Vec2;
        (i64_displacement, self.float_displacement) =
            to_i64coords_with_rem(self.float_displacement);
        game.player.position += i64_displacement;
    }
}

fn ground_collision(game: &Game, displacement: Vec2) -> Option<I64Vec2> {
    let circ = game.world.height_map.len();
    let terrain_index = game.world.terrain_index_beneath(game.player.position);
    let terrain_a = game.world.surface(terrain_index);
    let terrain_b = game.world.surface((terrain_index + 1) % circ);

    let next_position = game.player.position + to_i64coords(displacement);

    // If player over terrain at next position
    if orientation(terrain_a, terrain_b, next_position) == Orientation::Clockwise {
        return None;
    }
    get_intersection(terrain_a, terrain_b, game.player.position, next_position)
}

#[derive(Debug, PartialEq)]
enum Orientation {
    Clockwise,
    AntiClockwise,
    Colinear,
}

/// Orientation of ordered points
fn orientation(p: I64Vec2, q: I64Vec2, r: I64Vec2) -> Orientation {
    match (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y) {
        o if o > 0 => Orientation::Clockwise,
        o if o < 0 => Orientation::AntiClockwise,
        0 => Orientation::Colinear,
        _ => unreachable!(),
    }
}

fn to_i64coords_with_rem(f32coords: Vec2) -> (I64Vec2, Vec2) {
    let i64coords = (f32coords * 256.0).floor().as_i64vec2();
    let remainder = f32coords.rem_euclid(Vec2::splat(1.0 / 256.0));
    (i64coords, remainder)
}

#[cfg(test)]
mod physics_test {
    use super::to_i64coords_with_rem;
    use macroquad::math::{I64Vec2, Vec2};

    #[macroquad::test("Test")]
    async fn i64remainder() {
        let mut ami = Vec2::splat(0.5);
        let mut cute = (I64Vec2::splat(128), Vec2::ZERO);
        assert_eq!(to_i64coords_with_rem(ami), cute);

        let love = Vec2::splat(1.0 / 512.0);
        ami += love;
        cute.1 += love;
        assert_eq!(to_i64coords_with_rem(ami), cute);
    }
}
