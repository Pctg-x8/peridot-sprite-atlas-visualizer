use windows::{
    Foundation::TimeSpan,
    UI::Composition::{
        CompositionBrush, CompositionColorGradientStop, CompositionEasingFunction,
        CompositionLinearGradientBrush, CompositionMaskBrush, CompositionNineGridBrush,
        CompositionStretch, CompositionSurfaceBrush, Compositor, ContainerVisual,
        ICompositionSurface, ScalarKeyFrameAnimation, SpriteVisual, Vector3KeyFrameAnimation,
    },
};
use windows_core::{HSTRING, h};
use windows_numerics::{Vector2, Vector3};

pub struct CompositionColorGradientStopParams {
    pub offset: f32,
    pub color: windows::UI::Color,
}
impl CompositionColorGradientStopParams {
    #[inline]
    pub fn instantiate(
        &self,
        compositor: &Compositor,
    ) -> windows_core::Result<CompositionColorGradientStop> {
        compositor.CreateColorGradientStopWithOffsetAndColor(self.offset, self.color)
    }
}

pub struct CompositionLinearGradientBrushParams<'s> {
    pub stops: &'s [CompositionColorGradientStopParams],
    pub start_point: Vector2,
    pub end_point: Vector2,
}
impl CompositionLinearGradientBrushParams<'_> {
    #[inline]
    pub fn instantiate(
        self,
        compositor: &Compositor,
    ) -> windows_core::Result<CompositionLinearGradientBrush> {
        let x = compositor.CreateLinearGradientBrush()?;
        x.SetStartPoint(self.start_point)?;
        x.SetEndPoint(self.end_point)?;
        let stops = x.ColorStops()?;
        for x in self.stops {
            stops.Append(&x.instantiate(compositor)?)?;
        }

        Ok(x)
    }
}

pub struct CompositionSurfaceBrushParams<Surface> {
    pub surface: Surface,
    pub stretch: Option<CompositionStretch>,
}
impl<Surface> CompositionSurfaceBrushParams<Surface> {
    pub const fn new(surface: Surface) -> Self {
        Self {
            surface,
            stretch: None,
        }
    }

    pub const fn stretch(mut self, stretch: CompositionStretch) -> Self {
        self.stretch = Some(stretch);
        self
    }
}
impl<Surface> CompositionSurfaceBrushParams<Surface>
where
    Surface: windows::core::Param<ICompositionSurface>,
{
    #[inline]
    pub fn instantiate(
        self,
        compositor: &Compositor,
    ) -> windows::core::Result<CompositionSurfaceBrush> {
        let x = compositor.CreateSurfaceBrushWithSurface(self.surface)?;
        if let Some(p) = self.stretch {
            x.SetStretch(p)?;
        }

        Ok(x)
    }
}

pub struct CompositionNineGridBrushParams<Source> {
    pub source: Source,
    pub insets: Option<f32>,
}
impl<Source> CompositionNineGridBrushParams<Source> {
    pub const fn new(source: Source) -> Self {
        Self {
            source,
            insets: None,
        }
    }

    pub const fn insets(mut self, insets: f32) -> Self {
        self.insets = Some(insets);
        self
    }
}
impl<Source> CompositionNineGridBrushParams<Source>
where
    Source: windows::core::Param<CompositionBrush>,
{
    #[inline]
    pub fn instantiate(
        self,
        compositor: &Compositor,
    ) -> windows::core::Result<CompositionNineGridBrush> {
        let x = compositor.CreateNineGridBrush()?;
        x.SetSource(self.source)?;
        if let Some(p) = self.insets {
            x.SetInsets(p)?;
        }

        Ok(x)
    }
}

pub struct CompositionMaskBrushParams<Source, Mask> {
    pub source: Source,
    pub mask: Mask,
}
impl<Source, Mask> CompositionMaskBrushParams<Source, Mask>
where
    Source: windows::core::Param<CompositionBrush>,
    Mask: windows::core::Param<CompositionBrush>,
{
    #[inline]
    pub fn instantiate(
        self,
        compositor: &Compositor,
    ) -> windows::core::Result<CompositionMaskBrush> {
        let x = compositor.CreateMaskBrush()?;
        x.SetSource(self.source)?;
        x.SetMask(self.mask)?;

        Ok(x)
    }
}

