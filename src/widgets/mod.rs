//! Reusable canvas widgets for color visualization.

mod alpha_bar;
mod color_swatch;
mod hue_bar;
mod sl_box;

pub use alpha_bar::AlphaBar;
pub use color_swatch::ColorSwatch;
pub use hue_bar::HueBar;
pub use sl_box::SaturationLightnessBox;

use iced::widget::canvas;

/// Draw a checkerboard pattern for transparency visualization.
pub fn draw_checkerboard(frame: &mut canvas::Frame, bounds: iced::Rectangle, check_size: f32) {
    let light = iced::Color::from_rgb(0.8, 0.8, 0.8);
    let dark = iced::Color::from_rgb(0.5, 0.5, 0.5);

    let cols = (bounds.width / check_size).ceil() as usize;
    let rows = (bounds.height / check_size).ceil() as usize;

    for row in 0..rows {
        for col in 0..cols {
            let is_light = (row + col) % 2 == 0;
            let color = if is_light { light } else { dark };
            frame.fill_rectangle(
                iced::Point::new(col as f32 * check_size, row as f32 * check_size),
                iced::Size::new(check_size, check_size),
                color,
            );
        }
    }
}
