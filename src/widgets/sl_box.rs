//! A canvas widget that draws a saturation/lightness picker area.

use iced::widget::canvas;
use iced::{mouse, Rectangle, Renderer, Theme};

use crate::color::hsl_to_rgb;
use crate::message::Message;

/// A canvas program that draws a saturation/lightness picker area.
pub struct SaturationLightnessBox {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

impl canvas::Program<Message> for SaturationLightnessBox {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw a grid of saturation (x-axis) and lightness (y-axis)
        let step_x = bounds.width / 50.0;
        let step_y = bounds.height / 50.0;

        for ix in 0..50 {
            for iy in 0..50 {
                let s = (ix as f32 + 0.5) / 50.0;
                let l = 1.0 - (iy as f32 + 0.5) / 50.0; // Invert Y so light is at top
                let (r, g, b) = hsl_to_rgb(self.hue, s, l);
                let color = iced::Color::from_rgb8(r, g, b);
                frame.fill_rectangle(
                    iced::Point::new(ix as f32 * step_x, iy as f32 * step_y),
                    iced::Size::new(step_x.ceil(), step_y.ceil()),
                    color,
                );
            }
        }

        // Draw indicator for current position
        let indicator_x = self.saturation * bounds.width;
        let indicator_y = (1.0 - self.lightness) * bounds.height;

        // Draw crosshair
        let circle_radius = 8.0;
        frame.stroke(
            &canvas::Path::circle(iced::Point::new(indicator_x, indicator_y), circle_radius),
            canvas::Stroke::default()
                .with_color(iced::Color::WHITE)
                .with_width(2.0),
        );
        frame.stroke(
            &canvas::Path::circle(
                iced::Point::new(indicator_x, indicator_y),
                circle_radius + 1.0,
            ),
            canvas::Stroke::default()
                .with_color(iced::Color::BLACK)
                .with_width(1.0),
        );

        // Draw border
        frame.stroke(
            &canvas::Path::rectangle(iced::Point::ORIGIN, bounds.size()),
            canvas::Stroke::default()
                .with_color(iced::Color::from_rgb(0.3, 0.3, 0.3))
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}
