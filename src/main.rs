use std::thread;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use crossbeam::channel::{unbounded, Sender};
use sfml::{
    graphics::{Color as SFMLColor, Font, RenderTarget, RenderWindow, Sprite, Text, Texture, Transformable},
    window::{Event, Style},
};
use tracy::{
    camera::Camera,
    hittable::{sphere::Sphere, HittableList, moving_sphere::MovingSphere},
    material::{dielectric::Dielectric, lambertian::Lambertian, metal::Metal},
    random_float, random_float_between, Color, Point3, Vec3,
};

use rayon::prelude::*;

// Image
const ASPECT_RATIO: f64 = 3.0 / 2.0;
const IMAGE_WIDTH: u32 = 240;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO as f64) as u32;
const SAMPLES_PER_PIXEL: u32 = 100;
const MAX_DEPTH: i32 = 50;

fn main() {
    // UI

    let mut window = RenderWindow::new(
        (IMAGE_WIDTH, IMAGE_HEIGHT),
        "Tracy",
        Style::CLOSE,
        &Default::default(),
    );

    let mut texture = Texture::new().unwrap();
    
    if !texture.create(IMAGE_WIDTH, IMAGE_HEIGHT) {
        panic!("Unable to create texture");
    }

    texture.set_smooth(false);

    // Load a font for rendering text
    let font = unsafe {
        Font::from_memory(include_bytes!("/System/Library/Fonts/Helvetica.ttc"))
            .expect("Failed to load font")
    };

    let (s, r) = unbounded();

    thread::spawn(|| {
        render(s);
    });

    let total_pixels = IMAGE_WIDTH * IMAGE_HEIGHT;
    let mut pixels_rendered = 0u32;
    let mut rendering_complete = false;

    while window.is_open() {
        // Event processing
        while let Some(event) = window.poll_event() {
            // Request closing for the window
            if event == Event::Closed {
                window.close();
            }
        }

        // Process all pending messages without blocking
        while let Ok(msg) = r.try_recv() {
            match msg {
                RenderMessage::Pixel(pixel) => {
                    unsafe {
                        texture.update_from_pixels(&pixel.color, 1, 1, pixel.x, pixel.y);
                    }
                    pixels_rendered += 1;
                }
                RenderMessage::Progress(count) => {
                    pixels_rendered = count;
                }
                RenderMessage::Done => {
                    rendering_complete = true;
                    eprintln!("Rendering complete!");
                }
            }
        }

        window.clear(SFMLColor::rgb(30, 30, 30));

        if rendering_complete {
            // Show the final rendered image
            let mut sprite = Sprite::new();
            sprite.set_texture(&texture, true);
            window.draw(&sprite);
        } else {
            // Show progress text in the center of the window
            let progress_text = format!(
                "Beep Boop..Tracing..\n\n{}/{} pixels",
                pixels_rendered, total_pixels
            );
            
            let mut text = Text::new(&progress_text, &font, 24);
            text.set_fill_color(SFMLColor::WHITE);
            
            // Center the text
            let text_bounds = text.local_bounds();
            text.set_position((
                (IMAGE_WIDTH as f32 - text_bounds.width) / 2.0 - text_bounds.left,
                (IMAGE_HEIGHT as f32 - text_bounds.height) / 2.0 - text_bounds.top,
            ));
            
            window.draw(&text);
        }

        window.display();
    }
}

enum RenderMessage {
    Pixel(Pixel),
    Progress(u32), // Number of pixels rendered so far
    Done,
}

struct Pixel {
    pub x: u32,
    pub y: u32,
    pub color: [u8; 4],
}

