use std::marker::PhantomData;

use morphorm::GeometryChanged;

use crate::{Context, Entity, Event, Handle, MouseButton, View, ViewHandler, WindowEvent};

// Press
pub struct Press<'b, V> {
    view: Box<dyn ViewHandler<'b> + 'b>,
    action: Option<Box<dyn for<'c> Fn(&'c mut Context<'b>) + 'b>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b> + 'b> Press<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Press<'b, V>>
    where
        F: for<'c> Fn(&'c mut Context<'b>) + 'b,
    {
        if let Some(view) = handle.cx.views.remove(&handle.entity) {
            let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

            handle.cx.views.insert(handle.entity, Box::new(item));
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Press<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                    //if event.target == cx.current {
                    if let Some(action) = self.action.take() {
                        (action)(cx);

                        self.action = Some(action);
                    }

                    //cx.captured = cx.current;
                    //}
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

// Release
pub struct Release<'b, V: View<'b>> {
    view: Box<dyn ViewHandler<'b> + 'b>,
    action: Option<Box<dyn Fn(&mut Context)>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b>> Release<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Release<'b, V>>
    where
        F: 'static + Fn(&mut Context),
    {
        if let Some(mut view) = handle.cx.views.remove(&handle.entity) {
            if view.downcast_ref::<V>().is_some() {
                let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

                handle.cx.views.insert(handle.entity, Box::new(item));
            } else {
                if let Some(release) = view.downcast_mut::<Release<V>>() {
                    release.action = Some(Box::new(action));
                }
                handle.cx.views.insert(handle.entity, view);
            }
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Release<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                    if event.target == cx.current {
                        if let Some(action) = self.action.take() {
                            (action)(cx);

                            self.action = Some(action);
                        }

                        cx.captured = Entity::null();
                    }
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

// Hover
pub struct Hover<'b, V: View<'b>> {
    view: Box<dyn ViewHandler<'b>>,
    action: Option<Box<dyn Fn(&mut Context)>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b>> Hover<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Hover<'b, V>>
    where
        F: 'static + Fn(&mut Context),
    {
        if let Some(mut view) = handle.cx.views.remove(&handle.entity) {
            if view.downcast_ref::<V>().is_some() {
                let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

                handle.cx.views.insert(handle.entity, Box::new(item));
            } else {
                if let Some(hover) = view.downcast_mut::<Hover<V>>() {
                    hover.action = Some(Box::new(action));
                }
                handle.cx.views.insert(handle.entity, view);
            }
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Hover<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseEnter => {
                    if event.target == cx.current {
                        if let Some(action) = self.action.take() {
                            (action)(cx);

                            self.action = Some(action);
                        }
                    }
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

// Hover
pub struct Over<'b, V: View<'b>> {
    view: Box<dyn ViewHandler<'b>>,
    action: Option<Box<dyn Fn(&mut Context)>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b>> Over<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Over<'b, V>>
    where
        F: 'static + Fn(&mut Context),
    {
        if let Some(mut view) = handle.cx.views.remove(&handle.entity) {
            if view.downcast_ref::<V>().is_some() {
                let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

                handle.cx.views.insert(handle.entity, Box::new(item));
            } else {
                if let Some(over) = view.downcast_mut::<Over<V>>() {
                    over.action = Some(Box::new(action));
                }
                handle.cx.views.insert(handle.entity, view);
            }
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Over<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseOver => {
                    if let Some(action) = self.action.take() {
                        (action)(cx);

                        self.action = Some(action);
                    }
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

// Leave
pub struct Leave<'b, V: View<'b>> {
    view: Box<dyn ViewHandler<'b>>,
    action: Option<Box<dyn Fn(&mut Context)>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b>> Leave<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Leave<'b, V>>
    where
        F: 'static + Fn(&mut Context),
    {
        if let Some(mut view) = handle.cx.views.remove(&handle.entity) {
            if view.downcast_ref::<V>().is_some() {
                let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

                handle.cx.views.insert(handle.entity, Box::new(item));
            } else {
                if let Some(hover) = view.downcast_mut::<Leave<V>>() {
                    hover.action = Some(Box::new(action));
                }
                handle.cx.views.insert(handle.entity, view);
            }
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Leave<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseLeave => {
                    if event.target == cx.current {
                        if let Some(action) = self.action.take() {
                            (action)(cx);

                            self.action = Some(action);
                        }
                    }
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

// Move
pub struct Move<'b, V: View<'b>> {
    view: Box<dyn ViewHandler<'b>>,
    action: Option<Box<dyn Fn(&mut Context, f32, f32)>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b>> Move<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Move<'b, V>>
    where
        F: 'static + Fn(&mut Context, f32, f32),
    {
        if let Some(mut view) = handle.cx.views.remove(&handle.entity) {
            if view.downcast_ref::<V>().is_some() {
                let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

                handle.cx.views.insert(handle.entity, Box::new(item));
            } else {
                if let Some(hover) = view.downcast_mut::<Move<V>>() {
                    hover.action = Some(Box::new(action));
                }
                handle.cx.views.insert(handle.entity, view);
            }
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Move<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::MouseMove(x, y) => {
                    if let Some(action) = self.action.take() {
                        (action)(cx, *x, *y);

                        self.action = Some(action);
                    }
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

// Geo
pub struct Geo<'b, V: View<'b>> {
    view: Box<dyn ViewHandler<'b>>,
    action: Option<Box<dyn Fn(&mut Context, GeometryChanged)>>,

    p: PhantomData<V>,
}

impl<'b, V: View<'b>> Geo<'b, V> {
    pub fn new<'a, F>(handle: Handle<'a, 'b, V>, action: F) -> Handle<'a, 'b, Geo<'b, V>>
    where
        F: 'static + Fn(&mut Context, GeometryChanged),
    {
        if let Some(mut view) = handle.cx.views.remove(&handle.entity) {
            if view.downcast_ref::<V>().is_some() {
                let item = Self { view, action: Some(Box::new(action)), p: Default::default() };

                handle.cx.views.insert(handle.entity, Box::new(item));
            } else {
                if let Some(geo) = view.downcast_mut::<Geo<V>>() {
                    geo.action = Some(Box::new(action));
                }
                handle.cx.views.insert(handle.entity, view);
            }
        }

        Handle { entity: handle.entity, p: Default::default(), cx: handle.cx }
    }
}

impl<'b, V: View<'b>> View<'b> for Geo<'b, V> {
    fn element(&self) -> Option<String> {
        self.view.element()
    }

    fn event(&mut self, cx: &mut Context<'b>, event: &mut Event) {
        self.view.event(cx, event);

        if let Some(window_event) = event.message.downcast() {
            match window_event {
                WindowEvent::GeometryChanged(geo) => {
                    if event.target == cx.current {
                        if let Some(action) = self.action.take() {
                            (action)(cx, *geo);

                            self.action = Some(action);
                        }
                    }
                }

                _ => {}
            }
        }
    }

    fn draw(&self, cx: &mut Context<'b>, canvas: &mut crate::Canvas) {
        self.view.draw(cx, canvas);
    }
}

pub trait Actions<'a, 'b> {
    type View: View<'b>;
    fn on_press<F>(self, action: F) -> Handle<'a, 'b, Press<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context);

    fn on_release<F>(self, action: F) -> Handle<'a, 'b, Release<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context);

    fn on_hover<F>(self, action: F) -> Handle<'a, 'b, Hover<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context);

    fn on_over<F>(self, action: F) -> Handle<'a, 'b, Over<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context);

    fn on_leave<F>(self, action: F) -> Handle<'a, 'b, Leave<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context);

    fn on_move<F>(self, action: F) -> Handle<'a, 'b, Move<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context, f32, f32);

    fn on_geo_changed<F>(self, action: F) -> Handle<'a, 'b, Geo<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context, GeometryChanged);
}

impl<'a, 'b, V: View<'b>> Actions<'a, 'b> for Handle<'a, 'b, V> {
    type View = V;
    fn on_press<F>(self, action: F) -> Handle<'a, 'b, Press<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context),
    {
        Press::new(self, action)
    }

    fn on_release<F>(self, action: F) -> Handle<'a, 'b, Release<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context),
    {
        Release::new(self, action)
    }

    fn on_hover<F>(self, action: F) -> Handle<'a, 'b, Hover<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context),
    {
        Hover::new(self, action)
    }

    fn on_over<F>(self, action: F) -> Handle<'a, 'b, Over<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context),
    {
        Over::new(self, action)
    }

    fn on_leave<F>(self, action: F) -> Handle<'a, 'b, Leave<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context),
    {
        Leave::new(self, action)
    }

    fn on_move<F>(self, action: F) -> Handle<'a, 'b, Move<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context, f32, f32),
    {
        Move::new(self, action)
    }

    fn on_geo_changed<F>(self, action: F) -> Handle<'a, 'b, Geo<'b, Self::View>>
    where
        F: 'static + Fn(&mut Context, GeometryChanged),
    {
        Geo::new(self, action)
    }
}

// pub trait ViewModifers {
//     type View: View;

//     fn overlay<B>(self, cx: &mut Context, builder: B) -> Handle<Self::View>
//     where
//         B: 'static + FnOnce(&mut Context);
// }

// impl<'a, V: View> ViewModifers for Handle<'a, V> {
//     type View = V;
//     fn overlay<B>(self, cx: &mut Context, builder: B) -> Handle<Self::View>
//     where
//         B: 'static + FnOnce(&mut Context),
//     {
//         (builder)(cx);

//         self
//     }
// }
