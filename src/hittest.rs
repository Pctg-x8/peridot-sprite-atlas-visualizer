use std::{collections::BTreeSet, rc::Rc};

use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::HCURSOR};

use crate::input::EventContinueControl;

pub struct PointerActionArgs {
    pub hwnd: HWND,
    pub client_x: f32,
    pub client_y: f32,
    pub client_width: f32,
    pub client_height: f32,
}

pub trait HitTestTreeActionHandler {
    type Context;

    #[allow(unused_variables)]
    fn hit_active(&self, sender: HitTestTreeRef, context: &Self::Context) -> bool {
        true
    }

    #[allow(unused_variables)]
    fn cursor(&self, sender: HitTestTreeRef, context: &mut Self::Context) -> Option<HCURSOR> {
        None
    }

    #[allow(unused_variables)]
    fn on_pointer_enter(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        args: PointerActionArgs,
    ) -> EventContinueControl {
        EventContinueControl::empty()
    }

    #[allow(unused_variables)]
    fn on_pointer_leave(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        args: PointerActionArgs,
    ) -> EventContinueControl {
        EventContinueControl::empty()
    }

    #[allow(unused_variables)]
    fn on_pointer_down(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        args: PointerActionArgs,
    ) -> EventContinueControl {
        EventContinueControl::empty()
    }

    #[allow(unused_variables)]
    fn on_pointer_up(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        args: PointerActionArgs,
    ) -> EventContinueControl {
        EventContinueControl::empty()
    }

    #[allow(unused_variables)]
    fn on_pointer_move(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        args: PointerActionArgs,
    ) -> EventContinueControl {
        EventContinueControl::empty()
    }

    #[allow(unused_variables)]
    fn on_click(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        args: PointerActionArgs,
    ) -> EventContinueControl {
        EventContinueControl::empty()
    }
}

pub struct HitTestTreeData<ActionContext> {
    pub left: f32,
    pub top: f32,
    pub left_adjustment_factor: f32,
    pub top_adjustment_factor: f32,
    pub width: f32,
    pub height: f32,
    pub width_adjustment_factor: f32,
    pub height_adjustment_factor: f32,
    pub parent: Option<HitTestTreeRef>,
    pub children: Vec<HitTestTreeRef>,
    pub action_handler:
        Option<std::rc::Weak<dyn HitTestTreeActionHandler<Context = ActionContext>>>,
}
impl<ActionContext> HitTestTreeData<ActionContext> {
    #[inline]
    pub fn action_handler(
        &self,
    ) -> Option<Rc<dyn HitTestTreeActionHandler<Context = ActionContext>>> {
        self.action_handler
            .as_ref()
            .and_then(std::rc::Weak::upgrade)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitTestTreeRef(usize);

pub struct HitTestTreeManager<ActionContext> {
    pub entities: Vec<HitTestTreeData<ActionContext>>,
    pub free: BTreeSet<usize>,
}
impl<ActionContext> HitTestTreeManager<ActionContext> {
    #[inline]
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free: BTreeSet::new(),
        }
    }

    pub fn alloc(&mut self, data: HitTestTreeData<ActionContext>) -> HitTestTreeRef {
        if let Some(f) = self.free.pop_first() {
            self.entities[f] = data;
            return HitTestTreeRef(f);
        }

        self.entities.push(data);
        HitTestTreeRef(self.entities.len() - 1)
    }

    #[inline]
    pub fn free(&mut self, index: HitTestTreeRef) {
        self.free.insert(index.0);
    }

    pub fn free_rec(&mut self, index: HitTestTreeRef) {
        let mut stack = vec![index];
        while !stack.is_empty() {
            for x in core::mem::replace(&mut stack, Vec::new()) {
                stack.extend(self.entities[x.0].children.iter().copied());

                self.free(x);
            }
        }
    }

    #[inline]
    pub fn get(&self, index: HitTestTreeRef) -> &HitTestTreeData<ActionContext> {
        &self.entities[index.0]
    }

    #[inline]
    pub fn get_mut(&mut self, index: HitTestTreeRef) -> &mut HitTestTreeData<ActionContext> {
        &mut self.entities[index.0]
    }

