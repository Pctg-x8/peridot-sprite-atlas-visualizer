use windows::{
    Foundation::Size,
    UI::Composition::{
        CompositionEffectSourceParameter, CompositionPropertySet, ScalarKeyFrameAnimation,
        SpriteVisual, VisualCollection,
    },
    Win32::Graphics::{
        Direct2D::{
            Common::{D2D_POINT_2F, D2D1_COLOR_F},
            D2D1_DRAW_TEXT_OPTIONS_NONE,
        },
        DirectWrite::{DWRITE_FONT_WEIGHT_SEMI_LIGHT, DWRITE_TEXT_RANGE},
    },
};
use windows_core::{HSTRING, Interface, h};
use windows_numerics::Vector2;

use crate::{
    ViewInitContext,
    color_factory::{d2d1_color_f_from_websafe_hex_rgb, ui_color_from_websafe_hex_rgb_with_alpha},
    composition_element_builder::{
        CompositionSurfaceBrushParams, SimpleScalarAnimationParams, SpriteVisualParams,
    },
    effect_builder::{ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams},
    extra_bindings::Microsoft::Graphics::Canvas::CanvasComposite,
    surface_helper::draw_2d,
    timespan_helper::timespan_ms,
};

pub struct FileDragAndDropOverlayView {
    root: SpriteVisual,
    composition_params: CompositionPropertySet,
    show_animation: ScalarKeyFrameAnimation,
    hide_animation: ScalarKeyFrameAnimation,
}
impl FileDragAndDropOverlayView {
    const BASE_COLOR: windows::UI::Color = ui_color_from_websafe_hex_rgb_with_alpha(0xfff, 64);
    const TEXT_COLOR: D2D1_COLOR_F = d2d1_color_f_from_websafe_hex_rgb(0xccc);

    pub fn new(init: &mut ViewInitContext) -> Self {
        let effect_factory = init
            .subsystem
            .compositor
            .CreateEffectFactoryWithProperties(
                &CompositeEffectParams::new(&[
                    GaussianBlurEffectParams::new(
                        &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                    )
                    .name(h!("Blur"))
                    .instantiate()
                    .unwrap()
                    .cast()
                    .unwrap(),
                    ColorSourceEffectParams {
                        color: Some(Self::BASE_COLOR),
                    }
                    .instantiate()
                    .unwrap()
                    .cast()
                    .unwrap(),
                ])
                .mode(CanvasComposite::SourceOver)
                .instantiate()
                .unwrap(),
                &windows_collections::IIterable::<HSTRING>::from(vec![
                    h!("Blur.BlurAmount").clone(),
                ]),
            )
            .unwrap();

        let bg_brush = effect_factory.CreateBrush().unwrap();
        bg_brush
            .SetSourceParameter(
                h!("source"),
                &init.subsystem.compositor.CreateBackdropBrush().unwrap(),
            )
            .unwrap();
        let root = SpriteVisualParams::new(&bg_brush)
            .expand()
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        let label_text_utf16 = "ドロップしてファイルを追加"
            .encode_utf16()
            .collect::<Vec<_>>();
        let label = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &label_text_utf16,
                    &init.subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
        unsafe {
            label
                .SetFontSize(
                    96.0,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: label_text_utf16.len() as _,
                    },
                )
                .unwrap();
            label
                .SetFontWeight(
                    DWRITE_FONT_WEIGHT_SEMI_LIGHT,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: label_text_utf16.len() as _,
                    },
                )
                .unwrap();
        }
        let mut metrics = core::mem::MaybeUninit::uninit();
        unsafe {
            label.GetMetrics(metrics.as_mut_ptr()).unwrap();
        }
        let metrics = unsafe { metrics.assume_init() };
        let label_surface = init
            .subsystem
            .new_2d_drawing_surface(Size {
                Width: init.dip_to_pixels(metrics.width),
                Height: init.dip_to_pixels(metrics.height),
            })
            .unwrap();
        draw_2d(&label_surface, |dc, offset| {
            unsafe {
                dc.SetDpi(init.dpi, init.dpi);

                dc.Clear(None);
                dc.DrawTextLayout(
                    D2D_POINT_2F {
                        x: init.signed_pixels_to_dip(offset.x),
                        y: init.signed_pixels_to_dip(offset.y),
                    },
                    &label,
                    &dc.CreateSolidColorBrush(&Self::TEXT_COLOR, None)?,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                );
            }

            Ok::<_, windows_core::Error>(())
        })
        .unwrap();
        let label = SpriteVisualParams::new(
            &CompositionSurfaceBrushParams::new(&label_surface)
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
        )
        .size(Vector2 {
            X: init.dip_to_pixels(metrics.width),
            Y: init.dip_to_pixels(metrics.height),
        })
        .anchor_point(Vector2 { X: 0.5, Y: 0.5 })
        .relative_offset_adjustment_xy(Vector2 { X: 0.5, Y: 0.5 })
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        root.Children().unwrap().InsertAtTop(&label).unwrap();

        let composition_params = init.subsystem.compositor.CreatePropertySet().unwrap();
        composition_params.InsertScalar(h!("Rate"), 0.0).unwrap();

        let bg_blur_expression = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!("cp.Rate * 63.0 / 3.0"))
            .unwrap();
        bg_blur_expression
            .SetExpressionReferenceParameter(h!("cp"), &composition_params)
            .unwrap();
        bg_brush
            .Properties()
            .unwrap()
            .InsertScalar(h!("Blur.BlurAmount"), 0.0)
            .unwrap();
        bg_brush
            .Properties()
            .unwrap()
            .StartAnimation(h!("Blur.BlurAmount"), &bg_blur_expression)
            .unwrap();

        let opacity_expression = init
            .subsystem
            .compositor
            .CreateExpressionAnimationWithExpression(h!("cp.Rate"))
            .unwrap();
        opacity_expression
            .SetExpressionReferenceParameter(h!("cp"), &composition_params)
            .unwrap();
        root.SetOpacity(0.0).unwrap();
        root.StartAnimation(h!("Opacity"), &opacity_expression)
            .unwrap();

        let easing = init
            .subsystem
            .compositor
            .CreateCubicBezierEasingFunction(Vector2 { X: 0.5, Y: 0.0 }, Vector2 { X: 0.5, Y: 1.0 })
            .unwrap();
        let show_animation = SimpleScalarAnimationParams::new(0.0, 1.0, &easing)
            .duration(timespan_ms(200))
            .instantiate(&init.subsystem.compositor)
            .unwrap();
        let hide_animation = SimpleScalarAnimationParams::new(1.0, 0.0, &easing)
            .duration(timespan_ms(200))
            .instantiate(&init.subsystem.compositor)
            .unwrap();

        Self {
            root,
            composition_params,
            show_animation,
            hide_animation,
        }
    }

    pub fn mount(&self, children: &VisualCollection) {
        children.InsertAtTop(&self.root).unwrap();
    }

    pub fn show(&self) {
        self.composition_params
            .StartAnimation(h!("Rate"), &self.show_animation)
            .unwrap();
    }

    pub fn hide(&self) {
        self.composition_params
            .StartAnimation(h!("Rate"), &self.hide_animation)
            .unwrap();
    }
}
