//! A canvas widget that draws a color swatch with checkerboard background for transparency.

use iced::widget::canvas;
use iced::{mouse, Rectangle, Renderer, Theme};

use super::draw_checkerboard;
use crate::message::Message;

/// A canvas program that draws a color swatch with a checkerboard background for transparency.
pub struct ColorSwatch {
    pub color: iced::Color,
}

impl canvas::Program<Message> for ColorSwatch {
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

        // Draw checkerboard pattern for transparency preview
        draw_checkerboard(&mut frame, bounds, 8.0);

        // Draw the actual color on top
        frame.fill_rectangle(iced::Point::ORIGIN, bounds.size(), self.color);

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
