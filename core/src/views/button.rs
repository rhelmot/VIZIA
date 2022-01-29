
use crate::{Context, Event, View};
use crate::{Handle, MouseButton, WindowEvent};

/// A simple push button with an action and a label.
///
///
pub struct Button<'b> {
    action: Option<Box<dyn Fn(&mut Context<'b>) + 'b>>,
}

impl <'b> Button<'b> {
    pub fn new<'a, A, L, Label>(cx: &'a mut Context<'b>, action: A, label: L) -> Handle<'a, 'b, Self>
    where
        A: 'b + Fn(&mut Context<'b>),
        L: for<'c> Fn(&'c mut Context<'b>) -> Handle<'c, 'b, Label>,
        Label: View<'b>,
    {
        Self { action: Some(Box::new(action)) }.build2(cx, move |cx| {
            (label)(cx);
        })
    }
}

impl <'b> View<'b> for Button<'b> {
    fn element(&self) -> Option<String> {
        Some("button".to_string())
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
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
