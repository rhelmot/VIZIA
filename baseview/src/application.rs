//use crate::event_manager::EventManager;
use crate::window::ViziaWindow;
use crate::Renderer;
use baseview::{WindowHandle, WindowScalePolicy};
use femtovg::Canvas;
use raw_window_handle::HasRawWindowHandle;
use vizia_core::{apply_inline_inheritance, apply_shared_inheritance, TreeDepthIterator, TreeExt};
use vizia_core::{MouseButton, MouseButtonState};
//use vizia_core::WindowWidget;
use vizia_core::{
    apply_clipping, apply_hover, apply_styles, apply_text_constraints, apply_transform,
    apply_visibility, apply_z_ordering, geometry_changed, Context, Display, Entity, EventManager,
    FontOrId, Modifiers, Units, Visibility, WindowEvent, WindowSize,
};
use vizia_core::{BoundingBox, Event, Propagation, WindowDescription};

pub struct Application<F>
where
    F: Fn(&mut Context),
    F: 'static + Send,
{
    app: F,
    window_description: WindowDescription,
    on_idle: Option<Box<dyn Fn(&mut Context) + Send>>,
}

impl<F> Application<F>
where
    F: Fn(&mut Context),
    F: 'static + Send,
{
    pub fn new(window_description: WindowDescription, app: F) -> Self {
        Self { app, window_description, on_idle: None }
    }

    /// Open a new window that blocks the current thread until the window is destroyed.
    ///
    /// Do **not** use this in the context of audio plugins, unless it is compiled as a
    /// standalone application.
    ///
    /// * `app` - The Tuix application builder.
    pub fn run(self) {
        ViziaWindow::open_blocking(self.window_description, self.app, self.on_idle)
    }

    /// Open a new child window.
    ///
    /// This function does **not** block the current thread. This is only to be
    /// used in the context of audio plugins.
    ///
    /// * `parent` - The parent window.
    /// * `app` - The Tuix application builder.
    pub fn open_parented<P: HasRawWindowHandle>(self, parent: &P) -> WindowHandle {
        ViziaWindow::open_parented(parent, self.window_description, self.app, self.on_idle)
    }

    /// Open a new window as if it had a parent window.
    ///
    /// This function does **not** block the current thread. This is only to be
    /// used in the context of audio plugins.
    ///
    /// * `app` - The Tuix application builder.
    pub fn open_as_if_parented(self) -> WindowHandle {
        ViziaWindow::open_as_if_parented(self.window_description, self.app, self.on_idle)
    }

    /// Takes a closure which will be called at the end of every loop of the application.
    ///
    /// The callback provides a place to run 'idle' processing and happens at the end of each loop but before drawing.
    /// If the callback pushes events into the queue in state then the event loop will re-run. Care must be taken not to
    /// push events into the queue every time the callback runs unless this is intended.
    ///
    /// # Example
    /// ```no_run
    /// # use vizia_core::*;
    /// # use vizia_baseview::Application;
    /// Application::new(WindowDescription::new(), |cx|{
    ///     // Build application here
    /// })
    /// .on_idle(|cx|{
    ///     // Code here runs at the end of every event loop after OS and tuix events have been handled
    /// })
    /// .run();
    /// ```
    pub fn on_idle<I: 'static + Fn(&mut Context) + Send>(mut self, callback: I) -> Self {
        self.on_idle = Some(Box::new(callback));

        self
    }
}

pub(crate) struct ApplicationRunner {
    context: Context,
    event_manager: EventManager,
    canvas: Canvas<Renderer>,
    should_redraw: bool,
    scale_policy: WindowScalePolicy,
    scale_factor: f64,
}

