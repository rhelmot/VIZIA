use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::time::Instant;

#[cfg(feature = "clipboard")]
use copypasta::ClipboardContext;
use femtovg::TextContext;
// use fluent_bundle::{FluentBundle, FluentResource};
// use unic_langid::LanguageIdentifier;

use crate::{
    storage::sparse_set::SparseSet, CachedData, Entity, Enviroment, Event, FontOrId, IdManager,
    Message, ModelDataStore, Modifiers, MouseState, Propagation, ResourceManager, Style, Tree,
    TreeExt, View, ViewHandler, TimedEvent, TimedEventHandle,
};

static DEFAULT_THEME: &str = include_str!("default_theme.css");

pub struct Context {
    pub entity_manager: IdManager<Entity>,
    pub tree: Tree,
    pub current: Entity,
    pub count: usize,
    pub views: HashMap<Entity, Box<dyn ViewHandler>>,
    pub data: SparseSet<ModelDataStore>,
    pub event_queue: VecDeque<Event>,
    pub event_schedule: BinaryHeap<TimedEvent>,
    next_event_id: usize,
    pub listeners: HashMap<Entity, Box<dyn Fn(&mut dyn ViewHandler, &mut Context, &mut Event)>>,
    pub style: Style,
    pub cache: CachedData,

    pub enviroment: Enviroment,

    pub mouse: MouseState,
    pub modifiers: Modifiers,

    pub captured: Entity,
    pub hovered: Entity,
    pub focused: Entity,

    pub resource_manager: ResourceManager,

    pub text_context: TextContext,

    pub event_proxy: Option<Box<dyn EventProxy>>,

    #[cfg(feature = "clipboard")]
    pub clipboard: ClipboardContext,
}

impl Context {
    pub fn new() -> Self {
        let mut cache = CachedData::default();
        cache.add(Entity::root()).expect("Failed to add entity to cache");

        Self {
            entity_manager: IdManager::new(),
            tree: Tree::new(),
            current: Entity::root(),
            count: 0,
            views: HashMap::new(),
            data: SparseSet::new(),
            style: Style::default(),
            cache,
            enviroment: Enviroment::new(),
            event_queue: VecDeque::new(),
            event_schedule: BinaryHeap::new(),
            next_event_id: 0,
            listeners: HashMap::default(),
            mouse: MouseState::default(),
            modifiers: Modifiers::empty(),
            captured: Entity::null(),
            hovered: Entity::root(),
            focused: Entity::root(),
            resource_manager: ResourceManager::new(),
            text_context: TextContext::default(),

            event_proxy: None,

            #[cfg(feature = "clipboard")]
            clipboard: ClipboardContext::new().expect("Failed to init clipboard"),
        }
    }

    pub fn remove_children(&mut self, entity: Entity) {
        let children = entity.child_iter(&self.tree).collect::<Vec<_>>();
        for child in children.into_iter() {
            self.remove(child);
        }
    }

    /// Remove any extra children that were cached in the tree but are no longer required
    pub fn remove_trailing_children(&mut self) {
        while let Some(child) = self.tree.get_child(self.current, self.count) {
            self.remove(child);
            self.count += 1;
        }
    }

    pub fn remove(&mut self, entity: Entity) {
        let delete_list = entity.branch_iter(&self.tree).collect::<Vec<_>>();

        if !delete_list.is_empty() {
            self.style.needs_restyle = true;
            self.style.needs_relayout = true;
            self.style.needs_redraw = true;
        }

        for entity in delete_list.iter().rev() {
            // Remove from observers
            for entry in self.data.dense.iter_mut() {
                let model_list = &mut entry.value;
                for (_, model) in model_list.data.iter_mut() {
                    model.remove_observer(*entity);
                }
            }

            //println!("Removing: {}", entity);
            self.tree.remove(*entity).expect("");
            self.cache.remove(*entity);
            self.style.remove(*entity);
            self.data.remove(*entity);
            self.entity_manager.destroy(*entity);
            self.views.remove(entity);
        }
    }

    /// Get stored data from the context.
    pub fn data<T: 'static>(&self) -> Option<&T> {
        // return data for the static model
        if let Some(t) = ().as_any().downcast_ref::<T>() {
            return Some(t);
        }

        for entity in self.current.parent_iter(&self.tree) {
            //println!("Current: {} {:?}", entity, entity.parent(&self.tree));
            if let Some(data_list) = self.data.get(entity) {
                for (_, model) in data_list.data.iter() {
                    if let Some(data) = model.downcast_ref::<T>() {
                        return Some(data);
                    }
                }
            }
        }

