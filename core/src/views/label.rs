use crate::{Context, Handle, LocalizedStringKey, View};

pub struct Label;

impl Label {
    pub fn new<'a, 'b, 'c>(cx: &'a mut Context<'b>, text: impl LocalizedStringKey<'c>) -> Handle<'a, 'b, Self> {
        Self {}.build2(cx, |_| {}).text(text.key())
    }
}

impl View<'_> for Label {
    fn element(&self) -> Option<String> {
        Some("label".to_string())
    }
}