impl ApplicationRunner {
    pub fn new(mut context: Context, win_desc: WindowDescription, renderer: Renderer) -> Self {
        let event_manager = EventManager::new();

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

        // // TODO: Get scale policy from `win_desc`.
        let scale_policy = WindowScalePolicy::SystemScaleFactor;

        // // Assume scale for now until there is an event with a new one.
        // let scale = match scale_policy {
        //     WindowScalePolicy::ScaleFactor(scale) => scale,
        //     WindowScalePolicy::SystemScaleFactor => 1.0,
        // };

        let scale = 1.0;

        let logical_size = win_desc.inner_size;
        let physical_size = WindowSize {
            width: (logical_size.width as f64 * scale).round() as u32,
            height: (logical_size.height as f64 * scale).round() as u32,
        };

        canvas.set_size(physical_size.width, physical_size.height, 1.0);

        let regular_font = include_bytes!("../../fonts/Roboto-Regular.ttf");
        let bold_font = include_bytes!("../../fonts/Roboto-Bold.ttf");
        let icon_font = include_bytes!("../../fonts/entypo.ttf");
        let emoji_font = include_bytes!("../../fonts/OpenSansEmoji.ttf");
        let arabic_font = include_bytes!("../../fonts/amiri-regular.ttf");
        let material_font = include_bytes!("../../fonts/MaterialIcons-Regular.ttf");

        context.add_font_mem("roboto", regular_font);
        context.add_font_mem("roboto-bold", bold_font);
        context.add_font_mem("icons", icon_font);
        context.add_font_mem("emoji", emoji_font);
        context.add_font_mem("arabic", arabic_font);
        context.add_font_mem("material", material_font);

        context.style.default_font = "roboto".to_string();

        //canvas.scale(scale as f32, scale as f32);

        context.style.width.insert(Entity::root(), Units::Pixels(logical_size.width as f32));
        context.style.height.insert(Entity::root(), Units::Pixels(logical_size.height as f32));

        context.style.disabled.insert(Entity::root(), false);

        context.cache.set_width(Entity::root(), physical_size.width as f32);
        context.cache.set_height(Entity::root(), physical_size.height as f32);
        context.cache.set_opacity(Entity::root(), 1.0);

        let mut bounding_box = BoundingBox::default();
        bounding_box.w = logical_size.width as f32;
        bounding_box.h = logical_size.height as f32;

        context.cache.set_clip_region(Entity::root(), bounding_box);

        ApplicationRunner {
            event_manager,
            context,
            canvas,
            should_redraw: true,
            scale_policy,
            scale_factor: scale,
        }
    }

    /*
    pub fn get_window(&self) -> Entity {
        self.Entity::root()
    }

    pub fn get_state(&mut self) -> &mut State {
        &mut self.context
    }

    pub fn get_event_manager(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }
    */

    // pub fn update_data(&mut self) {
    //     // Data Updates
    //     let mut observers: Vec<Entity> = Vec::new();
    //     for model_list in self.context.data.model_data.dense.iter().map(|entry| &entry.value){
    //         for (_, model) in model_list.iter() {
    //             //println!("Lenses: {:?}", context.lenses.len());
    //             for (_, lens) in self.context.lenses.iter_mut() {
    //                 if lens.update(model) {
    //                     observers.extend(lens.observers().iter());
    //                 }
    //             }
    //         }
    //     }

    //     for observer in observers.iter() {
    //         if let Some(mut view) = self.context.views.remove(observer) {
    //             let prev = self.context.current;
    //             self.context.current = *observer;
    //             let prev_count = self.context.count;
    //             self.context.count = 0;
    //             view.body(&mut self.context);
    //             self.context.current = prev;
    //             self.context.count = prev_count;

    //             self.context.style.needs_redraw = true;

    //             self.context.views.insert(*observer, view);
    //         }
    //     }
    // }

