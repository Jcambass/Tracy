use tracy::Color;

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

fn main() {
    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("\rScanlines remaining: {} ", j);

        for i in 0..IMAGE_WIDTH {
            let pixel_color = Color::new(i as f64 / (IMAGE_WIDTH - 1) as f64, j as f64 / (IMAGE_HEIGHT - 1) as f64, 0.25);
            pixel_color.print_color();
        }
    }

    eprint!("\nDone.\n");
}
