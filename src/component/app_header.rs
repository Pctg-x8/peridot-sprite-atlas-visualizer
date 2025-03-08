use core::mem::MaybeUninit;
use std::{cell::Cell, rc::Rc};
use windows::{
    Foundation::Size,
    UI::Composition::{
        CompositionEffectSourceParameter, CompositionMappingMode, ContainerVisual, SpriteVisual,
        VisualCollection,
    },
    Win32::{
        Graphics::Direct2D::{
            Common::{D2D_POINT_2F, D2D1_COLOR_F, D2D1_GRADIENT_STOP},
            D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_GAMMA_2_2,
            D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, ID2D1DeviceContext, ID2D1RenderTarget,
            ID2D1SolidColorBrush,
        },
        UI::WindowsAndMessaging::{HTCAPTION, HTCLIENT, HTCLOSE, HTMINBUTTON},
    },
};
use windows_core::{Interface, h};
use windows_numerics::{Matrix3x2, Vector2, Vector3};

use crate::{
    AppHitTestTreeManager, D2D1_COLOR_F_WHITE, PointDIP, PresenterInitContext, RectDIP,
    ViewInitContext,
    app_state::AppState,
    color_factory::{
        d2d1_color_f_from_websafe_hex_argb, d2d1_color_f_from_websafe_hex_rgb,
        ui_color_from_websafe_hex_rgb, ui_color_from_websafe_hex_rgb_with_alpha,
    },
    composition_element_builder::{
        CompositionColorGradientStopParams, CompositionLinearGradientBrushParams,
        CompositionMaskBrushParams, CompositionSurfaceBrushParams, ContainerVisualParams,
        SpriteVisualParams,
    },
    coordinate::{dip_to_pixels, size_sq},
    create_instant_effect_brush,
    effect_builder::{ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams},
    extra_bindings::Microsoft::Graphics::Canvas::CanvasComposite,
    hittest::{HitTestTreeActionHandler, HitTestTreeData, HitTestTreeManager, HitTestTreeRef},
    input::EventContinueControl,
    surface_helper::draw_2d,
};

#[inline(always)]
fn new_icon_brush(dc: &ID2D1DeviceContext) -> windows_core::Result<ID2D1SolidColorBrush> {
    unsafe { dc.CreateSolidColorBrush(&d2d1_color_f_from_websafe_hex_rgb(0x111), None) }
}

struct AppCloseButtonView {
    root: ContainerVisual,
}
impl AppCloseButtonView {
    const BUTTON_SIZE: f32 = 24.0;
    const ICON_SIZE: f32 = 6.0;
    const SURFACE_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 128);

    fn new(init: &mut ViewInitContext) -> Self {
        let icon_size_px = init.dip_to_pixels(Self::ICON_SIZE);
        let button_size_px = init.dip_to_pixels(Self::BUTTON_SIZE);

        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(icon_size_px))
            .unwrap();
        draw_2d(&icon_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let brush = new_icon_brush(&dc)?;

            unsafe {
                dc.Clear(None);
                dc.DrawLine(
                    D2D_POINT_2F { x: 0.0, y: 0.0 },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: Self::ICON_SIZE,
                    },
                    &brush,
                    1.5,
                    None,
                );
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: 0.0,
                    },
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_SIZE,
                    },
                    &brush,
                    1.5,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let circle_mask_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(button_size_px))
            .unwrap();
        draw_2d(&circle_mask_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let gradient_brush = unsafe {
                dc.CreateRadialGradientBrush(
                    &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                        center: D2D_POINT_2F {
                            x: Self::BUTTON_SIZE * 0.5,
                            y: Self::BUTTON_SIZE * 0.5,
                        },
                        radiusX: Self::BUTTON_SIZE * 0.5,
                        radiusY: Self::BUTTON_SIZE * 0.5,
                        gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 },
                    },
                    None,
                    &dc.cast::<ID2D1RenderTarget>()
                        .unwrap()
                        .CreateGradientStopCollection(
                            &[
                                D2D1_GRADIENT_STOP {
                                    position: 0.0,
                                    color: d2d1_color_f_from_websafe_hex_argb(0xffff),
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 0.75,
                                    color: d2d1_color_f_from_websafe_hex_argb(0xffff),
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 1.0,
                                    color: d2d1_color_f_from_websafe_hex_argb(0x0fff),
                                },
                            ],
                            D2D1_GAMMA_2_2,
                            D2D1_EXTEND_MODE_CLAMP,
                        )?,
                )?
            };

            unsafe {
                dc.Clear(None);
                dc.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: Self::BUTTON_SIZE * 0.5,
                            y: Self::BUTTON_SIZE * 0.5,
                        },
                        radiusX: Self::BUTTON_SIZE * 0.5,
                        radiusY: Self::BUTTON_SIZE * 0.5,
                    },
                    &gradient_brush,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let root = ContainerVisualParams::new()
            .size_sq(button_size_px)
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let bg_brush = create_instant_effect_brush(
            init.subsystem,
            &CompositeEffectParams::new(&[
                GaussianBlurEffectParams::new(
                    &CompositionEffectSourceParameter::Create(h!("backdrop")).unwrap(),
                )
                .blur_amount_px(9.0)
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
                ColorSourceEffectParams {
                    color: Some(Self::SURFACE_COLOR),
                }
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
            ])
            .mode(CanvasComposite::SourceOver)
            .instantiate()
            .unwrap(),
            &[(
                h!("backdrop"),
                init.subsystem
                    .compositor
                    .CreateBackdropBrush()
                    .unwrap()
                    .cast()
                    .unwrap(),
            )],
        )
        .unwrap();

        let bg = SpriteVisualParams::new(
            &CompositionMaskBrushParams {
                source: &bg_brush,
                mask: &CompositionSurfaceBrushParams::new(&circle_mask_surface)
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .expand()
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let icon = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&icon_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size_sq(icon_size_px)
        .anchor_point(Vector2 { X: 0.5, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.5, Y: 0.5 })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&icon).unwrap();

        Self { root }
    }

    fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    fn locate(&self, offset: Vector3, relative_offset_adjustment: Vector3) {
        self.root.SetOffset(offset).unwrap();
        self.root
            .SetRelativeOffsetAdjustment(relative_offset_adjustment)
            .unwrap();
    }
}

