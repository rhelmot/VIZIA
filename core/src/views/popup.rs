use morphorm::PositionType;

use crate::{
    Binding, Context, Data, Handle, Lens, Model, PropSet, View, Visibility,
};

#[derive(Debug, Default, Data, Lens, Clone)]
pub struct PopupData {
    is_open: bool,
}

impl Model for PopupData {
    fn event(&mut self, _: &mut Context, event: &mut crate::Event) {
        if let Some(popup_event) = event.message.downcast() {
            match popup_event {
                PopupEvent::Open => {
                    self.is_open = true;
                    event.consume();
                }

                PopupEvent::Close => {
                    self.is_open = false;
                    event.consume();
                }

                PopupEvent::Switch => {
                    self.is_open ^= true;
                    event.consume();
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum PopupEvent {
    Open,
    Close,
    Switch,
}

pub struct Popup {}

impl Popup {
    pub fn new<'a, 'b, F>(cx: &'a mut Context<'b>, builder: F) -> Handle<'a, 'b, Self>
    where
        F: 'static + Fn(&mut Context<'b>),
    {
        Self {}
            .build2(cx, |cx| {
                Binding::new(cx, PopupData::is_open, move |cx, flag| {
                    let is_open = *flag.get(cx);

                    cx.current.set_visibility(
                        cx,
                        if is_open { Visibility::Visible } else { Visibility::Invisible },
                    );

                    (builder)(cx);
                });

                // cx.add_listener(|_: &mut Self, cx, event| {
                //     if let Some(popup_data) = cx.data::<PopupData>() {
                //         if let Some(window_event) = event.message.downcast() {
                //             match window_event {
                //                 WindowEvent::MouseDown(_) => {
                //                     if popup_data.is_open {
                //                         if event.origin != cx.current {
                //                             if !cx.current.is_over(cx) {
                //                                 cx.emit(PopupEvent::Close);
                //                                 event.consume();
                //                             }
                //                         }
                //                     }
                //                 }

                //                 WindowEvent::KeyDown(ref code, _) => {
                //                     if popup_data.is_open {
                //                         if *code == Code::Escape {
                //                             cx.emit(PopupEvent::Close);
                //                         }
                //                     }
                //                 }

                //                 _ => {}
                //             }
                //         }
                //     }
                // });
            })
            .position_type(PositionType::SelfDirected)
            .z_order(100)
    }
}

impl View<'_> for Popup {
    fn element(&self) -> Option<String> {
        Some("popup".to_string())
    }
}
