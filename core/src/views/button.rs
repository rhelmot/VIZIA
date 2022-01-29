use crate::{Context, Event, View};
use crate::{Handle, MouseButton, WindowEvent};

/// A simple push button with an action and a label.
///
///
pub struct Button {
    action: Option<Box<dyn Fn(&mut Context)>>,
}

impl Button {
    pub fn new<'a, 'b, A, L, Label>(cx: &'a mut Context<'b>, action: A, label: L) -> Handle<'a, 'b, Self>
    where
        A: 'b + Fn(&'a mut Context<'b>),
        L: Fn(&'a mut Context<'b>) -> Handle<'a, 'b, Label>,
        Label: View,
    {
        Self { action: Some(Box::new(action)) }.build2(cx, move |cx| {
            (label)(cx);
        })
    }
}

impl View for Button {
    fn element(&self) -> Option<String> {
        Some("button".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                    if let Some(callback) = self.action.take() {
                        (callback)(cx);

                        self.action = Some(callback);
                    }
                }

                _ => {}
            }
        }
    }
}
