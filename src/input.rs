use bitflags::bitflags;
use windows::{
    Foundation::Size,
    Win32::{
        Foundation::HWND,
        UI::{
            Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
            WindowsAndMessaging::HCURSOR,
        },
    },
};
use windows_numerics::Vector2;

use crate::hittest::{HitTestTreeManager, HitTestTreeRef};

const CLICK_DETECTION_MAX_DISTNACE: f32 = 4.0;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct EventContinueControl: u8 {
        const STOP_PROPAGATION = 1 << 0;
        const CAPTURE_ELEMENT = 1 << 1;
        const RELEASE_CAPTURE_ELEMENT = 1 << 2;
        const RECOMPUTE_POINTER_ENTER = 1 << 3;
    }
}

pub enum PointerFocusState {
    None,
    Entering(HitTestTreeRef),
    Capturing(HitTestTreeRef),
}

pub struct PointerInputManager {
    last_client_pointer_pos: Option<Vector2>,
    pointer_focus: PointerFocusState,
    click_base_client_pointer_pos: Option<Vector2>,
}
impl PointerInputManager {
    pub fn new() -> Self {
        Self {
            last_client_pointer_pos: None,
            pointer_focus: PointerFocusState::None,
            click_base_client_pointer_pos: None,
        }
    }

    pub fn on_mouse_move<ActionContext>(
        &mut self,
        ht: &mut HitTestTreeManager<ActionContext>,
        action_context: &mut ActionContext,
        ht_root: HitTestTreeRef,
        client_size: Size,
        client_x: f32,
        client_y: f32,
    ) {
        self.last_client_pointer_pos = Some(Vector2 {
            X: client_x,
            Y: client_y,
        });

        if let Some(ref c) = self.click_base_client_pointer_pos {
            let d = (c.X - client_x).powi(2) + (c.Y - client_y).powi(2);

            if d >= CLICK_DETECTION_MAX_DISTNACE * CLICK_DETECTION_MAX_DISTNACE {
                // 動きすぎた場合はクリック判定状態を解く
                self.click_base_client_pointer_pos = None;
            }
        }

        if let PointerFocusState::Capturing(tr) = self.pointer_focus {
            let _ = ht
                .get(tr)
                .action_handler()
                .map_or(EventContinueControl::empty(), |a| {
                    a.on_pointer_move(
                        tr,
                        action_context,
                        ht,
                        client_x,
                        client_y,
                        client_size.Width,
                        client_size.Height,
                    )
                });

            return;
        }

        let new_hit = ht.perform_test(
            action_context,
            ht_root,
            client_x,
            client_y,
            0.0,
            0.0,
            client_size.Width,
            client_size.Height,
        );
        if let PointerFocusState::Entering(tr) = self.pointer_focus {
            if Some(tr) != new_hit {
                // entering changed: leave and enter
                let mut p = Some(tr);
                while let Some(tr) = p {
                    let t = ht.get(tr);
                    let next = t.parent;
                    let action_handler = t.action_handler();
                    let cont = action_handler.map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_leave(
                            tr,
                            action_context,
                            ht,
                            client_x,
                            client_y,
                            client_size.Width,
                            client_size.Height,
                        )
                    });
                    if cont.contains(EventContinueControl::STOP_PROPAGATION) {
                        break;
                    }

                    p = next;
                }

                if let Some(tr) = new_hit {
                    let mut p = Some(tr);
                    while let Some(tr) = p {
                        let t = ht.get(tr);
                        let next = t.parent;
                        let action_handler = t.action_handler();
                        let cont = action_handler.map_or(EventContinueControl::empty(), |a| {
                            a.on_pointer_enter(
                                tr,
                                action_context,
                                ht,
                                client_x,
                                client_y,
                                client_size.Width,
                                client_size.Height,
                            )
                        });
                        if cont.contains(EventContinueControl::STOP_PROPAGATION) {
                            break;
                        }

                        p = next;
                    }
                }
            }
        }

        self.pointer_focus = match new_hit {
            Some(tr) => PointerFocusState::Entering(tr),
            None => PointerFocusState::None,
        };

        let mut p = new_hit;
        while let Some(tr) = p {
            let t = ht.get(tr);
            let next = t.parent;
            let action_handler = t.action_handler();
            let flags = action_handler.map_or(EventContinueControl::empty(), |a| {
                a.on_pointer_move(
                    tr,
                    action_context,
                    ht,
                    client_x,
                    client_y,
                    client_size.Width,
                    client_size.Height,
                )
            });
            if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                self.on_mouse_move(ht, action_context, ht_root, client_size, client_x, client_y);
            }
            if flags.contains(EventContinueControl::STOP_PROPAGATION) {
                break;
            }

            p = next;
        }
    }

    pub fn on_mouse_left_down<ActionContext>(
        &mut self,
        hwnd: HWND,
        ht: &mut HitTestTreeManager<ActionContext>,
        action_context: &mut ActionContext,
        ht_root: HitTestTreeRef,
        client_size: Size,
        client_x: f32,
        client_y: f32,
    ) {
        self.click_base_client_pointer_pos = Some(Vector2 {
            X: client_x,
            Y: client_y,
        });

        match self.pointer_focus {
            PointerFocusState::Capturing(tr) => {
                let flags = ht
                    .get(tr)
                    .action_handler()
                    .map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_down(tr, action_context, ht, client_x, client_y)
                    });
                if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                    self.on_mouse_move(
                        ht,
                        action_context,
                        ht_root,
                        client_size,
                        client_x,
                        client_y,
                    );
                }
                if flags.contains(EventContinueControl::RELEASE_CAPTURE_ELEMENT) {
                    unsafe {
                        ReleaseCapture().expect("Failed to release captured pointer");
                    }
                    self.pointer_focus = PointerFocusState::Entering(tr);
                    self.on_mouse_move(
                        ht,
                        action_context,
                        ht_root,
                        client_size,
                        client_x,
                        client_y,
                    );
                }
            }
            PointerFocusState::Entering(tr) => {
                // bubbling
                let mut p = Some(tr);
                while let Some(tr) = p {
                    let t = ht.get(tr);
                    let next = t.parent;
                    let action_handler = t.action_handler();
                    let flags = action_handler.map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_down(tr, action_context, ht, client_x, client_y)
                    });
                    if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                        self.on_mouse_move(
                            ht,
                            action_context,
                            ht_root,
                            client_size,
                            client_x,
                            client_y,
                        );
                    }
                    if flags.contains(EventContinueControl::CAPTURE_ELEMENT) {
                        self.pointer_focus = PointerFocusState::Capturing(tr);
                        unsafe {
                            SetCapture(hwnd);
                        }
                    }
                    if flags.contains(EventContinueControl::STOP_PROPAGATION) {
                        break;
                    }

                    p = next;
                }
            }
            PointerFocusState::None => (),
        }
    }

    pub fn on_mouse_left_up<ActionContext>(
        &mut self,
        hwnd: HWND,
        ht: &mut HitTestTreeManager<ActionContext>,
        action_context: &mut ActionContext,
        ht_root: HitTestTreeRef,
        client_size: Size,
        client_x: f32,
        client_y: f32,
    ) {
        self.on_mouse_move(ht, action_context, ht_root, client_size, client_x, client_y);

        match self.pointer_focus {
            PointerFocusState::Capturing(tr) => {
                let flags = ht
                    .get(tr)
                    .action_handler()
                    .map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_up(tr, action_context, ht, client_x, client_y)
                    });
                if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                    self.on_mouse_move(
                        ht,
                        action_context,
                        ht_root,
                        client_size,
                        client_x,
                        client_y,
                    );
                }
                if flags.contains(EventContinueControl::RELEASE_CAPTURE_ELEMENT) {
                    unsafe {
                        ReleaseCapture().expect("Failed to release captured pointer");
                    }
                    self.pointer_focus = PointerFocusState::Entering(tr);
                    self.on_mouse_move(
                        ht,
                        action_context,
                        ht_root,
                        client_size,
                        client_x,
                        client_y,
                    );
                }
            }
            PointerFocusState::Entering(tr) => {
                // bubbling
                let mut p = Some(tr);
                while let Some(tr) = p {
                    let t = ht.get(tr);
                    let next = t.parent;
                    let action_handler = t.action_handler();
                    let flags = action_handler.map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_up(tr, action_context, ht, client_x, client_y)
                    });
                    if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                        self.on_mouse_move(
                            ht,
                            action_context,
                            ht_root,
                            client_size,
                            client_x,
                            client_y,
                        );
                    }
                    if flags.contains(EventContinueControl::CAPTURE_ELEMENT) {
                        self.pointer_focus = PointerFocusState::Capturing(tr);
                        unsafe {
                            SetCapture(hwnd);
                        }
                    }
                    if flags.contains(EventContinueControl::STOP_PROPAGATION) {
                        break;
                    }

                    p = next;
                }
            }
            PointerFocusState::None => (),
        }

        if self.click_base_client_pointer_pos.take().is_some() {
            match self.pointer_focus {
                PointerFocusState::Capturing(tr) => {
                    let flags =
                        ht.get(tr)
                            .action_handler()
                            .map_or(EventContinueControl::empty(), |a| {
                                a.on_click(
                                    tr,
                                    action_context,
                                    ht,
                                    client_x,
                                    client_y,
                                    client_size.Width,
                                    client_size.Height,
                                )
                            });
                    if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                        self.on_mouse_move(
                            ht,
                            action_context,
                            ht_root,
                            client_size,
                            client_x,
                            client_y,
                        );
                    }
                    if flags.contains(EventContinueControl::RELEASE_CAPTURE_ELEMENT) {
                        unsafe {
                            ReleaseCapture().expect("Failed to release captured pointer");
                        }
                        self.pointer_focus = PointerFocusState::Entering(tr);
                        self.on_mouse_move(
                            ht,
                            action_context,
                            ht_root,
                            client_size,
                            client_x,
                            client_y,
                        );
                    }
                }
                PointerFocusState::Entering(tr) => {
                    // bubbling
                    let mut p = Some(tr);
                    while let Some(tr) = p {
                        let t = ht.get(tr);
                        let next = t.parent;
                        let action_handler = t.action_handler();
                        let flags = action_handler.map_or(EventContinueControl::empty(), |a| {
                            a.on_click(
                                tr,
                                action_context,
                                ht,
                                client_x,
                                client_y,
                                client_size.Width,
                                client_size.Height,
                            )
                        });
                        if flags.contains(EventContinueControl::RECOMPUTE_POINTER_ENTER) {
                            self.on_mouse_move(
                                ht,
                                action_context,
                                ht_root,
                                client_size,
                                client_x,
                                client_y,
                            );
                        }
                        if flags.contains(EventContinueControl::CAPTURE_ELEMENT) {
                            self.pointer_focus = PointerFocusState::Capturing(tr);
                            unsafe {
                                SetCapture(hwnd);
                            }
                        }
                        if flags.contains(EventContinueControl::STOP_PROPAGATION) {
                            break;
                        }

                        p = next;
                    }
                }
                PointerFocusState::None => (),
            }
        }
    }

    pub fn cursor<ActionContext>(
        &self,
        ht: &HitTestTreeManager<ActionContext>,
        action_context: &mut ActionContext,
    ) -> Option<HCURSOR> {
        match self.pointer_focus {
            PointerFocusState::Capturing(tr) => ht
                .get(tr)
                .action_handler()
                .and_then(|a| a.cursor(tr, action_context)),
            PointerFocusState::Entering(tr) => {
                // bubbling
                let mut p = Some(tr);
                while let Some(tr) = p {
                    let t = ht.get(tr);
                    if let Some(c) = t
                        .action_handler()
                        .and_then(|a| a.cursor(tr, action_context))
                    {
                        return Some(c);
                    }

                    p = t.parent;
                }

                // not processed
                None
            }
            PointerFocusState::None => None,
        }
    }
}
