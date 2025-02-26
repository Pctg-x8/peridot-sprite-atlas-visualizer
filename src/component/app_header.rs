use windows::{
    Foundation::Size,
    Win32::{
        Graphics::Direct2D::{
            Common::{D2D1_COLOR_F, D2D1_GRADIENT_STOP, D2D_POINT_2F},
            ID2D1DeviceContext, ID2D1RenderTarget, ID2D1SolidColorBrush,
            D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_GAMMA_2_2,
            D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
        },
        System::WinRT::Composition::ICompositionDrawingSurfaceInterop,
        UI::WindowsAndMessaging::{HTCAPTION, HTCLOSE, HTMINBUTTON},
    },
    UI::Composition::{CompositionEffectSourceParameter, ContainerVisual, VisualCollection},
};
use windows_core::{h, Interface};
use windows_numerics::{Vector2, Vector3};

use crate::{
    composition_element_builder::{
        CompositionColorGradientStopParams, CompositionLinearGradientBrushParams,
        CompositionMaskBrushParams, CompositionSurfaceBrushParams, ContainerVisualParams,
        SpriteVisualParams,
    },
    create_instant_effect_brush,
    effect_builder::{ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams},
    extra_bindings::Microsoft::Graphics::Canvas::CanvasComposite,
    scoped_try, PointDIP, RectDIP, ViewInitContext, D2D1_COLOR_F_WHITE,
};

