use std::marker::PhantomData;

use crate::Units::*;
use crate::{
    Actions, Binding, Button, Color, Context, Data, Field, Handle, Label, Lens, Model, Overflow,
    Popup, PopupData, PopupEvent, View,
};

pub struct Picker<L>
where
    L: Lens,
{
    lens: PhantomData<L>,
}

impl<L> Picker<L>
where
    L: 'static + Lens,
    <L as Lens>::Source: 'static,
    <L as Lens>::Target: Data,
{
    pub fn new<'a, 'b, F>(cx: &'a mut Context<'b>, lens: L, builder: F) -> Handle<'a, 'b, Self>
    where
        F: 'static + Fn(&'a mut Context<'b>, Field<L>),
    {
        Self { lens: PhantomData::default() }
            .build2(cx, move |cx| {
                Binding::new(cx, lens, move |cx, option| {
                    (builder)(cx, option);
                });
            })
            .size(Auto)
    }
}

impl<L: Lens> View<'_> for Picker<L> {
    fn element(&self) -> Option<String> {
        Some("picker".to_string())
    }
}

#[derive(Debug)]
pub enum PickerEvent<T: std::fmt::Debug> {
    SetOption(T),
}

pub struct PickerItem {}

impl PickerItem {
    pub fn new<'a, 'b, T: 'static + std::fmt::Debug + Clone + PartialEq + Send>(
        cx: &'a mut Context<'b>,
        text: &'static str,
        option: T,
        value: T,
    ) -> Handle<'a, 'b, Self> {
        Self {}
            .build2(cx, move |cx| {
                let opt = option.clone();
                Button::new(
                    cx,
                    move |cx| cx.emit(PickerEvent::SetOption(option.clone())),
                    move |cx| Label::new(cx, text),
                )
                .background_color(if value == opt {
                    Color::red()
                } else {
                    Color::blue()
                });
            })
            .height(Auto)
    }
}

impl View<'_> for PickerItem {}

pub struct Dropdown {}

impl Dropdown {
    pub fn new<'a, 'b, F, L, Label>(cx: &'a mut Context<'b>, label: L, builder: F) -> Handle<'a, 'b, Self>
    where
        L: 'static + Fn(&'a mut Context<'b>) -> Handle<'a, 'b, Label>,
        F: 'static + Fn(&mut Context),
        Label: 'static + View<'b>,
    {
        Self {}
            .build2(cx, move |cx| {
                if cx.data::<PopupData>().is_none() {
                    PopupData::default().build(cx);
                }

                (label)(cx).class("title").on_press(|cx| cx.emit(PopupEvent::Switch));

                Popup::new(cx, move |cx| {
                    (builder)(cx);
                })
                .top(Percentage(100.0))
                .width(Stretch(1.0))
                .height(Auto)
                .overflow(Overflow::Visible);
            })
            .size(Auto)
    }
}

impl View<'_> for Dropdown {
    fn element(&self) -> Option<String> {
        Some("dropdown".to_string())
    }
}