        None
    }

    pub fn emit<M: Message>(&mut self, message: M) {
        self.event_queue.push_back(
            Event::new(message)
                .target(self.current)
                .origin(self.current)
                .propagate(Propagation::Up),
        );
    }

    pub fn emit_to<M: Message>(&mut self, target: Entity, message: M) {
        self.event_queue.push_back(
            Event::new(message).target(target).origin(self.current).propagate(Propagation::Direct),
        );
    }

    pub fn schedule_emit<M: Message>(&mut self, message: M, at: Instant) -> TimedEventHandle {
        self.schedule_event(
            Event::new(message)
                .target(self.current)
                .origin(self.current)
                .propagate(Propagation::Up), at)
    }

    pub fn schedule_emit_to<M: Message>(&mut self, message: M, target: Entity, at: Instant) -> TimedEventHandle {
        self.schedule_event(
            Event::new(message)
                .target(target)
                .origin(self.current)
                .propagate(Propagation::Direct), at)
    }

    fn schedule_event(&mut self, event: Event, at: Instant) -> TimedEventHandle {
        let handle = TimedEventHandle(self.next_event_id);
        self.event_schedule.push(
            TimedEvent {
                event,
                time: at,
                ident: handle,
            }
        );
        self.next_event_id += 1;
        handle
    }

    pub fn cancel_scheduled(&mut self, handle: TimedEventHandle) -> Result<(), ()> {
        // holy shit. why is every single useful method on BinaryHeap unstable.
        let orig_size = self.event_schedule.len();
        self.event_schedule = self.event_schedule
            .drain()
            .filter(|item| item.ident != handle)
            .collect();
        if self.event_schedule.len() != orig_size {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_listener<F, W>(&mut self, listener: F)
    where
        W: View,
        F: 'static + Fn(&mut W, &mut Context, &mut Event),
    {
        self.listeners.insert(
            self.current,
            Box::new(move |event_handler, context, event| {
                if let Some(widget) = event_handler.downcast_mut::<W>() {
                    (listener)(widget, context, event);
                }
            }),
        );
    }

    pub fn emit_trace<M: Message>(&mut self, message: M) {
        self.event_queue.push_back(
            Event::new(message)
                .target(self.current)
                .origin(self.current)
                .propagate(Propagation::Up)
                .trace(),
        );
    }

    /// Add a font from memory to the application
    pub fn add_font_mem(&mut self, name: &str, data: &[u8]) {
        // TODO - return error
        if self.resource_manager.fonts.contains_key(name) {
            println!("Font already exists");
            return;
        }

        self.resource_manager.fonts.insert(name.to_owned(), FontOrId::Font(data.to_vec()));
    }

    /// Sets the global default font for the application
    pub fn set_default_font(&mut self, name: &str) {
        self.style.default_font = name.to_string();
    }

    pub fn add_theme(&mut self, theme: &str) {
        self.resource_manager.themes.push(theme.to_owned());

        self.reload_styles().expect("Failed to reload styles");
    }

    pub fn remove_user_themes(&mut self) {
        self.resource_manager.themes.clear();

        self.add_theme(DEFAULT_THEME);
    }

    pub fn add_stylesheet(&mut self, path: &str) -> Result<(), std::io::Error> {
        let style_string = std::fs::read_to_string(path.clone())?;
        self.resource_manager.stylesheets.push(path.to_owned());
        self.style.parse_theme(&style_string);

        Ok(())
    }

    pub fn reload_styles(&mut self) -> Result<(), std::io::Error> {
        if self.resource_manager.themes.is_empty() && self.resource_manager.stylesheets.is_empty() {
            return Ok(());
        }

        self.style.remove_rules();

        self.style.rules.clear();

        self.style.remove_all();

        let mut overall_theme = String::new();

        // Reload the stored themes
        for (index, theme) in self.resource_manager.themes.iter().enumerate() {
            if !self.enviroment.include_default_theme && index == 0 {
                continue;
            }

            //self.style.parse_theme(theme);
            overall_theme += theme;
        }

        // Reload the stored stylesheets
        for stylesheet in self.resource_manager.stylesheets.iter() {
            let theme = std::fs::read_to_string(stylesheet)?;
            overall_theme += &theme;
        }

        self.style.parse_theme(&overall_theme);

        // self.enviroment.needs_rebuild = true;

        // Entity::root().restyle(self);
        // Entity::root().relayout(self);
        // Entity::root().redraw(self);

        Ok(())
    }

    pub fn spawn<F>(&self, target: F) where F: 'static + Send + Fn(&mut ContextProxy) {
        let mut cxp = ContextProxy {
            current: self.current,
            event_proxy: self.event_proxy.as_ref().map(|p| p.make_clone())
        };

        std::thread::spawn(move || target(&mut cxp));
    }
}

/// A bundle of data representing a snapshot of the context when a thread was spawned. It supports
/// a small subset of context operations. You will get one of these passed to you when you create a
/// new thread with `cx.spawn()`.
pub struct ContextProxy {
    pub current: Entity,
    pub event_proxy: Option<Box<dyn EventProxy>>,
}

pub enum ProxyEmitError {
    Unsupported,
    EventLoopClosed,
}

impl ContextProxy {
    pub fn emit<M: Message>(&mut self, message: M) -> Result<(), ProxyEmitError> {
        if let Some(proxy) = &self.event_proxy {
            let event = Event::new(message)
                .target(self.current)
                .origin(self.current)
                .propagate(Propagation::Up);

            proxy.send(event).map_err(|_| ProxyEmitError::EventLoopClosed)
        } else {
            Err(ProxyEmitError::Unsupported)
        }
    }

    pub fn emit_to<M: Message>(&mut self, target: Entity, message: M) -> Result<(), ProxyEmitError> {
        if let Some(proxy) = &self.event_proxy {
            let event = Event::new(message)
                .target(target)
                .origin(self.current)
                .propagate(Propagation::Direct);

            proxy.send(event).map_err(|_| ProxyEmitError::EventLoopClosed)
        } else {
            Err(ProxyEmitError::Unsupported)
        }
    }
}

pub trait EventProxy: Send {
    fn send(&self, event: Event) -> Result<(), ()>;
    fn make_clone(&self) -> Box<dyn EventProxy>;
}