fn render(s: Sender<RenderMessage>) {
    // World
    let world = sebi_scene();

    let lookfrom = Point3::new(4.5, 2.5, 18.0);
    let lookat = Point3::new(4.5, 1.8, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 18.0;
    let aperture = 0.05;

    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        50.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
        Some((0.0, 1.0)),
    );

    eprintln!("Start Render!");

    let pixel_count = Arc::new(AtomicU32::new(0));

    (0..IMAGE_HEIGHT).into_par_iter().rev().for_each(|j| {
        (0..IMAGE_WIDTH).for_each(|i| {
            let color: Color = (0..SAMPLES_PER_PIXEL)
                .map(|_| {
                    let u = (i as f64 + random_float()) / (IMAGE_WIDTH - 1) as f64;
                    let v = (j as f64 + random_float()) / (IMAGE_HEIGHT - 1) as f64;
                    let ray = camera.get_ray(u, v);
                    ray.color(&world, MAX_DEPTH)
                })
                .sum();

            let result = s.send(RenderMessage::Pixel(Pixel {
                x: i,
                y: IMAGE_HEIGHT - j,
                color: [
                    (255.99
                        * (color.x() / SAMPLES_PER_PIXEL as f64)
                            .sqrt()
                            .max(0.0)
                            .min(1.0)) as u8,
                    (255.99
                        * (color.y() / SAMPLES_PER_PIXEL as f64)
                            .sqrt()
                            .max(0.0)
                            .min(1.0)) as u8,
                    (255.99
                        * (color.z() / SAMPLES_PER_PIXEL as f64)
                            .sqrt()
                            .max(0.0)
                            .min(1.0)) as u8,
                    255,
                ],
            }));
            
            // Update counter and send progress every 100 pixels
            let count = pixel_count.fetch_add(1, Ordering::Relaxed) + 1;
            if count % 100 == 0 {
                let _ = s.send(RenderMessage::Progress(count));
            }
            
            // If send fails, window was closed, so we can stop rendering
            if result.is_err() {
                return;
            }
        });
    });

    // Send completion message
    let _ = s.send(RenderMessage::Done);
}