    pub fn on_frame_update(&mut self) {
        //if let Some(mut window_view) = context.views.remove(&Entity::root()) {
        //if let Some(window) = window_view.downcast_mut::<Window>() {

        // Load resources
        for (name, font) in self.context.resource_manager.fonts.iter_mut() {
            match font {
                FontOrId::Font(data) => {
                    let id1 = self
                        .canvas
                        .add_font_mem(&data.clone())
                        .expect(&format!("Failed to load font file for: {}", name));
                    let id2 =
                        self.context.text_context.add_font_mem(&data.clone()).expect("failed");
                    if id1 != id2 {
                        panic!(
                            "Fonts in canvas must have the same id as fonts in the text context"
                        );
                    }
                    *font = FontOrId::Id(id1);
                }

                _ => {}
            }
        }

        //}

        //context.views.insert(Entity::root(), window_view);
        //}

        // Events
        while !self.context.event_queue.is_empty() {
            self.event_manager.flush_events(&mut self.context);
        }

        self.context.process_data_updates();
        self.context.process_visual_updates();

        if self.context.style.needs_redraw {
            //     // TODO - Move this to EventManager
            self.should_redraw = true;
            self.context.style.needs_redraw = false;
        }
    }

    pub fn render(&mut self) {
        let dpi_factor = 1.0; // TODO
        self.context.draw(&mut self.canvas, dpi_factor);
        self.should_redraw = false;
    }

    pub fn handle_event(&mut self, event: baseview::Event, should_quit: &mut bool) {
        if requests_exit(&event) {
            self.context.event_queue.push_back(Event::new(WindowEvent::WindowClose));
            *should_quit = true;
        }

        match event {
            baseview::Event::Mouse(event) => match event {
                baseview::MouseEvent::CursorMoved { position } => {
                    let cursorx = (position.x) as f32;
                    let cursory = (position.y) as f32;
                    self.context.dispatch_system_event(WindowEvent::MouseMove(cursorx, cursory));
                }
                baseview::MouseEvent::ButtonPressed(button) => {
                    let b = translate_mouse_button(button);
                    self.context.dispatch_system_event(WindowEvent::MouseDown(b));
                }
                baseview::MouseEvent::ButtonReleased(button) => {
                    let b = translate_mouse_button(button);
                    self.context.dispatch_system_event(WindowEvent::MouseUp(b));
                }
                baseview::MouseEvent::WheelScrolled(scroll_delta) => {
                    let (lines_x, lines_y) = match scroll_delta {
                        baseview::ScrollDelta::Lines { x, y } => (x, y),
                        baseview::ScrollDelta::Pixels { x, y } => (
                            if x < 0.0 {
                                -1.0
                            } else if x > 1.0 {
                                1.0
                            } else {
                                0.0
                            },
                            if y < 0.0 {
                                -1.0
                            } else if y > 1.0 {
                                1.0
                            } else {
                                0.0
                            },
                        ),
                    };

                    self.context.dispatch_system_event(WindowEvent::MouseScroll(lines_x, lines_y));
                }
                _ => {}
            },
            baseview::Event::Keyboard(event) => {
                use keyboard_types::Code;

                let (s, pressed) = match event.state {
                    keyboard_types::KeyState::Down => (MouseButtonState::Pressed, true),
                    keyboard_types::KeyState::Up => (MouseButtonState::Released, false),
                };

                match event.code {
                    Code::ShiftLeft | Code::ShiftRight => {
                        self.context.modifiers.set(Modifiers::SHIFT, pressed)
                    }
                    Code::ControlLeft | Code::ControlRight => {
                        self.context.modifiers.set(Modifiers::CTRL, pressed)
                    }
                    Code::AltLeft | Code::AltRight => {
                        self.context.modifiers.set(Modifiers::ALT, pressed)
                    }
                    Code::MetaLeft | Code::MetaRight => {
                        self.context.modifiers.set(Modifiers::LOGO, pressed)
                    }
                    _ => (),
                }


                match s {
                    MouseButtonState::Pressed => {
                        self.context.dispatch_system_event(WindowEvent::KeyDown(
                            event.code,
                            Some(event.key.clone()),
                        ));

                        if let keyboard_types::Key::Character(written) = &event.key {
                            for chr in written.chars() {
                                self.context.dispatch_system_event(WindowEvent::CharInput(chr));
                            }
                        }
                    }

                    MouseButtonState::Released => {
                        self.context.dispatch_system_event(WindowEvent::KeyUp(
                            event.code,
                            Some(event.key.clone()),
                        ));
                    }
                }
            }
            baseview::Event::Window(event) => match event {
                baseview::WindowEvent::Focused => {
                    self.context.style.needs_restyle = true;
                    self.context.style.needs_relayout = true;
                    self.context.style.needs_redraw = true;
                }
                baseview::WindowEvent::Resized(window_info) => {
                    self.scale_factor = match self.scale_policy {
                        WindowScalePolicy::ScaleFactor(scale) => scale,
                        WindowScalePolicy::SystemScaleFactor => window_info.scale(),
                    };

                    self.scale_factor = 1.0;

                    let logical_size = (
                        (window_info.physical_size().width as f64 / self.scale_factor),
                        (window_info.physical_size().height as f64 / self.scale_factor),
                    );

                    let physical_size =
                        (window_info.physical_size().width, window_info.physical_size().height);

                    self.context
                        .style
                        .width
                        .insert(Entity::root(), Units::Pixels(logical_size.0 as f32));
                    self.context
                        .style
                        .height
                        .insert(Entity::root(), Units::Pixels(logical_size.1 as f32));

                    self.context.cache.set_width(Entity::root(), physical_size.0 as f32);
                    self.context.cache.set_height(Entity::root(), physical_size.1 as f32);

                    let mut bounding_box = BoundingBox::default();
                    bounding_box.w = physical_size.0 as f32;
                    bounding_box.h = physical_size.1 as f32;

                    self.context.cache.set_clip_region(Entity::root(), bounding_box);

                    self.context.style.needs_restyle = true;
                    self.context.style.needs_relayout = true;
                    self.context.style.needs_redraw = true;
                }
                baseview::WindowEvent::WillClose => {
                    self.context.event_queue.push_back(Event::new(WindowEvent::WindowClose));
                }
                _ => {}
            },
        }
    }