#[inline(always)]
fn new_icon_brush(dc: &ID2D1DeviceContext) -> windows_core::Result<ID2D1SolidColorBrush> {
    unsafe {
        dc.CreateSolidColorBrush(
            &D2D1_COLOR_F {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            None,
        )
    }
}

pub struct AppCloseButtonView {
    root: ContainerVisual,
}
impl AppCloseButtonView {
    const BUTTON_SIZE: f32 = 24.0;
    const ICON_SIZE: f32 = 6.0;
    const SURFACE_COLOR: windows::UI::Color = windows::UI::Color {
        A: 128,
        R: 255,
        G: 255,
        B: 255,
    };

    pub fn new(init: &mut ViewInitContext) -> Self {
        let icon_size_px = init.dip_to_pixels(Self::ICON_SIZE);
        let button_size_px = init.dip_to_pixels(Self::BUTTON_SIZE);

        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: icon_size_px,
                Height: icon_size_px,
            })
            .unwrap();
        {
            let interop = icon_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!('drawing, new_icon_brush(&dc));

                let offset_x_dip = init.signed_pixels_to_dip(offset.x);
                let offset_y_dip = init.signed_pixels_to_dip(offset.y);

                unsafe {
                    dc.Clear(None);
                    dc.DrawLine(
                        D2D_POINT_2F {
                            x: offset_x_dip,
                            y: offset_y_dip,
                        },
                        D2D_POINT_2F {
                            x: offset_x_dip + Self::ICON_SIZE,
                            y: offset_y_dip + Self::ICON_SIZE,
                        },
                        &brush,
                        1.5,
                        None,
                    );
                    dc.DrawLine(
                        D2D_POINT_2F {
                            x: offset_x_dip + Self::ICON_SIZE,
                            y: offset_y_dip,
                        },
                        D2D_POINT_2F {
                            x: offset_x_dip,
                            y: offset_y_dip + Self::ICON_SIZE,
                        },
                        &brush,
                        1.5,
                        None,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

        let circle_mask_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: button_size_px,
                Height: button_size_px,
            })
            .unwrap();
        {
            let interop = circle_mask_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let offset_x_dip = init.signed_pixels_to_dip(offset.x);
                let offset_y_dip = init.signed_pixels_to_dip(offset.y);

                let gradient_stops = scoped_try!('drawing, unsafe {
                    dc.cast::<ID2D1RenderTarget>()
                        .unwrap()
                        .CreateGradientStopCollection(
                            &[
                                D2D1_GRADIENT_STOP {
                                    position: 0.0,
                                    color: D2D1_COLOR_F {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 1.0,
                                    },
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 0.75,
                                    color: D2D1_COLOR_F {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 1.0,
                                    },
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 1.0,
                                    color: D2D1_COLOR_F {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 0.0,
                                    },
                                },
                            ],
                            D2D1_GAMMA_2_2,
                            D2D1_EXTEND_MODE_CLAMP,
                        )
                });
                let gradient_brush = scoped_try!('drawing, unsafe {
                    dc.CreateRadialGradientBrush(
                        &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                            center: D2D_POINT_2F {
                                x: offset_x_dip + Self::BUTTON_SIZE * 0.5,
                                y: offset_y_dip + Self::BUTTON_SIZE * 0.5,
                            },
                            radiusX: Self::BUTTON_SIZE * 0.5,
                            radiusY: Self::BUTTON_SIZE * 0.5,
                            gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 },
                        },
                        None,
                        &gradient_stops,
                    )
                });

                unsafe {
                    dc.Clear(None);
                    dc.FillEllipse(
                        &D2D1_ELLIPSE {
                            point: D2D_POINT_2F {
                                x: offset_x_dip + Self::BUTTON_SIZE * 0.5,
                                y: offset_y_dip + Self::BUTTON_SIZE * 0.5,
                            },
                            radiusX: Self::BUTTON_SIZE * 0.5,
                            radiusY: Self::BUTTON_SIZE * 0.5,
                        },
                        &gradient_brush,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

        let root = ContainerVisualParams::new()
            .size_sq(button_size_px)
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let bg_brush = create_instant_effect_brush(
            init.subsystem,
            &CompositeEffectParams {
                sources: &[
                    GaussianBlurEffectParams {
                        source: &CompositionEffectSourceParameter::Create(h!("backdrop")).unwrap(),
                        blur_amount: Some(9.0 / 3.0),
                    }
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
                ],
                mode: Some(CanvasComposite::SourceOver),
            }
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
                mask: &CompositionSurfaceBrushParams {
                    surface: &circle_mask_surface,
                    stretch: None,
                }
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
            &init
                .subsystem
                .compositor
                .CreateSurfaceBrushWithSurface(&icon_surface)
                .unwrap(),
        )
        .size_sq(icon_size_px)
        .offset_xy(Vector2 {
            X: -icon_size_px * 0.5,
            Y: -icon_size_px * 0.5,
        })
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
}

pub struct AppMinimizeButtonView {
    root: ContainerVisual,
}
impl AppMinimizeButtonView {
    const BUTTON_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 6.0;
    const SURFACE_COLOR: windows::UI::Color = windows::UI::Color {
        A: 128,
        R: 255,
        G: 255,
        B: 255,
    };

    pub fn new(init: &mut ViewInitContext) -> Self {
        let icon_size_px = init.dip_to_pixels(Self::ICON_SIZE);
        let button_size_px = init.dip_to_pixels(Self::BUTTON_SIZE);

        let icon_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: icon_size_px,
                Height: icon_size_px,
            })
            .unwrap();
        {
            let interop = icon_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!('drawing, new_icon_brush(&dc));

                let offset_x_dip = init.signed_pixels_to_dip(offset.x);
                let offset_y_dip = init.signed_pixels_to_dip(offset.y);

                unsafe {
                    dc.Clear(None);
                    dc.DrawLine(
                        D2D_POINT_2F {
                            x: offset_x_dip,
                            y: offset_y_dip + Self::ICON_SIZE - 0.5,
                        },
                        D2D_POINT_2F {
                            x: offset_x_dip + Self::ICON_SIZE,
                            y: offset_y_dip + Self::ICON_SIZE - 0.5,
                        },
                        &brush,
                        1.5,
                        None,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

        let circle_mask_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: button_size_px,
                Height: button_size_px,
            })
            .unwrap();
        {
            let interop = circle_mask_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let offset_x_dip = init.signed_pixels_to_dip(offset.x);
                let offset_y_dip = init.signed_pixels_to_dip(offset.y);

                // Create gradient stops collection
                let gradient_stops = match unsafe {
                    dc.cast::<ID2D1RenderTarget>()
                        .unwrap()
                        .CreateGradientStopCollection(
                            &[
                                D2D1_GRADIENT_STOP {
                                    position: 0.0,
                                    color: D2D1_COLOR_F {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 1.0,
                                    },
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 0.75,
                                    color: D2D1_COLOR_F {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 1.0,
                                    },
                                },
                                D2D1_GRADIENT_STOP {
                                    position: 1.0,
                                    color: D2D1_COLOR_F {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 0.0,
                                    },
                                },
                            ],
                            D2D1_GAMMA_2_2,
                            D2D1_EXTEND_MODE_CLAMP,
                        )
                } {
                    Ok(x) => x,
                    Err(e) => break 'drawing Err(e),
                };

                // Create radial gradient brush
                let gradient_brush = match unsafe {
                    dc.CreateRadialGradientBrush(
                        &D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
                            center: D2D_POINT_2F {
                                x: offset_x_dip + Self::BUTTON_SIZE * 0.5,
                                y: offset_y_dip + Self::BUTTON_SIZE * 0.5,
                            },
                            radiusX: Self::BUTTON_SIZE * 0.5,
                            radiusY: Self::BUTTON_SIZE * 0.5,
                            gradientOriginOffset: D2D_POINT_2F { x: 0.0, y: 0.0 },
                        },
                        None,
                        &gradient_stops,
                    )
                } {
                    Ok(x) => x,
                    Err(e) => break 'drawing Err(e),
                };

                unsafe {
                    dc.Clear(None);
                    dc.FillEllipse(
                        &D2D1_ELLIPSE {
                            point: D2D_POINT_2F {
                                x: offset_x_dip + Self::BUTTON_SIZE * 0.5,
                                y: offset_y_dip + Self::BUTTON_SIZE * 0.5,
                            },
                            radiusX: Self::BUTTON_SIZE * 0.5,
                            radiusY: Self::BUTTON_SIZE * 0.5,
                        },
                        &gradient_brush,
                    );
                }

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

        let root = ContainerVisualParams::new()
            .size_sq(init.dip_to_pixels(Self::BUTTON_SIZE))
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let bg_brush = create_instant_effect_brush(
            init.subsystem,
            &CompositeEffectParams {
                sources: &[
                    GaussianBlurEffectParams {
                        source: &CompositionEffectSourceParameter::Create(h!("backdrop")).unwrap(),
                        blur_amount: Some(9.0 / 3.0),
                    }
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
                ],
                mode: Some(CanvasComposite::SourceOver),
            }
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
                mask: &CompositionSurfaceBrushParams {
                    surface: &circle_mask_surface,
                    stretch: None,
                }
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
            &init
                .subsystem
                .compositor
                .CreateSurfaceBrushWithSurface(&icon_surface)
                .unwrap(),
        )
        .size_sq(icon_size_px)
        .offset_xy(Vector2 {
            X: -icon_size_px * 0.5,
            Y: -icon_size_px * 0.5,
        })
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
        let tl = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &init_label.encode_utf16().collect::<Vec<_>>(),
                    &init.subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
        let mut tm = core::mem::MaybeUninit::uninit();
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
        {
            let interop = label_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                }

                let brush = scoped_try!('drawing, unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None) });

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

                Ok(())
            };
            unsafe {
                interop.EndDraw().unwrap();
            }
            r.unwrap();
        }

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
                        color: windows::UI::Color {
                            A: 128,
                            R: 0,
                            G: 0,
                            B: 0,
                        },
                    },
                    CompositionColorGradientStopParams {
                        offset: 1.0,
                        color: windows::UI::Color {
                            A: 32,
                            R: 0,
                            G: 0,
                            B: 0,
                        },
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

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&label).unwrap();

        let spc = (height - AppCloseButtonView::BUTTON_SIZE) * 0.5;
        let close_button_view = AppCloseButtonView::new(init);
        close_button_view.mount(&children);
        close_button_view
            .root
            .SetOffset(Vector3 {
                X: init.dip_to_pixels(-spc - AppCloseButtonView::BUTTON_SIZE),
                Y: init.dip_to_pixels(spc),
                Z: 0.0,
            })
            .unwrap();
        close_button_view
            .root
            .SetRelativeOffsetAdjustment(Vector3 {
                X: 1.0,
                Y: 0.0,
                Z: 0.0,
            })
            .unwrap();
        let close_button_rect_rel = RectDIP {
            left: -spc - AppCloseButtonView::BUTTON_SIZE,
            top: spc,
            right: -spc,
            bottom: spc + AppCloseButtonView::BUTTON_SIZE,
        };

        let minimize_button_view = AppMinimizeButtonView::new(init);
        minimize_button_view.mount(&children);
        minimize_button_view
            .root
            .SetOffset(Vector3 {
                X: init.dip_to_pixels(
                    close_button_rect_rel.left - (6.0 + AppMinimizeButtonView::BUTTON_SIZE),
                ),
                Y: init.dip_to_pixels(spc),
                Z: 0.0,
            })
            .unwrap();
        minimize_button_view
            .root
            .SetRelativeOffsetAdjustment(Vector3 {
                X: 1.0,
                Y: 0.0,
                Z: 0.0,
            })
            .unwrap();
        let minimize_button_rect_rel = RectDIP {
            left: close_button_rect_rel.left - 6.0 - AppMinimizeButtonView::BUTTON_SIZE,
            top: spc,
            right: close_button_rect_rel.left - 6.0,
            bottom: spc + AppMinimizeButtonView::BUTTON_SIZE,
        };

        Self {
            root,
            close_button_view,
            close_button_rect_rel,
            minimize_button_view,
            minimize_button_rect_rel,
            height,
        }
    }

    #[inline(always)]
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
