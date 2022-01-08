use std::any::TypeId;
use morphorm::LayoutType;
use crate::{Canvas, Checkbox, Context, Entity, Event, Handle, Propagation, TreeExt, View, WindowEvent};

pub struct Group {
    target: Option<Entity>,
}

impl Group {
    pub fn new<F>(cx: &mut Context, builder: F) -> Handle<Group>
    where
        F: 'static + FnOnce(&mut Context)
    {
        let mut result = Group {
            target: None,
        }.build2(cx, builder)
            .layout_type(LayoutType::Row);

        let mut found_child = None;
        for child in result.entity.branch_iter(&cx.tree) {
            if let Some(view) = cx.views.get(&child) {
                let view_id = view.type_id();
                // expand this as we add more input views that respond to focus!
                if view_id == TypeId::of::<Checkbox>() {
                    found_child = Some(child);
                    break;
                }
            }
        }
        if let Some(found_child) = found_child {
            result.target(cx, found_child);
        }
        result
    }
}

impl Handle<Group> {
    pub fn target(&mut self, cx: &mut Context, target: Entity) -> &mut Self {
        if let Some(view) = cx.views.get_mut(&self.entity) {
            if let Some(this) = view.downcast_mut::<Group>() {
                this.target = Some(target);
            }
        }
        self
    }
}

impl View for Group {
    fn element(&self) -> Option<String> {
        Some("group".to_owned())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(target) = self.target {
            if !event.target.is_child_of(&cx.tree, target) {
                if let Some(window_event) = event.message.downcast() {
                    match window_event {
                        WindowEvent::MouseDown(_) | WindowEvent::MouseUp(_) => {
                            cx.event_queue.push_back(Event::new(window_event.clone())
                                .target(target)
                                .origin(event.origin)
                                .propagate(Propagation::Direct));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