fn sebi_scene() -> HittableList {
    let mut world = HittableList::default();
    let m1 = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        m1,
    ));

    let sphere_radius = 0.3;
    let spacing = 0.7;
    let primary_mat = Metal::new(Color::new(0.8, 0.2, 0.2), 0.1);
    let secondary_mat = Dielectric::new(1.5);
    
    world.add(Sphere::new(Point3::new(-2.0 + spacing * 2.0, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0, sphere_radius + spacing * 1.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-2.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-2.0, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0 + spacing * 1.3, sphere_radius + spacing * 2.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0, sphere_radius + spacing * 0.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 0.5, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 0.7, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-2.0 + spacing * 2.0, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(1.5, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2 + spacing, sphere_radius + spacing * 1.5, 0.0), sphere_radius, secondary_mat));
    world.add(Sphere::new(Point3::new(7.8, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2 + spacing * 1.5, sphere_radius + spacing * 2.5, 0.0), sphere_radius, secondary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2 + spacing * 0.5, sphere_radius + spacing * 3.0, 0.0), sphere_radius, secondary_mat));
    world.add(Sphere::new(Point3::new(10.6 + spacing * 2.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2, sphere_radius + spacing * 2.5, 0.0), sphere_radius, secondary_mat));
    world.add(Sphere::new(Point3::new(6.5, 0.7, -4.5), 0.7, Metal::new(Color::new(0.2, 0.8, 0.3), 0.1)));
    world.add(Sphere::new(Point3::new(10.6 + spacing, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(1.5 + spacing * 0.5, sphere_radius + spacing * 2.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0, sphere_radius + spacing * 2.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(10.6 + spacing, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(0.5, 1.0, -4.0), 1.0, Dielectric::new(1.5)));
    world.add(Sphere::new(Point3::new(10.6, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0 + spacing * 0.7, sphere_radius + spacing * 1.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(2.0, 0.55, -8.0), 0.55, Metal::new(Color::new(0.95, 0.6, 0.2), 0.15)));
    world.add(Sphere::new(Point3::new(5.0 + spacing * 2.0, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0 + spacing * 2.0, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-2.0 + spacing * 2.0, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(8.5, 0.8, -3.5), 0.8, Metal::new(Color::new(0.95, 0.85, 0.3), 0.0)));
    world.add(Sphere::new(Point3::new(1.5 + spacing, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 2.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 2.1, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0, 0.6, -7.0), 0.6, Lambertian::new(Color::new(0.8, 0.3, 0.7))));
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 1.4, sphere_radius + spacing * 0.5, 0.0), sphere_radius, primary_mat));
    
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 2.8, sphere_radius + spacing * 1.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0 + spacing * 2.0, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 2.8, sphere_radius + spacing * 0.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 2.0, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 1.5, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0, 0.4, -6.5), 0.4, Lambertian::new(Color::new(0.9, 0.4, 0.8))));
    world.add(Sphere::new(Point3::new(10.6 + spacing, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 2.8, sphere_radius + spacing * 2.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(1.5 + spacing * 2.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 0.5, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2 + spacing * 1.5, sphere_radius + spacing * 2.0, 0.0), sphere_radius, secondary_mat));
    world.add(Sphere::new(Point3::new(-7.0, 0.6, -4.0), 0.6, Lambertian::new(Color::new(0.2, 0.4, 0.8))));
    world.add(Sphere::new(Point3::new(-2.0, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(3.0, 0.5, -6.0), 0.5, Metal::new(Color::new(0.4, 0.9, 0.4), 0.0)));
    world.add(Sphere::new(Point3::new(-6.0 + spacing * 1.4, sphere_radius + spacing * 1.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 2.0, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-2.0 + spacing * 2.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2 + spacing, sphere_radius + spacing * 3.0, 0.0), sphere_radius, secondary_mat));
    world.add(Sphere::new(Point3::new(-2.0 + spacing, sphere_radius + spacing * 1.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(13.2 + spacing * 0.75, sphere_radius + spacing * 0.3, 0.0), sphere_radius * 0.8, secondary_mat));
    world.add(Sphere::new(Point3::new(-3.5, 0.7, -7.5), 0.7, Metal::new(Color::new(0.9, 0.5, 0.1), 0.2)));
    world.add(Sphere::new(Point3::new(10.6 + spacing, sphere_radius, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-9.0, 0.5, -5.0), 0.5, Dielectric::new(1.5)));
    world.add(Sphere::new(Point3::new(5.0 + spacing * 2.0, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-4.0, 0.8, -5.0), 0.8, Lambertian::new(Color::new(0.3, 0.6, 0.9))));
    world.add(Sphere::new(Point3::new(1.5 + spacing * 1.5, sphere_radius + spacing * 2.5, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-1.0, 0.65, -5.5), 0.65, Metal::new(Color::new(0.9, 0.8, 0.2), 0.05)));
    world.add(Sphere::new(Point3::new(1.5 + spacing, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 2.0, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(-2.0, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(1.5 + spacing, sphere_radius + spacing * 2.0, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(5.0, sphere_radius + spacing, 0.0), sphere_radius, primary_mat));
    world.add(Sphere::new(Point3::new(7.8 + spacing * 1.5, sphere_radius + spacing * 3.0, 0.0), sphere_radius, primary_mat));
    world
}

#[allow(dead_code)]
fn test_scene() -> HittableList {
    let mut world = HittableList::default();

    let material_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Lambertian::new(Color::new(0.1, 0.2, 0.5));
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2), 1.0);

    world.add(Sphere::new(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        material_ground,
    ));
    world.add(Sphere::new(
        Point3::new(0.0, 0.0, -1.0),
        0.5,
        material_center,
    ));
    world.add(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        material_left,
    ));
    world.add(Sphere::new(
        Point3::new(-1.0, 0.0, -1.0),
        -0.45,
        material_left,
    ));
    world.add(Sphere::new(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        material_right,
    ));

    world
}

#[allow(dead_code)]
fn random_scene() -> HittableList {
    let mut world = HittableList::default();

    let ground_material = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    ));

    for a in -11..11 {
        let a = f64::from(a);

        for b in -11..11 {
            let b = f64::from(b);

            let choose_mat = random_float();
            let center = Point3::new(a + 0.9 * random_float(), 0.2, b + 0.9 * random_float());

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    let material = Lambertian::new(albedo);
                    let center2 = center + Point3::new(0.0, random_float_between(0.0, 0.5), 0.0);
                    world.add(MovingSphere::new(center, center2, 0.0, 1.0, 0.2, material));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_between(0.5, 1.0);
                    let fuzz = random_float_between(0.0, 0.5);
                    let material = Metal::new(albedo, fuzz);
                    world.add(Sphere::new(center, 0.2, material));
                } else {
                    // glass
                    let material = Dielectric::new(1.5);
                    world.add(Sphere::new(center, 0.2, material));
                }
            }
        }
    }

    let material1 = Dielectric::new(1.5);
    world.add(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, material1));

    let material2 = Lambertian::new(Color::new(0.4, 0.2, 0.1));
    world.add(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, material2));

    let material3 = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);
    world.add(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, material3));

    world
}
