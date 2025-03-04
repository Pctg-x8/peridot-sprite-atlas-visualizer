use core::mem::MaybeUninit;
use windows::{
    UI::Composition::CompositionDrawingSurface,
    Win32::{
        Foundation::POINT, Graphics::Direct2D::ID2D1DeviceContext,
        System::WinRT::Composition::ICompositionDrawingSurfaceInterop,
    },
};
use windows_core::Interface;

#[inline]
pub fn draw_2d<T, E>(
    surface: &CompositionDrawingSurface,
    renderer: impl FnOnce(&ID2D1DeviceContext, &POINT) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<windows_core::Error>,
{
    let surface_interop: ICompositionDrawingSurfaceInterop = surface
        .cast()
        .expect("this surface does not support rendering with Direct2D");

    let mut offset = MaybeUninit::uninit();
    let dc: ID2D1DeviceContext = unsafe { surface_interop.BeginDraw(None, offset.as_mut_ptr())? };
    let r = renderer(&dc, unsafe { offset.assume_init_ref() });
    // Note: rendererがエラーでもEndDrawは必ずする
    unsafe {
        surface_interop.EndDraw()?;
    }

    r
}
