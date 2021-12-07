use glutin::{dpi::*, window::WindowId};
use glutin::event_loop::{EventLoop, EventLoopWindowTarget};
use glutin::window::{WindowBuilder};

use morphorm::PositionType;
use raw_window_handle::{RawWindowHandle, HasRawWindowHandle};

use crate::{View, Event, Context, Entity, AppEvent, WindowDescription};

pub struct RawWindow {
    pub id: Option<WindowId>,
    pub window: Option<glutin::window::Window>,
    pub handle: Option<RawWindowHandle>,
    pub builder: Option<Box<dyn Fn(&mut Context, &mut Self)>>,
    pub window_description: WindowDescription,
}

impl RawWindow {
    pub fn new<F>(cx: &mut Context, window_description: WindowDescription, builder: F) -> Entity 
    where F: 'static + Fn(&mut Context, &mut Self),
    {

        
        
        let raw_window = RawWindow {
            id: None,
            handle: None,
            window: None,
            builder: Some(Box::new(builder)),
            window_description,
        };

        let id = cx.entity_manager.create();
        cx.tree.add(id, Entity::root());
        cx.cache.add(id);
        cx.style.borrow_mut().add(id);
        cx.style.borrow_mut().position_type.insert(id, PositionType::SelfDirected);
        cx.views.insert(id, Box::new(raw_window));

        cx.event_queue.push_back(Event::new(AppEvent::CreateWindow).target(id).origin(id));
    
        id
    }

    pub fn create(&mut self,cx: &mut Context, event_loop: &EventLoopWindowTarget<Event>) -> WindowId {

        let window = WindowBuilder::new()
            .with_title(&self.window_description.title)
            .with_inner_size(PhysicalSize::new(
                self.window_description.inner_size.width,
                self.window_description.inner_size.height,
            ))
            .with_min_inner_size(PhysicalSize::new(
                self.window_description.min_inner_size.width,
                self.window_description.min_inner_size.height,
            ))
            .with_window_icon(if let Some(icon) = &self.window_description.icon {
                Some(
                    glutin::window::Icon::from_rgba(
                        icon.clone(),
                        self.window_description.icon_width,
                        self.window_description.icon_height,
                    )
                    .unwrap(),
                )
            } else {
                None
            }).build(event_loop).unwrap();


        self.id = Some(window.id());
        self.window = Some(window);

        if let Some(builder) = self.builder.take() {
            (builder)(cx, self);

            self.builder = Some(builder);
        }
        //self.handle = Some(window.raw_window_handle());

        self.id.unwrap()
    }
}

impl View for RawWindow {
    
}