struct AppMinimizeButtonView {
    root: ContainerVisual,
}
impl AppMinimizeButtonView {
    const BUTTON_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 6.0;
    const SURFACE_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 128);

    fn new(init: &mut ViewInitContext) -> Self {
        let icon_size_px = init.dip_to_pixels(Self::ICON_SIZE);
        let button_size_px = init.dip_to_pixels(Self::BUTTON_SIZE);

        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(icon_size_px))
            .unwrap();
        draw_2d(&icon_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let brush = new_icon_brush(&dc)?;

            unsafe {
                dc.Clear(None);
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_SIZE - 0.5,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: Self::ICON_SIZE - 0.5,
                    },
                    &brush,
                    1.5,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let circle_mask_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(button_size_px))
            .unwrap();
        draw_2d(&circle_mask_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            // Create radial gradient brush
            let gradient_brush = unsafe {
                dc.CreateRadialGradientBrush(
                    &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                        center: D2D_POINT_2F {
                            x: Self::BUTTON_SIZE * 0.5,
                            y: Self::BUTTON_SIZE * 0.5,
                        },
                        radiusX: Self::BUTTON_SIZE * 0.5,
                        radiusY: Self::BUTTON_SIZE * 0.5,
                        gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 },
                    },
                    None,
                    &dc.cast::<ID2D1RenderTarget>()
                        .unwrap()
                        .CreateGradientStopCollection(
                            &[
                                D2D1_GRADIENT_STOP {
                                    position: 0.0,
                                    color: d2d1_color_f_from_websafe_hex_argb(0xffff),
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 0.75,
                                    color: d2d1_color_f_from_websafe_hex_argb(0xffff),
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 1.0,
                                    color: d2d1_color_f_from_websafe_hex_argb(0x0fff),
                                },
                            ],
                            D2D1_GAMMA_2_2,
                            D2D1_EXTEND_MODE_CLAMP,
                        )?,
                )?
            };

            unsafe {
                dc.Clear(None);
                dc.FillEllipse(
                    &D2D1_ELLIPSE {
                        point: D2D_POINT_2F {
                            x: Self::BUTTON_SIZE * 0.5,
                            y: Self::BUTTON_SIZE * 0.5,
                        },
                        radiusX: Self::BUTTON_SIZE * 0.5,
                        radiusY: Self::BUTTON_SIZE * 0.5,
                    },
                    &gradient_brush,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let root = ContainerVisualParams::new()
            .size_sq(init.dip_to_pixels(Self::BUTTON_SIZE))
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let bg_brush = create_instant_effect_brush(
            init.subsystem,
            &CompositeEffectParams::new(&[
                GaussianBlurEffectParams::new(
                    &CompositionEffectSourceParameter::Create(h!("backdrop")).unwrap(),
                )
                .blur_amount_px(9.0)
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
                ColorSourceEffectParams {
                    color: Some(Self::SURFACE_COLOR),
                }
                .instantiate()
                .unwrap()
                .cast()
                .unwrap(),
            ])
            .mode(CanvasComposite::SourceOver)
            .instantiate()
            .unwrap(),
            &[(
                h!("backdrop"),
                init.subsystem
                    .compositor
                    .CreateBackdropBrush()
                    .unwrap()
                    .cast()
                    .unwrap(),
            )],
        )
        .unwrap();

        let bg = SpriteVisualParams::new(
            &CompositionMaskBrushParams {
                source: &bg_brush,
                mask: &CompositionSurfaceBrushParams::new(&circle_mask_surface)
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .expand()
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let icon = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&icon_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size_sq(icon_size_px)
        .anchor_point(Vector2 { X: 0.5, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.5, Y: 0.5 })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&icon).unwrap();

        Self { root }
    }

    fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    fn locate(&self, offset: Vector3, relative_offset_adjustment: Vector3) {
        self.root.SetOffset(offset).unwrap();
        self.root
            .SetRelativeOffsetAdjustment(relative_offset_adjustment)
            .unwrap();
    }
}

struct AppMenuButtonView {
    root: ContainerVisual,
    bg: SpriteVisual,
    ht_root: HitTestTreeRef,
    dpi: Cell<f32>,
    size: Cell<f32>,
}
impl AppMenuButtonView {
    const ICON_SIZE: f32 = 12.0;
    const ICON_COLOR: D2D1_COLOR_F = d2d1_color_f_from_websafe_hex_rgb(0xfff);
    const ICON_LINE_THICKNESS: f32 = 2.0;
    const BG_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb(0xcff);
    const BG_COLOR_OUT: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xcff, 0);

    fn new(init: &mut ViewInitContext) -> Self {
        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(size_sq(init.dip_to_pixels(Self::ICON_SIZE)))
            .unwrap();
        draw_2d(&icon_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
                dc.SetTransform(&Matrix3x2::translation(
                    init.signed_pixels_to_dip(offset.x),
                    init.signed_pixels_to_dip(offset.y),
                ));
            }

            let brush = unsafe { dc.CreateSolidColorBrush(&Self::ICON_COLOR, None)? };

            unsafe {
                dc.Clear(None);
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_LINE_THICKNESS * 0.5,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: Self::ICON_LINE_THICKNESS * 0.5,
                    },
                    &brush,
                    Self::ICON_LINE_THICKNESS,
                    None,
                );
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_SIZE * 0.5,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: Self::ICON_SIZE * 0.5,
                    },
                    &brush,
                    Self::ICON_LINE_THICKNESS,
                    None,
                );
                dc.DrawLine(
                    D2D_POINT_2F {
                        x: 0.0,
                        y: Self::ICON_SIZE - Self::ICON_LINE_THICKNESS * 0.5,
                    },
                    D2D_POINT_2F {
                        x: Self::ICON_SIZE,
                        y: Self::ICON_SIZE - Self::ICON_LINE_THICKNESS * 0.5,
                    },
                    &brush,
                    Self::ICON_LINE_THICKNESS,
                    None,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let bg_gradient = init
            .subsystem
            .compositor
            .CreateRadialGradientBrush()
            .unwrap();
        bg_gradient
            .ColorStops()
            .unwrap()
            .Append(
                &init
                    .subsystem
                    .compositor
                    .CreateColorGradientStopWithOffsetAndColor(0.0, Self::BG_COLOR)
                    .unwrap(),
            )
            .unwrap();
        bg_gradient
            .ColorStops()
            .unwrap()
            .Append(
                &init
                    .subsystem
                    .compositor
                    .CreateColorGradientStopWithOffsetAndColor(1.0, Self::BG_COLOR_OUT)
                    .unwrap(),
            )
            .unwrap();
        bg_gradient
            .SetMappingMode(CompositionMappingMode::Relative)
            .unwrap();
        bg_gradient
            .SetEllipseCenter(Vector2 { X: 0.0, Y: 0.0 })
            .unwrap();
        bg_gradient
            .SetEllipseRadius(Vector2 { X: 1.0, Y: 1.0 })
            .unwrap();

        let root = ContainerVisualParams::new()
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let icon = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&icon_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .anchor_point(Vector2 { X: 0.5, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.5, Y: 0.5 })
        .size_sq(init.dip_to_pixels(Self::ICON_SIZE))
        .instantiate(&init.subsystem.compositor)
        .unwrap();
        let bg = SpriteVisualParams::new(&bg_gradient)
            .expand()
            .opacity(0.0)
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&icon).unwrap();

        let ht_root = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: 0.0,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 0.0,
            height: 0.0,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 0.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            bg,
            ht_root,
            dpi: Cell::new(init.dpi),
            size: Cell::new(0.0),
        }
    }

    fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    fn set_size(&self, size: f32, ht: &mut AppHitTestTreeManager) {
        let dpi = self.dpi.get();

        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(size, dpi),
                Y: dip_to_pixels(size, dpi),
            })
            .unwrap();
        ht.get_mut(self.ht_root).width = size;
        ht.get_mut(self.ht_root).height = size;

        self.size.set(size);
    }

    fn set_opacity_by_local_pos(&self, lx: f32, ly: f32) {
        let size = self.size.get();
        let opacity = 1.0 - ((lx.powi(2) + ly.powi(2)) / size.powi(2)).min(1.0);

        self.bg.SetOpacity(1.0 - (1.0 - opacity).powi(2)).unwrap();
    }
}

