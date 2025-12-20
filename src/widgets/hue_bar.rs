//! A canvas widget that draws a hue spectrum bar with drag support.

use iced::widget::canvas::{self, Event};
use iced::{mouse, Rectangle, Renderer, Theme};

use crate::message::Message;
use crate::snippet::hsl_to_rgb;

/// A canvas program that draws a hue spectrum bar.
pub struct HueBar {
    pub current_hue: f32,
}

/// State to track if the user is currently dragging.
#[derive(Default)]
pub struct HueBarState {
    is_dragging: bool,
}

impl HueBar {
    /// Convert cursor x position to hue value (0-360).
    fn position_to_hue(bounds: Rectangle, position: iced::Point) -> f32 {
        ((position.x / bounds.width) * 360.0).clamp(0.0, 360.0)
    }
}

impl canvas::Program<Message> for HueBar {
    type State = HueBarState;

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
                let hue = Self::position_to_hue(bounds, cursor_position);
                Some(canvas::Action::publish(Message::PickerHueChanged(hue)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                let hue = Self::position_to_hue(bounds, cursor_position);
                Some(canvas::Action::publish(Message::PickerHueChanged(hue)).and_capture())
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
