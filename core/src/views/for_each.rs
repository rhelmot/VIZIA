use crate::{Context, Handle, TreeExt, View};
pub struct ForEach {}

impl ForEach {
    pub fn new<'a, 'b, F>(cx: &'a mut Context<'b>, range: std::ops::Range<usize>, mut template: F) -> Handle<'a, 'b, Self>
    where
        F: 'static + FnMut(&'a mut Context<'b>, usize),
    {
        Self {}.build2(cx, move |cx| {
            if cx.current.child_iter(&cx.tree.clone()).count() != range.len() {
                for child in cx.current.child_iter(&cx.tree.clone()) {
                    cx.remove(child);
                }

                cx.style.needs_relayout = true;
                cx.style.needs_redraw = true;
            }

            let prev_count = cx.count;
            cx.count = 0;
            for i in range {
                (template)(cx, i);
            }
            cx.count = prev_count;
        })
    }
}

impl View<'_> for ForEach {}
