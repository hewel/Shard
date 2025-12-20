//! A canvas widget that draws an alpha slider with checkerboard background and drag support.

use iced::widget::canvas::{self, Event};
use iced::{mouse, Rectangle, Renderer, Theme};

use super::draw_checkerboard;
use crate::message::Message;

/// A canvas program that draws an alpha slider with checkerboard background.
pub struct AlphaBar {
    pub color: iced::Color,
    pub alpha: f32,
}

/// State to track if the user is currently dragging.
#[derive(Default)]
pub struct AlphaBarState {
    is_dragging: bool,
}

impl AlphaBar {
    /// Convert cursor x position to alpha value (0-1).
    fn position_to_alpha(bounds: Rectangle, position: iced::Point) -> f32 {
        (position.x / bounds.width).clamp(0.0, 1.0)
    }
}

impl canvas::Program<Message> for AlphaBar {
    type State = AlphaBarState;

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
                let alpha = Self::position_to_alpha(bounds, cursor_position);
                Some(canvas::Action::publish(Message::PickerAlphaChanged(alpha)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                let alpha = Self::position_to_alpha(bounds, cursor_position);
                Some(canvas::Action::publish(Message::PickerAlphaChanged(alpha)).and_capture())
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
        if state.is_dragging || cursor.is_over(bounds) {
            mouse::Interaction::Pointer
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

        // Draw checkerboard background
        draw_checkerboard(&mut frame, bounds, 8.0);

        // Draw gradient from transparent to opaque
        let step = bounds.width / 100.0;
        for i in 0..100 {
            let alpha = i as f32 / 100.0;
            let color = iced::Color {
                r: self.color.r,
                g: self.color.g,
                b: self.color.b,
                a: alpha,
            };
            frame.fill_rectangle(
                iced::Point::new(i as f32 * step, 0.0),
                iced::Size::new(step.ceil(), bounds.height),
                color,
            );
        }

        // Draw indicator for current alpha
        let indicator_x = self.alpha * bounds.width;
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
