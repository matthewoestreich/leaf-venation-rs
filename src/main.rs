use raylib::{RaylibHandle, color::Color, ffi::KeyboardKey, math::Vector2, prelude::RaylibDraw};

const VEIN_RADIUS: f32 = 5.0;
const VEIN_COLOR: Color = Color::WHITE;
const VEIN_CORE_COLOR: Color = Color::BLACK;
const AUXINS_RATE: usize = 20;
const AUXINS_RADIUS: f32 = 5.0; // Should be same as VEIN_RADIUS
const AUXINS_COLOR: Color = Color::RED;
const AUXIMITY: f32 = 30.0;

fn main() {
    assert_eq!(
        VEIN_RADIUS, AUXINS_RADIUS,
        "VEIN_RADIUS ({VEIN_RADIUS}) and AUXINS_RADIUS ({AUXINS_RADIUS}) should be equal!",
    );

    let width = 800;
    let height = 600;

    let (mut rl, rl_thread) = raylib::init()
        .size(width, height)
        .title("Leaf Venation")
        .build();

    let mut veins = vec![];
    let mut auxins = vec![];

    init(&rl, &mut auxins, &mut veins);

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            init(&rl, &mut auxins, &mut veins);
        }

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            venate(&rl, &mut auxins, &mut veins);
        }

        let mut drawing = rl.begin_drawing(&rl_thread);

        drawing.set_target_fps(60);
        drawing.clear_background(Color::get_color(0x181818FF));

        for vein in veins.iter_mut() {
            drawing.draw_circle(
                vein.position.x as i32,
                vein.position.y as i32,
                VEIN_RADIUS,
                VEIN_COLOR,
            );
            drawing.draw_circle(
                vein.position.x as i32,
                vein.position.y as i32,
                VEIN_RADIUS / 2f32,
                VEIN_CORE_COLOR,
            );
            vein.direction.scale(20f32);
            drawing.draw_line_v(vein.position, vein.position + vein.direction, Color::PURPLE);
        }

        for auxin in auxins.iter() {
            drawing.draw_circle(auxin.x as i32, auxin.y as i32, AUXINS_RADIUS, AUXINS_COLOR);
        }
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

fn init(rl: &RaylibHandle, auxins: &mut Vec<Vector2>, veins: &mut Vec<Vein>) {
    veins.clear();
    auxins.clear();
    let width = (rl.get_screen_width() / 2) as f32;
    let height = (rl.get_screen_height() * 2 / 3) as f32;
    veins.push(Vein::new(Vector2::new(width, height)));
    spray_auxins(rl, auxins);
    kill_auxins_by_auximity(auxins, veins);
}

fn venate(rl: &RaylibHandle, auxins: &mut Vec<Vector2>, veins: &mut Vec<Vein>) {
    calc_growth_dir(auxins, veins);
    grow_new_veins(veins);
    kill_auxins_by_auximity(auxins, veins);
    spray_auxins(rl, auxins);
    kill_auxins_by_auximity(auxins, veins);
}

fn spray_auxins(rl: &RaylibHandle, auxins: &mut Vec<Vector2>) {
    let height = rl.get_screen_height();
    let width = rl.get_screen_width();

    for _ in 0..AUXINS_RATE {
        let x = rl.get_random_value::<i32>(0..width.saturating_sub(1)) as f32;
        let y = rl.get_random_value::<i32>(0..height.saturating_sub(1)) as f32;
        auxins.push(Vector2::new(x, y));
    }
}

fn kill_auxins_by_auximity(auxins: &mut Vec<Vector2>, veins: &mut [Vein]) {
    let mut to_remove = vec![];

    for (index, auxin) in auxins.iter().enumerate() {
        for vein in veins.iter() {
            if auxin.distance_to(vein.position) <= AUXIMITY {
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

fn grow_new_veins(veins: &mut Vec<Vein>) {
    let mut new_vein_positions = Vec::<Vector2>::new();

    for vein in veins.iter_mut() {
        if vein.direction.x == 0f32 && vein.direction.y == 0f32 {
            continue;
        }
        let x = vein.position.x + vein.direction.x * VEIN_RADIUS * 2f32;
        let y = vein.position.y + vein.direction.y * VEIN_RADIUS * 2f32;
        let vein_position = Vector2::new(x, y);
        new_vein_positions.push(vein_position);
    }

    for &position in new_vein_positions.iter() {
        veins.push(Vein {
            position,
            ..Default::default()
        });
    }
}
