use core::mem::MaybeUninit;
use windows::{
    Foundation::Size,
    UI::Composition::{CompositionEffectSourceParameter, ContainerVisual, VisualCollection},
    Win32::{
        Graphics::Direct2D::{
            Common::{D2D_POINT_2F, D2D1_GRADIENT_STOP},
            D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_GAMMA_2_2,
            D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES, ID2D1DeviceContext, ID2D1RenderTarget,
            ID2D1SolidColorBrush,
        },
        UI::WindowsAndMessaging::{HTCAPTION, HTCLOSE, HTMINBUTTON},
    },
};
use windows_core::{Interface, h};
use windows_numerics::{Matrix3x2, Vector2, Vector3};

use crate::{
    D2D1_COLOR_F_WHITE, PointDIP, RectDIP, ViewInitContext,
    color_factory::{
        d2d1_color_f_from_websafe_hex_argb, d2d1_color_f_from_websafe_hex_rgb,
        ui_color_from_websafe_hex_rgb_with_alpha,
    },
    composition_element_builder::{
        CompositionColorGradientStopParams, CompositionLinearGradientBrushParams,
        CompositionMaskBrushParams, CompositionSurfaceBrushParams, ContainerVisualParams,
        SpriteVisualParams,
    },
    coordinate::size_sq,
    create_instant_effect_brush,
    effect_builder::{ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams},
    extra_bindings::Microsoft::Graphics::Canvas::CanvasComposite,
    surface_helper::draw_2d,
};

#[inline(always)]
fn new_icon_brush(dc: &ID2D1DeviceContext) -> windows_core::Result<ID2D1SolidColorBrush> {
    unsafe { dc.CreateSolidColorBrush(&d2d1_color_f_from_websafe_hex_rgb(0x111), None) }
}

pub struct AppCloseButtonView {
    root: ContainerVisual,
}
impl AppCloseButtonView {
    const BUTTON_SIZE: f32 = 24.0;
    const ICON_SIZE: f32 = 6.0;
    const SURFACE_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 128);

    pub fn new(init: &mut ViewInitContext) -> Self {
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

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn locate(&self, offset: Vector3, relative_offset_adjustment: Vector3) {
        self.root.SetOffset(offset).unwrap();
        self.root
            .SetRelativeOffsetAdjustment(relative_offset_adjustment)
            .unwrap();
    }
}

pub struct AppMinimizeButtonView {
    root: ContainerVisual,
}
impl AppMinimizeButtonView {
    const BUTTON_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 6.0;
    const SURFACE_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 128);

    pub fn new(init: &mut ViewInitContext) -> Self {
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

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn locate(&self, offset: Vector3, relative_offset_adjustment: Vector3) {
        self.root.SetOffset(offset).unwrap();
        self.root
            .SetRelativeOffsetAdjustment(relative_offset_adjustment)
            .unwrap();
    }
}

pub struct AppHeaderView {
    root: ContainerVisual,
    close_button_view: AppCloseButtonView,
    minimize_button_view: AppMinimizeButtonView,
    height: f32,
    close_button_rect_rel: RectDIP,
    minimize_button_rect_rel: RectDIP,
}
impl AppHeaderView {
    pub fn new(init: &mut ViewInitContext, init_label: &str) -> Self {
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
            X: init.dip_to_pixels(24.0),
            Y: init.dip_to_pixels(16.0),
        })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let close_button_view = AppCloseButtonView::new(init);
        let minimize_button_view = AppMinimizeButtonView::new(init);

        let spc = (height - AppCloseButtonView::BUTTON_SIZE) * 0.5;
        close_button_view.locate(
            Vector3 {
                X: init.dip_to_pixels(-spc - AppCloseButtonView::BUTTON_SIZE),
                Y: init.dip_to_pixels(spc),
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
                X: init.dip_to_pixels(
                    close_button_rect_rel.left - 6.0 - AppMinimizeButtonView::BUTTON_SIZE,
                ),
                Y: init.dip_to_pixels(spc),
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

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&label).unwrap();
        close_button_view.mount(&children);
        minimize_button_view.mount(&children);

        Self {
            root,
            close_button_view,
            close_button_rect_rel,
            minimize_button_view,
            minimize_button_rect_rel,
            height,
        }
    }

    pub const fn height(&self) -> f32 {
        self.height
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn unmount(&self) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
    }

    pub fn nc_hittest(&self, p: &PointDIP, client_size: &Size) -> Option<u32> {
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

        if p.y < self.height {
            return Some(HTCAPTION);
        }

        None
    }
}
