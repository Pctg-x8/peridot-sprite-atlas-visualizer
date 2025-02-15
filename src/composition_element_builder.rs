use windows::{
    Foundation::Numerics::{Vector2, Vector3},
    UI::Composition::{
        CompositionBrush, CompositionMaskBrush, CompositionNineGridBrush, CompositionStretch,
        CompositionSurfaceBrush, Compositor, ContainerVisual, ICompositionSurface, SpriteVisual,
    },
};

pub struct CompositionSurfaceBrushParams<Surface> {
    pub surface: Surface,
    pub stretch: Option<CompositionStretch>,
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

pub struct ContainerVisualParams {
    pub offset: Option<Vector3>,
    pub relative_offset_adjustment: Option<Vector3>,
    pub size: Option<Vector2>,
    pub relative_size_adjustment: Option<Vector2>,
}
impl Default for ContainerVisualParams {
    fn default() -> Self {
        Self {
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: None,
        }
    }
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

        Ok(x)
    }
}

pub struct SpriteVisualParams<Brush> {
    pub brush: Brush,
    pub offset: Option<Vector3>,
    pub relative_offset_adjustment: Option<Vector3>,
    pub size: Option<Vector2>,
    pub relative_size_adjustment: Option<Vector2>,
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

        Ok(x)
    }
}