    pub fn add_child(&mut self, parent: HitTestTreeRef, child: HitTestTreeRef) {
        self.entities[parent.0].children.push(child);
        self.entities[child.0].parent = Some(parent);
    }

    pub fn remove_child(&mut self, child: HitTestTreeRef) {
        let Some(p) = self.entities[child.0].parent.take() else {
            return;
        };

        self.entities[p.0].children.retain(|&x| x != child);
    }

    pub fn dump(&self, root: HitTestTreeRef) {
        fn rec<ActionContext>(
            this: &HitTestTreeManager<ActionContext>,
            x: HitTestTreeRef,
            indent: usize,
        ) {
            for _ in 0..indent {
                print!("  ");
            }

            let e = &this.entities[x.0];
            println!(
                "#{}: (x{}+{}, x{}+{}) size (x{}+{}, x{}+{})",
                x.0,
                e.left_adjustment_factor,
                e.left,
                e.top_adjustment_factor,
                e.top,
                e.width_adjustment_factor,
                e.width,
                e.height_adjustment_factor,
                e.height
            );

            for &c in e.children.iter() {
                rec(this, c, indent + 1);
            }
        }

        rec(self, root, 0);
    }

    pub fn translate_client_to_tree_local(
        &self,
        x: HitTestTreeRef,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> (f32, f32, f32, f32) {
        let e = &self.entities[x.0];
        match e.parent {
            None => {
                // parent = clientなので直接計算する
                let actual_left = e.left + client_width * e.left_adjustment_factor;
                let actual_top = e.top + client_height * e.top_adjustment_factor;

                (
                    client_x - actual_left,
                    client_y - actual_top,
                    e.width + client_width * e.width_adjustment_factor,
                    e.height + client_height * e.height_adjustment_factor,
                )
            }
            Some(p) => {
                // 親で一回計算して、その中のローカル座標として計算する
                let (parent_local_x, parent_local_y, parent_actual_width, parent_actual_height) =
                    self.translate_client_to_tree_local(
                        p,
                        client_x,
                        client_y,
                        client_width,
                        client_height,
                    );
                let actual_left = e.left + parent_actual_width * e.left_adjustment_factor;
                let actual_top = e.top + parent_actual_height * e.top_adjustment_factor;

                (
                    parent_local_x - actual_left,
                    parent_local_y - actual_top,
                    e.width + parent_actual_width * e.width_adjustment_factor,
                    e.height + parent_actual_height * e.height_adjustment_factor,
                )
            }
        }
    }

    pub fn perform_test(
        &self,
        context: &ActionContext,
        x: HitTestTreeRef,
        global_x: f32,
        global_y: f32,
        parent_global_left: f32,
        parent_global_top: f32,
        parent_global_width: f32,
        parent_global_height: f32,
    ) -> Option<HitTestTreeRef> {
        let e = &self.entities[x.0];
        if !e
            .action_handler
            .as_ref()
            .and_then(|e| e.upgrade())
            .map_or(true, |e| e.hit_active(x, context))
        {
            // ヒットしない
            return None;
        }

        let (global_left, global_top, global_width, global_height) = (
            parent_global_left + e.left_adjustment_factor * parent_global_width + e.left,
            parent_global_top + parent_global_height * e.top_adjustment_factor + e.top,
            parent_global_width * e.width_adjustment_factor + e.width,
            parent_global_height * e.height_adjustment_factor + e.height,
        );
        let (global_right, global_bottom) =
            (global_left + global_width, global_top + global_height);

        // 後ろにある方が上なのでそれを優先して見る
        if let Some(h) = e.children.iter().rev().find_map(|&x| {
            self.perform_test(
                context,
                x,
                global_x,
                global_y,
                global_left,
                global_top,
                global_width,
                global_height,
            )
        }) {
            // found in child
            return Some(h);
        }

        if global_left <= global_x
            && global_x <= global_right
            && global_top <= global_y
            && global_y <= global_bottom
        {
            // in range
            Some(x)
        } else {
            None
        }
    }
}