    pub fn rebuild(&mut self, builder: &Option<Box<dyn Fn(&mut Context) + Send>>) {
        if self.context.enviroment.needs_rebuild {
            self.context.current = Entity::root();
            self.context.count = 0;
            if let Some(builder) = &builder {
                (builder)(&mut self.context);
            }
            self.context.enviroment.needs_rebuild = false;
        }
    }

    pub fn handle_idle(&mut self, on_idle: &Option<Box<dyn Fn(&mut Context) + Send>>) {
        // if let Some(idle_callback) = on_idle {
        //     (idle_callback)(&mut self.context);
        // }

        if let Some(idle_callback) = on_idle {
            self.context.current = Entity::root();
            self.context.count = 0;
            (idle_callback)(&mut self.context);
        }
    }
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn requests_exit(event: &baseview::Event) -> bool {
    match event {
        baseview::Event::Window(baseview::WindowEvent::WillClose) => true,
        #[cfg(target_os = "macos")]
        baseview::Event::Keyboard(event) => {
            if event.code == keyboard_types::Code::KeyQ {
                if event.modifiers == keyboard_types::Modifiers::META {
                    if event.state == keyboard_types::KeyState::Down {
                        return true;
                    }
                }
            }

            false
        }
        _ => false,
    }
}

fn translate_mouse_button(button: baseview::MouseButton) -> MouseButton {
    match button {
        baseview::MouseButton::Left => MouseButton::Left,
        baseview::MouseButton::Right => MouseButton::Right,
        baseview::MouseButton::Middle => MouseButton::Middle,
        baseview::MouseButton::Other(id) => MouseButton::Other(id as u16),
        baseview::MouseButton::Back => MouseButton::Other(4),
        baseview::MouseButton::Forward => MouseButton::Other(5),
    }
}
