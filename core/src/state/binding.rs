use std::any::TypeId;
use std::collections::HashSet;

use morphorm::{LayoutType, PositionType};

use crate::{Color, Context, Display, Entity, Handle, StateStore, TreeExt, Units, View, Visibility};

use crate::{Data, Lens, Model};

pub struct Binding<L>
where
    L: Lens,
{
    lens: L,
    parent: Entity,
    count: usize,
    builder: Option<Box<dyn Fn(&mut Context, Field<L>)>>,
}

struct FallibleMarker(bool, usize);

impl Model for FallibleMarker {}

impl<L> Binding<L>
where
    L: 'static + Lens,
    <L as Lens>::Source: 'static + Model,
    <L as Lens>::Target: Data,
{
    pub fn new_fallible<F1, F2>(cx: &mut Context, lens: L, builder_some: F1, builder_none: F2)
    where
        F1: 'static + Fn(&mut Context, Field<L>),
        F2: 'static + Fn(&mut Context),
    {
        Self::new(cx, lens, move |cx, field| {
            let some = field.get_fallible(cx).is_some();
            let real_current = cx.current;
            let fake_current = cx.tree.get_child(cx.current, cx.count - 1).unwrap();

            cx.current = fake_current;
            let (stored_some, stored_count, force) = cx.data::<FallibleMarker>()
                .map(|dat| (dat.0, dat.1, false))
                .unwrap_or((false, 0, true));
            cx.current = real_current;

            if some != stored_some {
                for child in cx.current.child_iter(&cx.tree)
                    .enumerate()
                    .filter(|(idx, _)| *idx >= cx.count && *idx < cx.count + stored_count)
                    .map(|(_, e)| e)
                    .collect::<Vec<_>>()
                {
                    cx.remove(child);
                }
            }
            let start_count = cx.count;
            if some {
                builder_some(cx, field);
            } else {
                builder_none(cx);
            }
            if some != stored_some || force {
                cx.current = fake_current;
                FallibleMarker(some, cx.count - start_count).build(cx);
                cx.current = real_current;
            }
        })
    }

    pub fn new<F>(cx: &mut Context, lens: L, builder: F)
    where
        F: 'static + Fn(&mut Context, Field<L>),
        <L as Lens>::Source: Model,
    {
        let parent = cx.current;

        let binding = Self { lens, parent, count: cx.count + 1, builder: Some(Box::new(builder)) };

        let id = if let Some(id) = cx.tree.get_child(cx.current, cx.count) {
            id
        } else {
            let id = cx.entity_manager.create();
            cx.tree.add(id, cx.current).expect("Failed to add to tree");
            cx.cache.add(id).expect("Failed to add to cache");
            cx.style.add(id);
            id
        };

        let ancestors = parent.parent_iter(&cx.tree).collect::<HashSet<_>>();

        for entity in id.parent_iter(&cx.tree) {
            if let Some(model_data_store) = cx.data.get_mut(entity) {
                if let Some(model_data) = model_data_store.data.get(&TypeId::of::<L::Source>()) {
                    if let Some(lens_wrap) = model_data_store.lenses.get_mut(&TypeId::of::<L>()) {
                        let observers = lens_wrap.observers();

                        if ancestors.intersection(observers).next().is_none() {
                            lens_wrap.add_observer(id);
                        }
                    } else {
                        let mut observers = HashSet::new();
                        observers.insert(id);

                        let model = model_data.downcast_ref::<L::Source>().unwrap();

                        let old = lens.view(model);

                        model_data_store.lenses.insert(
                            TypeId::of::<L>(),
                            Box::new(StateStore { entity: id, lens, old: old.cloned(), observers }),
                        );
                    }

                    break;
                }
            }
        }

        cx.views.insert(id, Box::new(binding));

        cx.count += 1;

        // Call the body of the binding
        if let Some(mut view_handler) = cx.views.remove(&id) {
            view_handler.body(cx);
            cx.views.insert(id, view_handler);
        }

        let _: Handle<Self> = Handle { entity: id, p: Default::default(), cx }
            .width(Units::Stretch(1.0))
            .height(Units::Stretch(1.0))
            .background_color(Color::blue())
            .display(Display::None);
    }
}

impl<L: 'static + Lens> View for Binding<L> {
    fn body<'a>(&mut self, cx: &'a mut Context) {
        if let Some(builder) = self.builder.take() {
            //let prev = cx.current;
            //let count = cx.count;
            cx.current = self.parent;
            cx.count = self.count;
            (builder)(cx, Field { lens: self.lens.clone() });
            //cx.current = prev;
            //cx.count = count;
            self.builder = Some(builder);
        }
    }
}

#[derive(Clone, Copy)]
pub struct Field<L> {
    lens: L,
}

impl<L: Lens> Field<L>
where
    <L as Lens>::Source: 'static,
{
    pub fn get<'a>(&self, cx: &'a Context) -> &'a L::Target {
        self.get_fallible(cx).expect("Tried to get data from lens but it could not be retrieved. Do you want to use Binding::new_fallible or Field::get_fallible?")
    }

    pub fn get_fallible<'a>(&self, cx: &'a Context) -> Option<&'a L::Target> {
        self.lens.view(cx.data().expect(&format!(
            "Failed to get {:?} for entity: {:?}. Is the data in the tree?",
            self.lens, cx.current
        )))
    }
}

macro_rules! impl_res_simple {
    ($t:ty) => {
        impl Res<$t> for $t {
            fn get<'a>(&'a self, _: &'a Context) -> &'a $t {
                self
            }
        }
    };
}

pub trait Res<T> {
    fn get<'a>(&'a self, cx: &'a Context) -> &'a T;
}

impl_res_simple!(i8);
impl_res_simple!(i16);
impl_res_simple!(i32);
impl_res_simple!(i64);
impl_res_simple!(i128);
impl_res_simple!(isize);
impl_res_simple!(u8);
impl_res_simple!(u16);
impl_res_simple!(u32);
impl_res_simple!(u64);
impl_res_simple!(u128);
impl_res_simple!(usize);
impl_res_simple!(char);
impl_res_simple!(bool);
impl_res_simple!(f32);
impl_res_simple!(f64);

impl<T, L> Res<T> for Field<L>
where
    L: Lens<Target = T>,
{
    fn get<'a>(&'a self, cx: &'a Context) -> &'a T {
        self.get(cx)
    }
}

impl Res<Color> for Color {
    fn get<'a>(&'a self, _: &'a Context) -> &'a Color {
        self
    }
}

impl Res<Units> for Units {
    fn get<'a>(&'a self, _: &'a Context) -> &'a Units {
        self
    }
}

impl Res<Visibility> for Visibility {
    fn get<'a>(&'a self, _: &'a Context) -> &'a Visibility {
        self
    }
}

impl Res<Display> for Display {
    fn get<'a>(&'a self, _: &'a Context) -> &'a Display {
        self
    }
}

impl Res<LayoutType> for LayoutType {
    fn get<'a>(&'a self, _: &'a Context) -> &'a LayoutType {
        self
    }
}

impl Res<PositionType> for PositionType {
    fn get<'a>(&'a self, _: &'a Context) -> &'a PositionType {
        self
    }
}

impl<T> Res<(T, T)> for (T, T) {
    fn get<'a>(&'a self, _: &'a Context) -> &'a (T, T) {
        self
    }
}
