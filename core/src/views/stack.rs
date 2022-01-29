use morphorm::LayoutType;

use crate::{Context, Handle, View};

/// A view which arranges its children into a vertical stack (column).
///
///
pub struct VStack {}

impl VStack {
    pub fn new<'a, 'b, F>(cx: &'a mut Context<'b>, content: F) -> Handle<'a, 'b, Self>
    where
        F: FnOnce(&mut Context),
    {
        Self {}.build2(cx, |cx| {
            (content)(cx);
        })
    }
}

impl View<'_> for VStack {
    fn element(&self) -> Option<String> {
        Some("vstack".to_string())
    }
}

/// A view which arranges its children into a horizontal stack (row).
pub struct HStack {}

impl HStack {
    pub fn new<'a, 'b, F>(cx: &'a mut Context<'b>, content: F) -> Handle<'a, 'b, Self>
    where
        F: FnOnce(&mut Context<'b>),
    {
        Self {}
            .build2(cx, |cx| {
                (content)(cx);
            })
            .layout_type(LayoutType::Row)
    }
}

impl View<'_> for HStack {
    fn element(&self) -> Option<String> {
        Some("hstack".to_string())
    }
}

/// A view which overlays its children on top of each other.
pub struct ZStack {}

impl ZStack {
    pub fn new<'a, 'b, F>(cx: &'a mut Context<'b>, content: F) -> Handle<'a, 'b, Self>
    where
        F: 'static + FnOnce(&mut Context<'b>),
    {
        Self {}.build2(cx, |cx| {
            (content)(cx);
        })
    }
}

impl View<'_> for ZStack {
    fn element(&self) -> Option<String> {
        Some("zstack".to_string())
    }
}
