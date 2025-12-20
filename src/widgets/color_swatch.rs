//! A canvas widget that draws a color swatch with checkerboard background for transparency.

use iced::widget::canvas;
use iced::{mouse, Rectangle, Renderer, Theme};

use super::draw_checkerboard;
use crate::message::Message;
use crate::theme::RADIUS_MD;

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

        let radius = RADIUS_MD;

        // Draw checkerboard pattern for transparency preview
        draw_checkerboard(&mut frame, bounds, 8.0);

        // Create a rounded rectangle path
        let rounded_rect = rounded_rectangle_path(bounds.size(), radius);

        // Draw the actual color on top with rounded corners
        frame.fill(&rounded_rect, self.color);

        // Draw border with rounded corners
        frame.stroke(
            &rounded_rect,
            canvas::Stroke::default()
                .with_color(iced::Color::from_rgb(0.3, 0.3, 0.3))
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

/// Create a rounded rectangle path.
fn rounded_rectangle_path(size: iced::Size, radius: f32) -> canvas::Path {
    use iced::Point;

    let width = size.width;
    let height = size.height;
    let r = radius.min(width / 2.0).min(height / 2.0);

    canvas::Path::new(|builder| {
        // Start at top-left, after the corner radius
        builder.move_to(Point::new(r, 0.0));

        // Top edge
        builder.line_to(Point::new(width - r, 0.0));

        // Top-right corner
        builder.arc_to(Point::new(width, 0.0), Point::new(width, r), r);

        // Right edge
        builder.line_to(Point::new(width, height - r));

        // Bottom-right corner
        builder.arc_to(Point::new(width, height), Point::new(width - r, height), r);

        // Bottom edge
        builder.line_to(Point::new(r, height));

        // Bottom-left corner
        builder.arc_to(Point::new(0.0, height), Point::new(0.0, height - r), r);

        // Left edge
        builder.line_to(Point::new(0.0, r));

        // Top-left corner
        builder.arc_to(Point::new(0.0, 0.0), Point::new(r, 0.0), r);

        builder.close();
    })
}