struct AppHeaderBaseView {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    height: f32,
}
impl AppHeaderBaseView {
    fn new(init: &mut ViewInitContext, init_label: &str) -> Self {
        let tl = init
            .subsystem
            .new_text_layout_unrestricted(init_label, &init.subsystem.default_ui_format)
            .unwrap();
        let mut tm = MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(tm.as_mut_ptr()).unwrap();
        }
        let tm = unsafe { tm.assume_init() };
        let label_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(tm.width),
                Height: init.dip_to_pixels(tm.height),
            })
            .unwrap();
        draw_2d(&label_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);
            }

            let brush = unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None)? };

            unsafe {
                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x),
                        y: init.signed_pixels_to_dip(offset.y),
                    },
                    &tl,
                    &brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();

        let height = 32.0 + tm.height;
        let root = ContainerVisualParams::new()
            .height(init.dip_to_pixels(height))
            .expand_width()
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let bg = SpriteVisualParams::new(
            &CompositionLinearGradientBrushParams {
                start_point: Vector2 { X: 0.0, Y: 0.0 },
                end_point: Vector2 { X: 0.0, Y: 1.0 },
                stops: &[
                    CompositionColorGradientStopParams {
                        offset: 0.0,
                        color: ui_color_from_websafe_hex_rgb_with_alpha(0x000, 128),
                    },
                    CompositionColorGradientStopParams {
                        offset: 1.0,
                        color: ui_color_from_websafe_hex_rgb_with_alpha(0x000, 32),
                    },
                ],
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
        )
        .expand()
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let label = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&label_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(tm.width),
            Y: init.dip_to_pixels(tm.height),
        })
        .offset_xy(Vector2 {
            X: init.dip_to_pixels(height + 8.0),
            Y: init.dip_to_pixels(16.0),
        })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&label).unwrap();

        let ht_root = init.ht.borrow_mut().alloc(HitTestTreeData {
            left: 0.0,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 0.0,
            height,
            width_adjustment_factor: 1.0,
            height_adjustment_factor: 0.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            ht_root,
            height,
        }
    }

    const fn height(&self) -> f32 {
        self.height
    }

    fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }
}