macro_rules! CoordinateElementFunctions {
    {} => {
        #[allow(dead_code)]
        pub const fn offset(mut self, offset: Vector3) -> Self {
            self.offset = Some(offset);
            self
        }

        #[allow(dead_code)]
        pub const fn offset_xy(mut self, offset: Vector2) -> Self {
            self.offset = Some(match self.offset {
                None => Vector3 { X: offset.X, Y: offset.Y, Z: 0.0 },
                Some(x) => Vector3 { X: offset.X, Y: offset.Y, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn left(mut self, left: f32) -> Self {
            self.offset = Some(match self.offset {
                None => Vector3 {
                    X: left,
                    Y: 0.0,
                    Z: 0.0,
                },
                Some(x) => Vector3 { X: left, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn relative_offset_adjustment(mut self, adjustment: Vector3) -> Self {
            self.relative_offset_adjustment = Some(adjustment);
            self
        }

        #[allow(dead_code)]
        pub const fn relative_offset_adjustment_xy(mut self, adjustment: Vector2) -> Self {
            self.relative_offset_adjustment = Some(match self.relative_offset_adjustment {
                None => Vector3 { X: adjustment.X, Y: adjustment.Y, Z: 0.0 },
                Some(x) => Vector3 { X: adjustment.X, Y: adjustment.Y, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn relative_horizontal_offset_adjustment(mut self, a: f32) -> Self {
            self.relative_offset_adjustment = Some(match self.relative_offset_adjustment {
                None => Vector3 { X: a, Y: 0.0, Z: 0.0 },
                Some(x) => Vector3 { X: a, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn relative_vertical_offset_adjustment(mut self, a: f32) -> Self {
            self.relative_offset_adjustment = Some(match self.relative_offset_adjustment {
                None => Vector3 { X: 0.0, Y: a, Z: 0.0 },
                Some(x) => Vector3 { Y: a, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn size(mut self, size: Vector2) -> Self {
            self.size = Some(size);
            self
        }

        #[allow(dead_code)]
        pub const fn size_sq(self, x: f32) -> Self {
            self.size(Vector2 { X: x, Y: x })
        }

        #[allow(dead_code)]
        pub const fn width(mut self, width: f32) -> Self {
            self.size = Some(match self.size {
                None => Vector2 { X: width, Y: 0.0 },
                Some(x) => Vector2 { X: width, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn height(mut self, height: f32) -> Self {
            self.size = match self.size {
                None => Some(Vector2 { X: 0.0, Y: height }),
                Some(x) => Some(Vector2 { Y: height, ..x }),
            };

            self
        }

        #[allow(dead_code)]
        pub const fn relative_size_adjustment(mut self, adjustment: Vector2) -> Self {
            self.relative_size_adjustment = Some(adjustment);
            self
        }

        #[allow(dead_code)]
        pub const fn expand(self) -> Self {
            self.relative_size_adjustment(Vector2 { X: 1.0, Y: 1.0 })
        }

        #[allow(dead_code)]
        pub const fn expand_width(mut self) -> Self {
            self.relative_size_adjustment = match self.relative_size_adjustment {
                None => Some(Vector2 { X: 1.0, Y: 0.0 }),
                Some(x) => Some(Vector2 { X: 1.0, ..x }),
            };

            self
        }

        #[allow(dead_code)]
        pub const fn expand_height(mut self) -> Self {
            self.relative_size_adjustment = Some(match self.relative_size_adjustment {
                None => Vector2 { X: 0.0, Y: 1.0 },
                Some(x) => Vector2 { Y: 1.0, ..x },
            });

            self
        }

        #[allow(dead_code)]
        pub const fn anchor_point(mut self, p: Vector2) -> Self {
            self.anchor_point = Some(p);
            self
        }
    };
}

pub struct ContainerVisualParams {
    pub offset: Option<Vector3>,
    pub relative_offset_adjustment: Option<Vector3>,
    pub size: Option<Vector2>,
    pub relative_size_adjustment: Option<Vector2>,
    pub anchor_point: Option<Vector2>,
}
impl ContainerVisualParams {
    pub const fn new() -> Self {
        Self {
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: None,
            anchor_point: None,
        }
    }

    CoordinateElementFunctions! {}
}
impl ContainerVisualParams {
    #[inline]
    pub fn instantiate(self, compositor: &Compositor) -> windows::core::Result<ContainerVisual> {
        let x = compositor.CreateContainerVisual()?;
        if let Some(p) = self.offset {
            x.SetOffset(p)?;
        }
        if let Some(p) = self.relative_offset_adjustment {
            x.SetRelativeOffsetAdjustment(p)?;
        }
        if let Some(p) = self.size {
            x.SetSize(p)?;
        }
        if let Some(p) = self.relative_size_adjustment {
            x.SetRelativeSizeAdjustment(p)?;
        }
        if let Some(p) = self.anchor_point {
            x.SetAnchorPoint(p)?;
        }

        Ok(x)
    }
}

pub struct SpriteVisualParams<Brush> {
    pub brush: Brush,
    pub offset: Option<Vector3>,
    pub relative_offset_adjustment: Option<Vector3>,
    pub size: Option<Vector2>,
    pub relative_size_adjustment: Option<Vector2>,
    pub opacity: Option<f32>,
    pub anchor_point: Option<Vector2>,
}
impl<Brush> SpriteVisualParams<Brush> {
    pub const fn new(brush: Brush) -> Self {
        Self {
            brush,
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: None,
            opacity: None,
            anchor_point: None,
        }
    }

    CoordinateElementFunctions! {}

    pub const fn opacity(mut self, x: f32) -> Self {
        self.opacity = Some(x);
        self
    }
}
impl<Brush> SpriteVisualParams<Brush>
where
    Brush: windows::core::Param<CompositionBrush>,
{
    #[inline]
    pub fn instantiate(self, compositor: &Compositor) -> windows::core::Result<SpriteVisual> {
        let x = compositor.CreateSpriteVisual()?;
        x.SetBrush(self.brush)?;
        if let Some(p) = self.offset {
            x.SetOffset(p)?;
        }
        if let Some(p) = self.relative_offset_adjustment {
            x.SetRelativeOffsetAdjustment(p)?;
        }
        if let Some(p) = self.size {
            x.SetSize(p)?;
        }
        if let Some(p) = self.relative_size_adjustment {
            x.SetRelativeSizeAdjustment(p)?;
        }
        if let Some(p) = self.opacity {
            x.SetOpacity(p)?;
        }
        if let Some(p) = self.anchor_point {
            x.SetAnchorPoint(p)?;
        }

        Ok(x)
    }
}

pub struct SimpleScalarAnimationParams<'s, Easing> {
    pub start_value: f32,
    pub end_value: f32,
    pub easing: Easing,
    pub duration: Option<TimeSpan>,
    pub target: Option<&'s HSTRING>,
}
impl<'s, Easing> SimpleScalarAnimationParams<'s, Easing> {
    pub const fn new(start_value: f32, end_value: f32, easing: Easing) -> Self {
        Self {
            start_value,
            end_value,
            easing,
            duration: None,
            target: None,
        }
    }

    pub const fn duration(mut self, d: TimeSpan) -> Self {
        self.duration = Some(d);
        self
    }

    pub const fn target(mut self, target: &'s HSTRING) -> Self {
        self.target = Some(target);
        self
    }
}
impl<Easing> SimpleScalarAnimationParams<'_, Easing>
where
    Easing: windows_core::Param<CompositionEasingFunction>,
{
    #[inline]
    pub fn instantiate(
        self,
        compositor: &Compositor,
    ) -> windows_core::Result<ScalarKeyFrameAnimation> {
        let x = compositor.CreateScalarKeyFrameAnimation()?;
        x.InsertKeyFrame(0.0, self.start_value)?;
        x.InsertKeyFrameWithEasingFunction(1.0, self.end_value, self.easing)?;

        if let Some(p) = self.duration {
            x.SetDuration(p)?;
        }
        if let Some(p) = self.target {
            x.SetTarget(p)?;
        }

        Ok(x)
    }
}

pub struct SimpleImplicitAnimationParams<'s, Easing> {
    easing: Easing,
    target: &'s HSTRING,
    duration: TimeSpan,
}
impl<'s, Easing> SimpleImplicitAnimationParams<'s, Easing> {
    pub const fn new(easing: Easing, target: &'s HSTRING, duration: TimeSpan) -> Self {
        Self {
            easing,
            target,
            duration,
        }
    }
}
impl<'s, Easing> SimpleImplicitAnimationParams<'s, Easing>
where
    Easing: windows_core::Param<CompositionEasingFunction>,
{
    #[inline]
    pub fn instantiate_scalar(
        self,
        compositor: &Compositor,
    ) -> windows_core::Result<ScalarKeyFrameAnimation> {
        let x = compositor.CreateScalarKeyFrameAnimation()?;
        x.InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))?;
        x.InsertExpressionKeyFrameWithEasingFunction(1.0, h!("this.FinalValue"), self.easing)?;
        x.SetTarget(self.target)?;
        x.SetDuration(self.duration)?;

        Ok(x)
    }

    #[inline]
    pub fn instantiate_vector3(
        self,
        compositor: &Compositor,
    ) -> windows_core::Result<Vector3KeyFrameAnimation> {
        let x = compositor.CreateVector3KeyFrameAnimation()?;
        x.InsertExpressionKeyFrame(0.0, h!("this.StartingValue"))?;
        x.InsertExpressionKeyFrameWithEasingFunction(1.0, h!("this.FinalValue"), self.easing)?;
        x.SetTarget(self.target)?;
        x.SetDuration(self.duration)?;

        Ok(x)
    }
}
