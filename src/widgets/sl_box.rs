//! A canvas widget that draws a saturation/lightness picker area with drag support.

use iced::widget::canvas::{self, Event};
use iced::{mouse, Rectangle, Renderer, Theme};

use crate::color::hsl_to_rgb;
use crate::message::Message;

/// A canvas program that draws a saturation/lightness picker area.
pub struct SaturationLightnessBox {
    pub hue: f32,
    pub saturation: f32,
    pub lightness: f32,
}

/// State to track if the user is currently dragging.
#[derive(Default)]
pub struct SLBoxState {
    is_dragging: bool,
}

impl SaturationLightnessBox {
    /// Convert cursor position (relative to bounds) to saturation/lightness values.
    fn position_to_sl(bounds: Rectangle, position: iced::Point) -> (f32, f32) {
        let saturation = (position.x / bounds.width).clamp(0.0, 1.0);
        // Invert Y: top = 1.0 (light), bottom = 0.0 (dark)
        let lightness = (1.0 - position.y / bounds.height).clamp(0.0, 1.0);

        (saturation, lightness)
    }
}

impl canvas::Program<Message> for SaturationLightnessBox {
    type State = SLBoxState;

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
                let (s, l) = Self::position_to_sl(bounds, cursor_position);
                Some(
                    canvas::Action::publish(Message::PickerSLChanged(s, l))
                        .and_capture(),
                )
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                let (s, l) = Self::position_to_sl(bounds, cursor_position);
                Some(
                    canvas::Action::publish(Message::PickerSLChanged(s, l))
                        .and_capture(),
                )
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.is_dragging => {
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
