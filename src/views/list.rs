

use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

use keyboard_types::Code;

use crate::{Color, Context, Event, Handle, Lens, Model, MouseButton, Data, StateStore, Store, TreeExt, View, WindowEvent};
use crate::Units::*;
#[derive(Debug)]
pub struct ItemPtr<L, T>
where
    L: Lens<Target = Vec<T>>,
{
    lens: L,
    index: usize,
    row: usize,
    col: usize,
}

impl<L,T> Copy for ItemPtr<L,T> 
where
    L: Lens<Target = Vec<T>>, {}

impl<L: Lens<Target = Vec<T>>,T> Clone for ItemPtr<L,T> {
    fn clone(&self) -> Self {
        Self {
            lens: self.lens.clone(),
            index: self.index,
            row: self.row,
            col: self.col,
        }
    }
}

impl<L, T> ItemPtr<L, T>
where
    L: Lens<Target = Vec<T>>,
{
    pub fn new(lens: L, index: usize, row: usize, col: usize) -> Self {
        Self { lens, index, row, col }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn value<'a>(&self, cx: &'a Context) -> &'a T
    where
        <L as Lens>::Source: 'static,
    {
        self.lens.view(cx.data().expect("Failed to get data")).get(self.index).expect(&format!("Failed to get item: {}", self.index))
    }
}


pub trait DataHandle: Clone + Copy {
    type Data;
    fn get<'a>(&self, cx: &'a Context) -> &'a Self::Data;
}

impl<L, T> DataHandle for ItemPtr<L, T> 
where
    L: Lens<Target = Vec<T>>,
{
    type Data = T;
    fn get<'a>(&self, cx: &'a Context) -> &'a Self::Data {
        self.value(cx)
    }
}



#[derive(Lens, Default)]
pub struct ListData {
    pub selected: usize,
    pub length: usize,
}

impl ListData {
    pub fn new(selected: usize) -> Self {
        Self {
            selected,
            length: 0,
        }
    }
}

#[derive(Debug)]
pub enum ListEvent {
    IncrementSelection,
    DecrementSelection,
    SetSelected(usize),
    SetLength(usize),
}

impl Model for ListData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(list_event) = event.message.downcast() {
            match list_event {
                ListEvent::IncrementSelection => {
                    self.selected += 1;
                    self.selected = self.selected.clamp(0, self.length-1);
                }

                ListEvent::DecrementSelection => {
                    if self.selected <= 1 {
                        self.selected = 0;
                    } else {
                        self.selected -= 1;
                    }
                }

                ListEvent::SetSelected(index) => {
                    if *index <= 0 {
                        self.selected = 0;
                    } else if *index > self.length - 1 {
                        self.selected = self.length - 1;
                    } else {
                        self.selected = *index;
                    }
                }

                ListEvent::SetLength(length) => {
                    self.length = *length;
                }
            }
        }
    }
}

pub struct List<L, T: 'static>
where
    L: Lens<Target = Vec<T>>,
    T: Data,
{
    lens: L,
    builder: Option<Box<dyn Fn(&mut Context, ItemPtr<L, T>)>>,
    list_data: bool,
}

