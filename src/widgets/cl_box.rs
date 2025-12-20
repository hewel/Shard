//! A canvas widget that draws a chroma/lightness picker area for OKLCH with drag support.

use iced::widget::canvas::{self, Event};
use iced::{mouse, Rectangle, Renderer, Theme};

use crate::color::oklch_to_rgb;
use crate::message::Message;

/// A canvas program that draws a chroma/lightness picker area for OKLCH.
/// X-axis: Chroma (0 to max_chroma)
/// Y-axis: Lightness (0 at bottom, 1 at top)
pub struct ChromaLightnessBox {
    pub hue: f32,
    pub chroma: f32,
    pub lightness: f32,
    /// Maximum chroma to display (typically 0.4 for most hues)
    pub max_chroma: f32,
}

impl Default for ChromaLightnessBox {
    fn default() -> Self {
        Self {
            hue: 0.0,
            chroma: 0.15,
            lightness: 0.5,
            max_chroma: 0.4,
        }
    }
}

/// State to track if the user is currently dragging.
#[derive(Default)]
pub struct CLBoxState {
    is_dragging: bool,
}

impl ChromaLightnessBox {
    /// Convert cursor position (relative to bounds) to chroma/lightness values.
    fn position_to_cl(&self, bounds: Rectangle, position: iced::Point) -> (f32, f32) {
        let chroma = (position.x / bounds.width * self.max_chroma).clamp(0.0, self.max_chroma);
        // Invert Y: top = 1.0 (light), bottom = 0.0 (dark)
        let lightness = (1.0 - position.y / bounds.height).clamp(0.0, 1.0);

        (chroma, lightness)
    }
}

impl canvas::Program<Message> for ChromaLightnessBox {
    type State = CLBoxState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let cursor_position = cursor.position_in(bounds)?;

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.is_dragging = true;
                let (c, l) = self.position_to_cl(bounds, cursor_position);
                Some(canvas::Action::publish(Message::PickerCLChanged(c, l)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                let (c, l) = self.position_to_cl(bounds, cursor_position);
                Some(canvas::Action::publish(Message::PickerCLChanged(c, l)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.is_dragging =>
            {
                state.is_dragging = false;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.is_dragging {
            return mouse::Interaction::Crosshair;
        }

        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // Draw a grid of chroma (x-axis) and lightness (y-axis)
        let steps = 50;
        let step_x = bounds.width / steps as f32;
        let step_y = bounds.height / steps as f32;

        for ix in 0..steps {
            for iy in 0..steps {
                let c = (ix as f32 + 0.5) / steps as f32 * self.max_chroma;
                let l = 1.0 - (iy as f32 + 0.5) / steps as f32; // Invert Y so light is at top
                let (r, g, b) = oklch_to_rgb(l, c, self.hue);
                let color = iced::Color::from_rgb8(r, g, b);
                frame.fill_rectangle(
                    iced::Point::new(ix as f32 * step_x, iy as f32 * step_y),
                    iced::Size::new(step_x.ceil(), step_y.ceil()),
                    color,
                );
            }
        }

        // Draw indicator for current position
        let indicator_x = (self.chroma / self.max_chroma).clamp(0.0, 1.0) * bounds.width;
        let indicator_y = (1.0 - self.lightness) * bounds.height;

        // Draw crosshair circle
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