struct AppHeaderHitTestActionHandler {
    menu_button_view: AppMenuButtonView,
}
impl HitTestTreeActionHandler for AppHeaderHitTestActionHandler {
    type Context = AppState;

    fn on_pointer_enter(
        &self,
        sender: HitTestTreeRef,
        _context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
        if sender == self.menu_button_view.ht_root {
            let (rel_x_dip, rel_y_dip, _, _) = ht.translate_client_to_tree_local(
                sender,
                client_x,
                client_y,
                client_width,
                client_height,
            );
            self.menu_button_view
                .set_opacity_by_local_pos(rel_x_dip, rel_y_dip);

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_leave(
        &self,
        sender: HitTestTreeRef,
        _context: &mut Self::Context,
        _ht: &mut HitTestTreeManager<Self::Context>,
        _client_x: f32,
        _client_y: f32,
        _client_width: f32,
        _client_height: f32,
    ) -> EventContinueControl {
        if sender == self.menu_button_view.ht_root {
            self.menu_button_view.bg.SetOpacity(0.0).unwrap();

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_move(
        &self,
        sender: HitTestTreeRef,
        _context: &mut Self::Context,
        ht: &mut HitTestTreeManager<Self::Context>,
        client_x: f32,
        client_y: f32,
        client_width: f32,
        client_height: f32,
    ) -> EventContinueControl {
        if sender == self.menu_button_view.ht_root {
            let (rel_x_dip, rel_y_dip, _, _) = ht.translate_client_to_tree_local(
                sender,
                client_x,
                client_y,
                client_width,
                client_height,
            );
            self.menu_button_view
                .set_opacity_by_local_pos(rel_x_dip, rel_y_dip);

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_click(
        &self,
        sender: HitTestTreeRef,
        context: &mut Self::Context,
        _ht: &mut HitTestTreeManager<Self::Context>,
        _client_x: f32,
        _client_y: f32,
        _client_width: f32,
        _client_height: f32,
    ) -> EventContinueControl {
        if sender == self.menu_button_view.ht_root {
            context.toggle_menu();

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }
}

pub struct AppHeaderPresenter {
    base_view: AppHeaderBaseView,
    close_button_view: AppCloseButtonView,
    close_button_rect_rel: RectDIP,
    minimize_button_view: AppMinimizeButtonView,
    minimize_button_rect_rel: RectDIP,
    _ht_action_handler: Rc<AppHeaderHitTestActionHandler>,
}
impl AppHeaderPresenter {
    pub fn new(init: &mut PresenterInitContext, init_title: &str) -> Self {
        let base_view = AppHeaderBaseView::new(&mut init.for_view, init_title);
        let close_button_view = AppCloseButtonView::new(&mut init.for_view);
        let minimize_button_view = AppMinimizeButtonView::new(&mut init.for_view);
        let menu_button_view = AppMenuButtonView::new(&mut init.for_view);

        let spc = (base_view.height - AppCloseButtonView::BUTTON_SIZE) * 0.5;
        close_button_view.locate(
            Vector3 {
                X: init
                    .for_view
                    .dip_to_pixels(-spc - AppCloseButtonView::BUTTON_SIZE),
                Y: init.for_view.dip_to_pixels(spc),
                Z: 0.0,
            },
            Vector3 {
                X: 1.0,
                Y: 0.0,
                Z: 0.0,
            },
        );
        let close_button_rect_rel = RectDIP {
            left: -spc - AppCloseButtonView::BUTTON_SIZE,
            top: spc,
            right: -spc,
            bottom: spc + AppCloseButtonView::BUTTON_SIZE,
        };

        minimize_button_view.locate(
            Vector3 {
                X: init.for_view.dip_to_pixels(
                    close_button_rect_rel.left - 6.0 - AppMinimizeButtonView::BUTTON_SIZE,
                ),
                Y: init.for_view.dip_to_pixels(spc),
                Z: 0.0,
            },
            Vector3 {
                X: 1.0,
                Y: 0.0,
                Z: 0.0,
            },
        );
        let minimize_button_rect_rel = RectDIP {
            left: close_button_rect_rel.left - 6.0 - AppMinimizeButtonView::BUTTON_SIZE,
            top: spc,
            right: close_button_rect_rel.left - 6.0,
            bottom: spc + AppMinimizeButtonView::BUTTON_SIZE,
        };

        menu_button_view.set_size(base_view.height, &mut init.for_view.ht.borrow_mut());

        let children = base_view.root.Children().unwrap();
        close_button_view.mount(&children);
        minimize_button_view.mount(&children);
        menu_button_view.mount(
            &children,
            &mut init.for_view.ht.borrow_mut(),
            base_view.ht_root,
        );

        let ht_action_handler = Rc::new(AppHeaderHitTestActionHandler { menu_button_view });
        init.for_view
            .ht
            .borrow_mut()
            .get_mut(ht_action_handler.menu_button_view.ht_root)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);

        Self {
            base_view,
            close_button_view,
            close_button_rect_rel,
            minimize_button_view,
            minimize_button_rect_rel,
            _ht_action_handler: ht_action_handler,
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut AppHitTestTreeManager,
        ht_parent: HitTestTreeRef,
    ) {
        self.base_view.mount(children, ht, ht_parent);
    }

    pub const fn height(&self) -> f32 {
        self.base_view.height()
    }

    pub fn nc_hittest(&self, p: &PointDIP, client_size: &Size) -> Option<u32> {
        if p.x <= self.base_view.height() {
            // Menu Buttonのぶんはクライアント領域判定
            return Some(HTCLIENT);
        }

        let rel_p = p.make_rel_from(&PointDIP {
            x: client_size.Width,
            y: 0.0,
        });

        if self.close_button_rect_rel.contains(&rel_p) {
            return Some(HTCLOSE);
        }

        if self.minimize_button_rect_rel.contains(&rel_p) {
            return Some(HTMINBUTTON);
        }

        if p.y < self.base_view.height {
            return Some(HTCAPTION);
        }

        None
    }
}