impl<L: 'static + Lens<Target = Vec<T>>, T: Data> List<L, T> {
    pub fn new<F>(cx: &mut Context, lens: L, item: F) -> Handle<Self>
    where
        F: 'static + Fn(&mut Context, ItemPtr<L, T>),
        <L as Lens>::Source: Model,
    {

        let parent = cx.current;
        let list = Self {
            lens,
            builder: Some(Box::new(item)),
            list_data: true,
        };

        let id = if let Some(id) = cx.tree.get_child(cx.current, cx.count) {
            id
        } else {
            let id = cx.entity_manager.create();
            cx.tree.add(id, cx.current).expect("Failed to add to tree");
            cx.cache.add(id).expect("Failed to add to cache");
            cx.style.borrow_mut().add(id);
            cx.views.insert(id, Box::new(list));
            id  
        };

        cx.count += 1;

        // let handle = Self {
        //     lens,
        //     builder: Some(Box::new(item)),
        // }
        // .build(cx);
        //.height(Auto)
        //.width(Auto)
        //.background_color(Color::rgb(50,70,90));

        // let mut ancestors = HashSet::new();
        // for entity in parent.parent_iter(&cx.tree) {
        //     ancestors.insert(entity);

        //     if let Some(model_list) = cx.data.model_data.get_mut(entity) {
        //         for (_, model) in model_list.iter_mut() {
        //             if let Some(store) = model.downcast::<Store<L::Source>>() {
        //                 if store.observers.intersection(&ancestors).next().is_some() {
        //                     break;
        //                 }
        //                 store.insert_observer(id);
        //             }
        //         }
        //     }
        // }

        if let Some(lens_wrap) = cx.lenses.get_mut(&TypeId::of::<L>()) {
            lens_wrap.add_observer(id);
        } else {
            let mut observers = HashSet::new();
            observers.insert(id);
            let old = lens.view(cx.data().unwrap());
            cx.lenses.insert(TypeId::of::<L>(), Box::new(StateStore {
                lens,
                old: old.clone(),
                observers,
            }));
        }

        if let Some(mut view_handler) = cx.views.remove(&id) {
            let prev = cx.current;
            cx.current = id;
            let prev_count = cx.count;
            cx.count = 0;
            view_handler.body(cx);
            cx.current = prev;
            cx.count = prev_count;
            cx.views.insert(id, view_handler);
        }

        let handle = Handle {
            entity: id,
            style: cx.style.clone(),
            p: PhantomData::default(),
        };

        cx.focused = handle.entity;

        handle
    }
}

impl<L: 'static + Lens<Target = Vec<T>>, T: Data> View for List<L, T> {
    fn body(&mut self, cx: &mut Context) {





        let builder = self.builder.take().unwrap();

        let mut found_store = None;

        'tree: for entity in cx.current.parent_iter(&cx.tree.clone()) {
            if let Some(model_list) = cx.data.model_data.get(entity) {
                for (_, model) in model_list.iter() {
                    if let Some(store) = model.downcast_ref::<Store<L::Source>>() {
                        found_store = Some(store); 
                        break 'tree;
                    }
                }
            }
        };

        if let Some(store) = found_store {
            
            let len = self.lens.view(&store.data).len();

            if cx.current.child_iter(&cx.tree.clone()).count() != len {
                println!("Remove Children: {} {}", cx.current.child_iter(&cx.tree.clone()).count(), len);
                for child in cx.current.child_iter(&cx.tree.clone()) {
                    cx.remove(child);
                }                
            }
            
            if self.list_data {
                ListData {
                    selected: 3,
                    length: len,
                }.build(cx);
            }

            for index in 0..len {
                let ptr = ItemPtr::new(self.lens.clone(), index, index, 0);
                (builder)(cx, ptr);
                //cx.count += 1;
            }
        }

        // let store = cx
        //     .data
        //     .model_data
        //     .get(&TypeId::of::<L::Source>())
        //     .and_then(|model| model.downcast_ref::<Store<L::Source>>());

        // if let Some(store) = store {
        //     let len = self.lens.view(&store.data).len();
        //     for index in 0..len {
        //         let ptr = ItemPtr::new(self.lens.clone(), index);
        //         (builder)(cx, ptr);
        //     }
        // }
        self.builder = Some(builder);
    }

    fn event(&mut self, cx: &mut Context, event: &mut crate::Event) {
        if let Some(window_event) = event.message.downcast() {
            match window_event {
                // WindowEvent::MouseDown(button) => {
                //     if *button == MouseButton::Left {
                //         cx.emit(ListEvent::IncrementSelection);
                //     }

                //     if *button == MouseButton::Right {
                //         cx.emit(ListEvent::DecrementSelection);
                //     }
                // }

                WindowEvent::KeyDown(code, _) => {
                    match code {
                        Code::ArrowDown => {
                            cx.emit(ListEvent::IncrementSelection);
                        }

                        Code::ArrowUp => {
                            cx.emit(ListEvent::DecrementSelection);
                        }

                        _=> {}
                    }
                }

                _=> {}
            }
        }
    }
}

// impl<L: Lens<Target = Vec<T>>,T: Data> Handle<List<L,T>> {
//     pub fn with_list_data(self, cx: &mut Context, flag: bool) -> Self {
//         if let Some(list) = cx.views.get_mut(&self.entity).and_then(|f| f.downcast_mut::<List<L,T>>()) {
//             list.list_data = flag;
//         }

//         self
//     }
// }