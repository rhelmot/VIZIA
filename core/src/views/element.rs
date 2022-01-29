use crate::{Context, Handle, View};

/// A basic element with no interactivity.
///
///
pub struct Element {}

impl Element {
    pub fn new<'a, 'b>(cx: &'a mut Context<'b>) -> Handle<'a, 'b, Self> {
        Self {}.build(cx)
    }
}

impl View<'_> for Element {
    fn element(&self) -> Option<String> {
        Some("element".to_string())
    }
}
