use bitflags::bitflags;
use windows::{
    Foundation::{Numerics::Vector2, Size},
    Win32::{
        Foundation::HWND,
        UI::{
            Input::KeyboardAndMouse::{ReleaseCapture, SetCapture},
            WindowsAndMessaging::HCURSOR,
        },
    },
};

use crate::hittest::{HitTestTreeContext, HitTestTreeRef};

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct EventContinueControl: u8 {
        const STOP_PROPAGATION = 1 << 0;
        const CAPTURE_ELEMENT = 1 << 1;
        const RELEASE_CAPTURE_ELEMENT = 1 << 2;
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
}
impl PointerInputManager {
    pub fn new() -> Self {
        Self {
            last_client_pointer_pos: None,
            pointer_focus: PointerFocusState::None,
        }
    }

    pub fn on_mouse_move(
        &mut self,
        ht: &mut HitTestTreeContext,
        ht_root: HitTestTreeRef,
        client_size: Size,
        client_x: f32,
        client_y: f32,
    ) {
        self.last_client_pointer_pos = Some(Vector2 {
            X: client_x,
            Y: client_y,
        });

        if let PointerFocusState::Capturing(tr) = self.pointer_focus {
            let _ = ht
                .get(tr)
                .action_handler()
                .map_or(EventContinueControl::empty(), |a| {
                    a.on_pointer_move(tr, ht, client_x, client_y)
                });

            return;
        }

        let new_hit = ht.perform_test(
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
                    let cont = t
                        .action_handler()
                        .map_or(EventContinueControl::empty(), |a| a.on_pointer_leave(tr));
                    if cont.contains(EventContinueControl::STOP_PROPAGATION) {
                        break;
                    }

                    p = t.parent;
                }

                if let Some(tr) = new_hit {
                    let mut p = Some(tr);
                    while let Some(tr) = p {
                        let t = ht.get(tr);
                        let cont = t
                            .action_handler()
                            .map_or(EventContinueControl::empty(), |a| a.on_pointer_enter(tr));
                        if cont.contains(EventContinueControl::STOP_PROPAGATION) {
                            break;
                        }

                        p = t.parent;
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
                a.on_pointer_move(tr, ht, client_x, client_y)
            });
            if flags.contains(EventContinueControl::STOP_PROPAGATION) {
                break;
            }

            p = next;
        }
    }

    pub fn on_mouse_left_down(
        &mut self,
        hwnd: HWND,
        ht: &mut HitTestTreeContext,
        ht_root: HitTestTreeRef,
        client_size: Size,
        client_x: f32,
        client_y: f32,
    ) {
        match self.pointer_focus {
            PointerFocusState::Capturing(tr) => {
                let flags = ht
                    .get(tr)
                    .action_handler()
                    .map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_down(tr, ht, client_x, client_y)
                    });
                if flags.contains(EventContinueControl::RELEASE_CAPTURE_ELEMENT) {
                    unsafe {
                        ReleaseCapture().expect("Failed to release captured pointer");
                    }
                    self.pointer_focus = PointerFocusState::Entering(tr);
                    self.on_mouse_move(ht, ht_root, client_size, client_x, client_y);
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
                        a.on_pointer_down(tr, ht, client_x, client_y)
                    });
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

    pub fn on_mouse_left_up(
        &mut self,
        hwnd: HWND,
        ht: &mut HitTestTreeContext,
        ht_root: HitTestTreeRef,
        client_size: Size,
        client_x: f32,
        client_y: f32,
    ) {
        self.on_mouse_move(ht, ht_root, client_size, client_x, client_y);

        match self.pointer_focus {
            PointerFocusState::Capturing(tr) => {
                let flags = ht
                    .get(tr)
                    .action_handler()
                    .map_or(EventContinueControl::empty(), |a| {
                        a.on_pointer_up(tr, ht, client_x, client_y)
                    });
                if flags.contains(EventContinueControl::RELEASE_CAPTURE_ELEMENT) {
                    unsafe {
                        ReleaseCapture().expect("Failed to release captured pointer");
                    }
                    self.pointer_focus = PointerFocusState::Entering(tr);
                    self.on_mouse_move(ht, ht_root, client_size, client_x, client_y);
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
                        a.on_pointer_down(tr, ht, client_x, client_y)
                    });
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

    pub fn cursor(&self, ht: &HitTestTreeContext) -> Option<HCURSOR> {
        match self.pointer_focus {
            PointerFocusState::Capturing(tr) => {
                ht.get(tr).action_handler().and_then(|a| a.cursor(tr))
            }
            PointerFocusState::Entering(tr) => {
                // bubbling
                let mut p = Some(tr);
                while let Some(tr) = p {
                    let t = ht.get(tr);
                    if let Some(c) = t.action_handler().and_then(|a| a.cursor(tr)) {
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
