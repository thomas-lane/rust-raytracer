use crate::{
    vec3::Vec3,
    interval::Interval,
};
use std::io::prelude::Write;

pub type Color = Vec3;

fn linear_to_gamma(linear_component: f64) -> f64 {
    linear_component.sqrt()
}

pub fn write_color(file: &mut std::fs::File, pixel_color: &Color, samples_per_pixel: i32) {
    let mut r = pixel_color.x();
    let mut g = pixel_color.y();
    let mut b = pixel_color.z();

    // Divide the color by the number of samples.
    let scale = 1.0 / samples_per_pixel as f64;
    r *= scale;
    g *= scale;
    b *= scale;

    // Apply the linear to gamma transform.
    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    // Write the translated [0,255] value of each color component.
    static INTENSITY: Interval = Interval::new(0.000, 0.999);
    write!(file, "{} {} {}\n",
        256.0 * INTENSITY.clamp(r),
        256.0 * INTENSITY.clamp(g),
        256.0 * INTENSITY.clamp(b),
        ).expect("Error writing color values to file");
}