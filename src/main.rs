#![allow(clippy::assertions_on_constants)]
use raylib::{
    RaylibHandle,
    color::Color,
    ffi::KeyboardKey,
    math::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle},
};

const FPS: u32 = 60;
const BACKGROUND_COLOR: u32 = 0x181818FF;
const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;
const TITLE: &str = "Leaf Venation";

const VEIN_RADIUS: f32 = 5.0;
const VEIN_DIRECTION_SCALE: f32 = 20.0;
const VEIN_DIRECTION_LINE_COLOR: Color = Color::PURPLE;
const VEIN_COLOR: Color = Color::WHITE;
const VEIN_CORE_COLOR: Color = Color::BLACK;
const AUXINS_SPRAY_RATE: usize = 20;
const AUXIN_RADIUS: f32 = 5.0; // Should be same as VEIN_RADIUS
const AUXIN_COLOR: Color = Color::RED;
const AUXIN_TO_VEIN_PROXIMITY: f32 = 30.0; // Auxins within this proximity to a vein will be killed.

fn main() {
    assert_eq!(
        VEIN_RADIUS, AUXIN_RADIUS,
        "VEIN_RADIUS ({VEIN_RADIUS}) and AUXINS_RADIUS ({AUXIN_RADIUS}) should be equal!",
    );
    assert!(
        AUXIN_TO_VEIN_PROXIMITY >= 20.0,
        "AUXIN_TO_VEIN_PROXIMITY ({AUXIN_TO_VEIN_PROXIMITY}) must be greater than or equal to 20.0!"
    );

    let (mut rl, rl_thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title(TITLE)
        .build();

    let mut veins = vec![];
    let mut auxins = vec![];

    init(
        &rl,
        &mut auxins,
        &mut veins,
        AUXIN_TO_VEIN_PROXIMITY,
        AUXINS_SPRAY_RATE,
    );

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            init(
                &rl,
                &mut auxins,
                &mut veins,
                AUXIN_TO_VEIN_PROXIMITY,
                AUXINS_SPRAY_RATE,
            );
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            calc_venation_step(
                &rl,
                &mut auxins,
                &mut veins,
                VEIN_RADIUS,
                AUXINS_SPRAY_RATE,
                AUXIN_TO_VEIN_PROXIMITY,
            );
        }

        let mut drawing = rl.begin_drawing(&rl_thread);
        drawing.set_target_fps(FPS);
        drawing.clear_background(Color::get_color(BACKGROUND_COLOR));
        draw_veins(
            &mut drawing,
            &mut veins,
            VEIN_RADIUS,
            VEIN_COLOR,
            VEIN_CORE_COLOR,
            VEIN_DIRECTION_SCALE,
            VEIN_DIRECTION_LINE_COLOR,
        );
        draw_auxins(&mut drawing, &auxins, AUXIN_RADIUS, AUXIN_COLOR);
    }
}

#[derive(Default)]
struct Vein {
    position: Vector2,
    direction: Vector2,
}

impl Vein {
    fn new(position: Vector2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }
}

fn init(
    rl: &RaylibHandle,
    auxins: &mut Vec<Vector2>,
    veins: &mut Vec<Vein>,
    proximity: f32,
    auxin_spray_rate: usize,
) {
    veins.clear();
    auxins.clear();

    let x = (rl.get_screen_width() / 2) as f32;
    let y = (rl.get_screen_height() * 2 / 3) as f32;
    veins.push(Vein::new(Vector2::new(x, y)));

    spray_auxins(rl, auxins, auxin_spray_rate);
    kill_auxins_by_vein_proximity(auxins, veins, proximity);
}

fn calc_venation_step(
    rl: &RaylibHandle,
    auxins: &mut Vec<Vector2>,
    veins: &mut Vec<Vein>,
    vein_radius: f32,
    auxin_spray_rate: usize,
    proximity: f32,
) {
    calc_growth_dir(auxins, veins);
    grow_new_veins(veins, vein_radius);
    spray_auxins(rl, auxins, auxin_spray_rate);
    kill_auxins_by_vein_proximity(auxins, veins, proximity);
}

fn draw_veins(
    draw_handle: &mut RaylibDrawHandle,
    veins: &mut [Vein],
    radius: f32,
    color: Color,
    core_color: Color,
    dir_line_scale: f32,
    dir_line_color: Color,
) {
    for vein in veins.iter_mut() {
        draw_handle.draw_circle(
            vein.position.x as i32,
            vein.position.y as i32,
            radius,
            color,
        );
        draw_handle.draw_circle(
            vein.position.x as i32,
            vein.position.y as i32,
            radius / 2f32,
            core_color,
        );

        vein.direction.scale(dir_line_scale);
        draw_handle.draw_line_v(
            vein.position,
            vein.position + vein.direction,
            dir_line_color,
        );
    }
}

fn draw_auxins(draw_handle: &mut RaylibDrawHandle, auxins: &[Vector2], radius: f32, color: Color) {
    for auxin in auxins.iter() {
        draw_handle.draw_circle(auxin.x as i32, auxin.y as i32, radius, color);
    }
}

fn spray_auxins(rl: &RaylibHandle, auxins: &mut Vec<Vector2>, spray_rate: usize) {
    let height = rl.get_screen_height();
    let width = rl.get_screen_width();

    for _ in 0..spray_rate {
        let x = rl.get_random_value::<i32>(0..width.saturating_sub(1)) as f32;
        let y = rl.get_random_value::<i32>(0..height.saturating_sub(1)) as f32;
        auxins.push(Vector2::new(x, y));
    }
}

fn kill_auxins_by_vein_proximity(auxins: &mut Vec<Vector2>, veins: &mut [Vein], proximity: f32) {
    let mut to_remove = vec![];

    for (index, auxin) in auxins.iter().enumerate() {
        for vein in veins.iter() {
            if auxin.distance_to(vein.position) <= proximity {
                to_remove.push(index);
                break;
            }
        }
    }

    for &i in to_remove.iter().rev() {
        auxins.remove(i);
    }
}

fn calc_growth_dir(auxins: &mut [Vector2], veins: &mut [Vein]) {
    if veins.is_empty() {
        return;
    }

    veins
        .iter_mut()
        .for_each(|vein| vein.direction = Vector2::zero());

    for &auxin in auxins.iter() {
        let mut closest = 0usize;
        for (i, vein) in veins.iter().enumerate() {
            if vein.position.distance_to(auxin) < veins[closest].position.distance_to(auxin) {
                closest = i;
            }
        }
        veins[closest].direction += auxin - veins[closest].position;
    }

    veins.iter_mut().for_each(|vein| vein.direction.normalize());
}

fn grow_new_veins(veins: &mut Vec<Vein>, radius: f32) {
    let mut new_veins = vec![];

    for vein in veins.iter() {
        if vein.direction.x == 0f32 && vein.direction.y == 0f32 {
            continue;
        }
        let x = vein.position.x + vein.direction.x * radius * 2f32;
        let y = vein.position.y + vein.direction.y * radius * 2f32;
        new_veins.push(Vein {
            position: Vector2::new(x, y),
            ..Default::default()
        });
    }

    veins.append(&mut new_veins);
}
