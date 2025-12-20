//! A canvas widget that draws a hue spectrum bar.

use iced::widget::canvas;
use iced::{mouse, Rectangle, Renderer, Theme};

use crate::color::hsl_to_rgb;
use crate::message::Message;

/// A canvas program that draws a hue spectrum bar.
pub struct HueBar {
    pub current_hue: f32,
}

impl canvas::Program<Message> for HueBar {
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

        // Draw hue gradient (360 degrees of hue)
        let step = bounds.width / 360.0;
        for i in 0..360 {
            let hue = i as f32;
            let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.5);
            let color = iced::Color::from_rgb8(r, g, b);
            frame.fill_rectangle(
                iced::Point::new(i as f32 * step, 0.0),
                iced::Size::new(step.ceil(), bounds.height),
                color,
            );
        }

        // Draw indicator for current hue
        let indicator_x = (self.current_hue / 360.0) * bounds.width;
        frame.fill_rectangle(
            iced::Point::new(indicator_x - 2.0, 0.0),
            iced::Size::new(4.0, bounds.height),
            iced::Color::WHITE,
        );
        frame.stroke(
            &canvas::Path::rectangle(
                iced::Point::new(indicator_x - 2.0, 0.0),
                iced::Size::new(4.0, bounds.height),
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
