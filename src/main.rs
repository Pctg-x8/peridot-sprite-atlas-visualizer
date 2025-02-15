use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use composition_element_builder::{
    CompositionMaskBrushParams, CompositionNineGridBrushParams, CompositionSurfaceBrushParams,
    ContainerVisualParams, SpriteVisualParams,
};
use effect_builder::{ColorSourceEffectParams, CompositeEffectParams, GaussianBlurEffectParams};
use extra_bindings::Microsoft::Graphics::Canvas::CanvasComposite;
use hittest::HitTestTreeActionHandler;
use subsystem::Subsystem;
use windows::{
    core::{h, w, Interface, HRESULT, PCWSTR},
    Foundation::{
        Numerics::{Vector2, Vector3},
        Size, TimeSpan,
    },
    Graphics::{
        DirectX::{DirectXAlphaMode, DirectXPixelFormat},
        Effects::IGraphicsEffect,
    },
    Win32::{
        Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, RECT, WAIT_OBJECT_0, WPARAM},
        Graphics::{
            Direct2D::{
                CLSID_D2D1AlphaMask, CLSID_D2D1ColorMatrix, CLSID_D2D1ConvolveMatrix,
                Common::{
                    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BORDER_MODE, D2D1_BORDER_MODE_SOFT,
                    D2D1_COLOR_F, D2D1_COMPOSITE_MODE_SOURCE_OVER, D2D1_GRADIENT_STOP,
                    D2D1_PIXEL_FORMAT, D2D_MATRIX_5X4_F, D2D_MATRIX_5X4_F_0, D2D_MATRIX_5X4_F_0_0,
                    D2D_POINT_2F, D2D_RECT_F,
                },
                ID2D1DeviceContext, ID2D1RenderTarget, D2D1_BITMAP_OPTIONS_NONE,
                D2D1_BITMAP_PROPERTIES1, D2D1_COLORMATRIX_PROP_COLOR_MATRIX,
                D2D1_CONVOLVEMATRIX_PROP_BORDER_MODE, D2D1_CONVOLVEMATRIX_PROP_DIVISOR,
                D2D1_CONVOLVEMATRIX_PROP_KERNEL_MATRIX, D2D1_CONVOLVEMATRIX_PROP_KERNEL_SIZE_X,
                D2D1_CONVOLVEMATRIX_PROP_KERNEL_SIZE_Y, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE,
                D2D1_EXTEND_MODE_CLAMP, D2D1_FEATURE_LEVEL_DEFAULT, D2D1_GAMMA_2_2,
                D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR, D2D1_PROPERTY_TYPE_ENUM,
                D2D1_PROPERTY_TYPE_MATRIX_5X4, D2D1_PROPERTY_TYPE_UINT32,
                D2D1_PROPERTY_TYPE_UNKNOWN, D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
                D2D1_RENDER_TARGET_PROPERTIES, D2D1_RENDER_TARGET_TYPE_HARDWARE,
                D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
            },
            Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            Direct3D11::{
                ID3D11Buffer, ID3D11PixelShader, ID3D11Texture2D, ID3D11VertexShader,
                D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE,
                D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE, D3D11_MAP_WRITE_DISCARD,
                D3D11_RENDER_TARGET_VIEW_DESC, D3D11_RENDER_TARGET_VIEW_DESC_0,
                D3D11_RTV_DIMENSION_TEXTURE2D, D3D11_SUBRESOURCE_DATA, D3D11_TEX2D_RTV,
                D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT,
            },
            DirectWrite::{
                IDWriteTextLayout1, DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_MEDIUM,
                DWRITE_FONT_WEIGHT_SEMI_BOLD, DWRITE_TEXT_RANGE,
            },
            Dwm::{
                DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE,
            },
            Dxgi::{
                Common::{DXGI_ALPHA_MODE_IGNORE, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
                IDXGIAdapter, IDXGIDevice2, IDXGIFactory2, IDXGISurface, IDXGISwapChain2,
                DXGI_PRESENT, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT, DXGI_SWAP_EFFECT_FLIP_DISCARD,
                DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
            Gdi::{MapWindowPoints, HBRUSH},
        },
        Storage::Packaging::Appx::PACKAGE_VERSION,
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::INFINITE,
            WinRT::{
                Composition::ICompositionDrawingSurfaceInterop, CreateDispatcherQueueController,
                DispatcherQueueOptions, DQTAT_COM_ASTA, DQTYPE_THREAD_CURRENT,
            },
        },
        UI::{
            Controls::MARGINS,
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetSystemMetrics,
                GetWindowLongPtrW, GetWindowRect, LoadCursorW, LoadIconW,
                MsgWaitForMultipleObjects, PeekMessageW, PostQuitMessage, RegisterClassExW,
                SetCursor, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage,
                CW_USEDEFAULT, GWLP_USERDATA, HCURSOR, HTCAPTION, HTCLIENT, HTCLOSE, HTMINBUTTON,
                HTTOP, IDC_ARROW, IDC_SIZEWE, IDI_APPLICATION, NCCALCSIZE_PARAMS, PM_REMOVE,
                QS_ALLINPUT, SM_CXSIZEFRAME, SM_CYSIZEFRAME, SWP_FRAMECHANGED, SW_SHOW,
                WM_ACTIVATE, WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP,
                WM_MOUSEMOVE, WM_NCCALCSIZE, WM_NCHITTEST, WM_QUIT, WM_SETCURSOR, WM_SIZE,
                WNDCLASSEXW, WNDCLASS_STYLES, WS_EX_APPWINDOW, WS_EX_NOREDIRECTIONBITMAP,
                WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
    UI::{
        Color,
        Composition::{
            CompositionBrush, CompositionDrawingSurface, CompositionEffectBrush,
            CompositionEffectSourceParameter, CompositionStretch, ContainerVisual,
            Desktop::DesktopWindowTarget, ScalarKeyFrameAnimation, SpriteVisual, VisualCollection,
        },
    },
};
use windows_core::HSTRING;

mod composition_element_builder;
mod effect_builder;
mod extra_bindings;
mod hittest;
mod input;
mod subsystem;

use crate::hittest::*;
use crate::input::*;

macro_rules! scoped_try {
    ($label: tt, $x: expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => break $label Err(e)
        }
    }
}

const fn timespan_ms(ms: u32) -> TimeSpan {
    TimeSpan {
        Duration: (10_000 * ms) as _,
    }
}

const fn pixels_to_dip(pixels: u32, dpi: f32) -> f32 {
    pixels as f32 * 96.0 / dpi
}
const fn signed_pixels_to_dip(pixels: i32, dpi: f32) -> f32 {
    pixels as f32 * 96.0 / dpi
}

const fn dip_to_pixels(dip: f32, dpi: f32) -> f32 {
    dip * dpi / 96.0
}

pub struct SizePixels {
    pub width: u32,
    pub height: u32,
}
impl SizePixels {
    pub const fn to_dip(&self, dpi: f32) -> Size {
        Size {
            Width: pixels_to_dip(self.width, dpi),
            Height: pixels_to_dip(self.height, dpi),
        }
    }
}

pub struct PointDIP {
    pub x: f32,
    pub y: f32,
}
impl PointDIP {
    pub const fn make_rel_from(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub struct RectDIP {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl RectDIP {
    pub const fn contains(&self, p: &PointDIP) -> bool {
        self.left <= p.x && p.x <= self.right && self.top <= p.y && p.y <= self.bottom
    }
}

// windows app sdk bootstrapping
#[repr(C)]
#[derive(Clone, Copy)]
enum MddBootstrapInitializeOptions {
    ShowUI = 0x08,
}
// copy from WindowsAppSDK-VersionInfo.h
const APP_SDK_VERSION_U64: u64 = 0;

#[link(name = "Microsoft.WindowsAppRuntime.Bootstrap")]
unsafe extern "system" {
    unsafe fn MddBootstrapInitialize2(
        majorMinorVersion: u32,
        versionTag: PCWSTR,
        minVersion: PACKAGE_VERSION,
        options: MddBootstrapInitializeOptions,
    ) -> HRESULT;

    unsafe fn MddBootstrapShutdown();
}

struct AppRuntime;
impl AppRuntime {
    #[inline(always)]
    pub fn init() -> windows::core::Result<Self> {
        unsafe {
            MddBootstrapInitialize2(
                0x00010006,
                w!(""),
                core::mem::transmute(APP_SDK_VERSION_U64),
                MddBootstrapInitializeOptions::ShowUI,
            )
            .ok()?;
        }

        Ok(Self)
    }
}
impl Drop for AppRuntime {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            MddBootstrapShutdown();
        }
    }
}

const fn rgb_color_from_hex(hex: u32) -> Color {
    Color {
        R: ((hex >> 16) & 0xff) as _,
        G: ((hex >> 8) & 0xff) as _,
        B: (hex & 0xff) as _,
        A: 255,
    }
}

const fn rgb_color_from_websafe_hex(hex: u16) -> Color {
    const fn e(x: u8) -> u8 {
        x | (x << 4)
    }

    Color {
        R: e(((hex >> 8) & 0x0f) as _),
        G: e(((hex >> 4) & 0x0f) as _),
        B: e((hex & 0x0f) as _),
        A: 255,
    }
}

const BG_COLOR: Color = rgb_color_from_hex(0x202030);
const PANE_BG_COLOR: Color = rgb_color_from_websafe_hex(0x333);
const SEPARATOR_COLOR: Color = rgb_color_from_websafe_hex(0x666);

pub trait DpiHandler {
    #[allow(unused_variables)]
    fn on_dpi_changed(&self, new_dpi: f32) {}
}

pub struct PresenterInitContext<'r> {
    pub for_view: ViewInitContext<'r>,
    pub dpi_handlers: &'r mut Vec<std::rc::Weak<dyn DpiHandler>>,
}
pub struct ViewInitContext<'r> {
    pub subsystem: &'r Subsystem,
    pub ht: &'r mut HitTestTreeContext,
    pub dpi: f32,
}

pub struct FileListCellView {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    dpi: Cell<f32>,
    y: Cell<f32>,
}
impl FileListCellView {
    const CELL_HEIGHT: f32 = 20.0;

    pub fn new(init: &mut ViewInitContext, init_label: &str) -> Self {
        let root = init.subsystem.compositor.CreateContainerVisual().unwrap();
        root.SetSize(Vector2 {
            X: 0.0,
            Y: dip_to_pixels(Self::CELL_HEIGHT, init.dpi),
        })
        .unwrap();
        root.SetRelativeSizeAdjustment(Vector2 { X: 1.0, Y: 0.0 })
            .unwrap();

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
        let mut text_metrics = core::mem::MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(text_metrics.as_mut_ptr()).unwrap();
        }
        let text_metrics = unsafe { text_metrics.assume_init() };
        let text_surface = init
            .subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(text_metrics.width, init.dpi),
                    Height: dip_to_pixels(text_metrics.height, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
            .unwrap();
        {
            let surface_interop = text_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext = unsafe {
                surface_interop
                    .BeginDraw(None, offset.as_mut_ptr())
                    .unwrap()
            };
            let offset = unsafe { offset.assume_init() };

            let r = 'drawing_block: {
                let brush = match unsafe {
                    dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                        None,
                    )
                } {
                    Ok(b) => b,
                    Err(e) => break 'drawing_block Err(e),
                };

                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                    dc.Clear(Some(&D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }));
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: pixels_to_dip(offset.x as _, init.dpi),
                            y: pixels_to_dip(offset.y as _, init.dpi),
                        },
                        &tl,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };

            unsafe {
                surface_interop.EndDraw().unwrap();
            }
            r.unwrap();
        }
        let text_vis = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        text_vis
            .SetBrush(
                &init
                    .subsystem
                    .compositor
                    .CreateSurfaceBrushWithSurface(&text_surface)
                    .unwrap(),
            )
            .unwrap();
        text_vis
            .SetSize(Vector2 {
                X: dip_to_pixels(text_metrics.width, init.dpi),
                Y: dip_to_pixels(text_metrics.height, init.dpi),
            })
            .unwrap();
        text_vis
            .SetOffset(Vector3 {
                X: dip_to_pixels(8.0, init.dpi),
                Y: dip_to_pixels((Self::CELL_HEIGHT - text_metrics.height) * 0.5, init.dpi),
                Z: 0.0,
            })
            .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&text_vis).unwrap();

        let ht_root = init.ht.alloc(HitTestTreeData {
            left: 0.0,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 0.0,
            height: Self::CELL_HEIGHT,
            width_adjustment_factor: 1.0,
            height_adjustment_factor: 0.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            ht_root,
            dpi: Cell::new(init.dpi),
            y: Cell::new(0.0),
        }
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn drop_context(&self, ht: &mut HitTestTreeContext) {
        ht.free_rec(self.ht_root);
    }

    pub fn set_y(&self, ht: &mut HitTestTreeContext, y: f32) {
        self.root
            .SetOffset(Vector3 {
                X: 0.0,
                Y: dip_to_pixels(y, self.dpi.get()),
                Z: 0.0,
            })
            .unwrap();
        ht.get_mut(self.ht_root).top = y;

        self.y.set(y);
    }
}

pub struct FileListView {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    dpi: Cell<f32>,
    width: Cell<f32>,
    height: Cell<f32>,
}
impl FileListView {
    const FRAME_IMAGE_SIZE_PIXELS: u32 = 128;

    fn gen_frame_image(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let mut d3d_tex2d = core::mem::MaybeUninit::uninit();
        unsafe {
            subsystem
                .d3d11_device
                .CreateTexture2D(
                    &D3D11_TEXTURE2D_DESC {
                        Width: Self::FRAME_IMAGE_SIZE_PIXELS,
                        Height: Self::FRAME_IMAGE_SIZE_PIXELS,
                        MipLevels: 1,
                        ArraySize: 1,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Usage: D3D11_USAGE_DEFAULT,
                        BindFlags: (D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE).0 as _,
                        CPUAccessFlags: 0,
                        MiscFlags: 0,
                    },
                    None,
                    Some(d3d_tex2d.as_mut_ptr()),
                )
                .unwrap();
        }
        let d3d_tex2d = unsafe { d3d_tex2d.assume_init().unwrap() };
        let base = d3d_tex2d.cast::<IDXGISurface>().unwrap();
        let d2d1_rt = unsafe {
            subsystem
                .d2d1_device
                .GetFactory()
                .unwrap()
                .CreateDxgiSurfaceRenderTarget(
                    &base,
                    &D2D1_RENDER_TARGET_PROPERTIES {
                        r#type: D2D1_RENDER_TARGET_TYPE_HARDWARE,
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                        },
                        dpiX: dpi,
                        dpiY: dpi,
                        usage: D2D1_RENDER_TARGET_USAGE_NONE,
                        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
                    },
                )
                .unwrap()
        };
        unsafe {
            d2d1_rt.BeginDraw();
        }
        let r = 'drawing: {
            let geometry = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: pixels_to_dip(Self::FRAME_IMAGE_SIZE_PIXELS, dpi) - 0.0,
                    bottom: pixels_to_dip(Self::FRAME_IMAGE_SIZE_PIXELS, dpi) - 0.0,
                },
                radiusX: pixels_to_dip(8, dpi),
                radiusY: pixels_to_dip(8, dpi),
            };

            let bg_brush = match unsafe {
                d2d1_rt.CreateSolidColorBrush(
                    &D2D1_COLOR_F {
                        r: 0.15,
                        g: 0.15,
                        b: 0.15,
                        a: 1.0,
                    },
                    None,
                )
            } {
                Ok(x) => x,
                Err(e) => break 'drawing Err(e),
            };

            unsafe {
                d2d1_rt.SetDpi(dpi, dpi);
                d2d1_rt.Clear(Some(&D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }));

                d2d1_rt.FillRoundedRectangle(&geometry, &bg_brush);
            }

            Ok(())
        };
        unsafe {
            d2d1_rt.EndDraw(None, None).unwrap();
        }
        r.unwrap();
        drop(d2d1_rt);

        let ds = subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: Self::FRAME_IMAGE_SIZE_PIXELS as _,
                    Height: Self::FRAME_IMAGE_SIZE_PIXELS as _,
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
            .unwrap();
        let interop: ICompositionDrawingSurfaceInterop = ds.cast().unwrap();
        let mut offset = core::mem::MaybeUninit::uninit();
        let dc: ID2D1DeviceContext =
            unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
        let offset = unsafe { offset.assume_init() };
        let r = 'drawing: {
            let base_image = match unsafe {
                dc.CreateBitmapFromDxgiSurface(
                    &base,
                    Some(&D2D1_BITMAP_PROPERTIES1 {
                        pixelFormat: D2D1_PIXEL_FORMAT {
                            format: DXGI_FORMAT_B8G8R8A8_UNORM,
                            alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                        },
                        dpiX: dpi,
                        dpiY: dpi,
                        bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                        colorContext: core::mem::ManuallyDrop::new(None),
                    }),
                )
            } {
                Ok(x) => x,
                Err(e) => break 'drawing Err(e),
            };

            let blur_effect = match unsafe { dc.CreateEffect(&CLSID_D2D1ConvolveMatrix) } {
                Ok(x) => x,
                Err(e) => break 'drawing Err(e),
            };
            let color_matrix_effect = match unsafe { dc.CreateEffect(&CLSID_D2D1ColorMatrix) } {
                Ok(x) => x,
                Err(e) => break 'drawing Err(e),
            };
            let alpha_mask_effect = match unsafe { dc.CreateEffect(&CLSID_D2D1AlphaMask) } {
                Ok(x) => x,
                Err(e) => break 'drawing Err(e),
            };

            unsafe {
                const CONV_MAX_DIST: i32 = 16;
                let stdev = 16.0f32 / 3.0;
                let conv_matrix = (-CONV_MAX_DIST..=CONV_MAX_DIST)
                    .flat_map(|yd| {
                        (-CONV_MAX_DIST..=CONV_MAX_DIST).map(move |xd| {
                            if xd < 0 || yd < 0 {
                                0.0
                            } else {
                                // gaussian distirbution
                                (core::f32::consts::TAU * stdev.powf(2.0)).recip()
                                    * (-((xd as f32).powf(2.0) + (yd as f32).powf(2.0))
                                        / (2.0 * stdev.powf(2.0)))
                                    .exp()
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                let matrix_size = (CONV_MAX_DIST * 2 + 1) as u32;
                let div: f32 = conv_matrix.iter().sum();

                blur_effect.SetInput(0, &base_image, true);
                blur_effect
                    .SetValue(
                        D2D1_CONVOLVEMATRIX_PROP_KERNEL_SIZE_X.0 as _,
                        D2D1_PROPERTY_TYPE_UINT32,
                        core::mem::transmute::<_, &[u8; 4]>(&matrix_size),
                    )
                    .unwrap();
                blur_effect
                    .SetValue(
                        D2D1_CONVOLVEMATRIX_PROP_KERNEL_SIZE_Y.0 as _,
                        D2D1_PROPERTY_TYPE_UINT32,
                        core::mem::transmute::<_, &[u8; 4]>(&matrix_size),
                    )
                    .unwrap();
                blur_effect
                    .SetValue(
                        D2D1_CONVOLVEMATRIX_PROP_KERNEL_MATRIX.0 as _,
                        D2D1_PROPERTY_TYPE_UNKNOWN,
                        core::slice::from_raw_parts(
                            conv_matrix.as_ptr() as *const u8,
                            conv_matrix.len() * 4,
                        ),
                    )
                    .unwrap();
                blur_effect
                    .SetValue(
                        D2D1_CONVOLVEMATRIX_PROP_DIVISOR.0 as _,
                        D2D1_PROPERTY_TYPE_UNKNOWN,
                        core::mem::transmute::<_, &[u8; 4]>(&div),
                    )
                    .unwrap();
                blur_effect
                    .SetValue(
                        D2D1_CONVOLVEMATRIX_PROP_BORDER_MODE.0 as _,
                        D2D1_PROPERTY_TYPE_ENUM,
                        core::mem::transmute::<_, &[u8; core::mem::size_of::<D2D1_BORDER_MODE>()]>(
                            &D2D1_BORDER_MODE_SOFT,
                        ),
                    )
                    .unwrap();

                // color matrix: (r, g, b, a) -> (0, 0, 0, 0.5 - 0.5 * a)
                color_matrix_effect.SetInput(0, &blur_effect.GetOutput().unwrap(), false);
                color_matrix_effect
                    .SetValue(
                        D2D1_COLORMATRIX_PROP_COLOR_MATRIX.0 as _,
                        D2D1_PROPERTY_TYPE_MATRIX_5X4,
                        core::mem::transmute::<_, &[u8; core::mem::size_of::<D2D_MATRIX_5X4_F>()]>(
                            &D2D_MATRIX_5X4_F {
                                Anonymous: D2D_MATRIX_5X4_F_0 {
                                    Anonymous: D2D_MATRIX_5X4_F_0_0 {
                                        _11: 0.0,
                                        _12: 0.0,
                                        _13: 0.0,
                                        _14: 0.0,
                                        _21: 0.0,
                                        _22: 0.0,
                                        _23: 0.0,
                                        _24: 0.0,
                                        _31: 0.0,
                                        _32: 0.0,
                                        _33: 0.0,
                                        _34: 0.0,
                                        _41: 0.0,
                                        _42: 0.0,
                                        _43: 0.0,
                                        _44: -0.25,
                                        _51: 0.0,
                                        _52: 0.0,
                                        _53: 0.0,
                                        _54: 0.25,
                                    },
                                },
                            },
                        ),
                    )
                    .unwrap();

                alpha_mask_effect.SetInput(0, &color_matrix_effect.GetOutput().unwrap(), false);
                alpha_mask_effect.SetInput(1, &base_image, false);
            }

            let drawing_base = D2D_POINT_2F {
                x: signed_pixels_to_dip(offset.x, dpi),
                y: signed_pixels_to_dip(offset.y, dpi),
            };

            unsafe {
                dc.SetDpi(dpi, dpi);
                dc.Clear(Some(&D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                }));

                dc.DrawImage(
                    &base_image,
                    Some(&drawing_base),
                    Some(&D2D_RECT_F {
                        left: 0.0,
                        top: 0.0,
                        right: Self::FRAME_IMAGE_SIZE_PIXELS as _,
                        bottom: Self::FRAME_IMAGE_SIZE_PIXELS as _,
                    }),
                    D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                    D2D1_COMPOSITE_MODE_SOURCE_OVER,
                );
                dc.DrawImage(
                    &alpha_mask_effect.GetOutput().unwrap(),
                    Some(&drawing_base),
                    Some(&D2D_RECT_F {
                        left: 0.0,
                        top: 0.0,
                        right: Self::FRAME_IMAGE_SIZE_PIXELS as _,
                        bottom: Self::FRAME_IMAGE_SIZE_PIXELS as _,
                    }),
                    D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                    D2D1_COMPOSITE_MODE_SOURCE_OVER,
                );
            }

            Ok(())
        };
        unsafe {
            interop.EndDraw().unwrap();
        }
        r.unwrap();

        ds
    }

    pub fn new(init: &mut ViewInitContext) -> Self {
        let root = init.subsystem.compositor.CreateContainerVisual().unwrap();
        root.SetSize(Vector2 { X: 256.0, Y: 128.0 }).unwrap();
        root.SetOffset(Vector3 {
            X: dip_to_pixels(8.0, init.dpi),
            Y: dip_to_pixels(8.0 + 16.0, init.dpi),
            Z: 0.0,
        })
        .unwrap();

        let frame_source_brush = init
            .subsystem
            .compositor
            .CreateSurfaceBrushWithSurface(&Self::gen_frame_image(&init.subsystem, init.dpi))
            .unwrap();
        frame_source_brush
            .SetStretch(CompositionStretch::Fill)
            .unwrap();
        let frame_brush = init.subsystem.compositor.CreateNineGridBrush().unwrap();
        frame_brush.SetSource(&frame_source_brush).unwrap();
        frame_brush.SetInsets(8.0).unwrap();
        frame_brush.SetTopInset(32.0).unwrap();
        frame_brush.SetLeftInset(32.0).unwrap();

        let frame = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        frame.SetBrush(&frame_brush).unwrap();
        frame.SetRelativeSizeAdjustment(Vector2::one()).unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&frame).unwrap();

        let ht_root = init.ht.alloc(HitTestTreeData {
            left: 8.0,
            top: 8.0 + 16.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 256.0,
            height: 128.0,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 0.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            ht_root,
            dpi: Cell::new(init.dpi),
            width: Cell::new(256.0),
            height: Cell::new(128.0),
        }
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn drop_context(&self, ht: &mut HitTestTreeContext) {
        ht.free_rec(self.ht_root);
    }

    pub fn set_width(&self, ht: &mut HitTestTreeContext, width: f32) {
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(width, self.dpi.get()),
                Y: dip_to_pixels(self.height.get(), self.dpi.get()),
            })
            .unwrap();
        ht.get_mut(self.ht_root).width = width;

        self.width.set(width);
    }
}

pub struct SpriteListPresenter {
    view: FileListView,
    cell_views: Rc<RefCell<Vec<FileListCellView>>>,
}
impl SpriteListPresenter {
    pub fn new(init: &mut PresenterInitContext) -> Self {
        let view = FileListView::new(&mut init.for_view);

        let mut cells = Vec::new();
        let cell = FileListCellView::new(&mut init.for_view, "example_sprite_1");
        cell.mount(
            &mut init.for_view.ht,
            &view.root.Children().unwrap(),
            view.ht_root,
        );
        cell.set_y(&mut init.for_view.ht, 8.0);
        cells.push(cell);
        let cell = FileListCellView::new(&mut init.for_view, "example_ui_player_card/bg");
        cell.mount(
            &mut init.for_view.ht,
            &view.root.Children().unwrap(),
            view.ht_root,
        );
        cell.set_y(&mut init.for_view.ht, 8.0 + FileListCellView::CELL_HEIGHT);
        cells.push(cell);
        let cell =
            FileListCellView::new(&mut init.for_view, "example_ui_player_card/name_underline");
        cell.mount(
            &mut init.for_view.ht,
            &view.root.Children().unwrap(),
            view.ht_root,
        );
        cell.set_y(
            &mut init.for_view.ht,
            8.0 + FileListCellView::CELL_HEIGHT * 2.0,
        );
        cells.push(cell);

        Self {
            view,
            cell_views: Rc::new(RefCell::new(cells)),
        }
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_parent: HitTestTreeRef,
    ) {
        self.view.mount(ht, children, ht_parent);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.view.unmount(ht);
    }

    pub fn drop_context(&self, ht: &mut HitTestTreeContext) {
        self.view.drop_context(ht);
    }

    pub fn set_width(&self, ht: &mut HitTestTreeContext, width: f32) {
        self.view.set_width(ht, width);
    }
}

pub struct RightPaneContentView {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    dpi: Cell<f32>,
    width: Cell<f32>,
}
impl RightPaneContentView {
    pub fn new(init: &mut ViewInitContext, init_width: f32) -> Self {
        let root = init.subsystem.compositor.CreateContainerVisual().unwrap();
        root.SetRelativeOffsetAdjustment(Vector3 {
            X: 1.0,
            Y: 0.0,
            Z: 0.0,
        })
        .unwrap();
        root.SetOffset(Vector3 {
            X: -dip_to_pixels(init_width, init.dpi),
            Y: 0.0,
            Z: 0.0,
        })
        .unwrap();
        root.SetRelativeSizeAdjustment(Vector2 { X: 0.0, Y: 1.0 })
            .unwrap();
        root.SetSize(Vector2 {
            X: dip_to_pixels(init_width, init.dpi),
            Y: 0.0,
        })
        .unwrap();

        let bg = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        bg.SetBrush(
            &init
                .subsystem
                .compositor
                .CreateColorBrushWithColor(PANE_BG_COLOR)
                .unwrap(),
        )
        .unwrap();
        bg.SetRelativeSizeAdjustment(Vector2::one()).unwrap();

        let tf = init
            .subsystem
            .text_format_store
            .get(h!("system-ui"), 12.0)
            .unwrap();
        let tl = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &"Sprites:".encode_utf16().collect::<Vec<_>>(),
                    &tf,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
        let mut text_metrics = core::mem::MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(text_metrics.as_mut_ptr()).unwrap();
        }
        let text_metrics = unsafe { text_metrics.assume_init() };
        let text_surface = init
            .subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(text_metrics.width, init.dpi),
                    Height: dip_to_pixels(text_metrics.height, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
            .unwrap();
        {
            let surface_interop = text_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext = unsafe {
                surface_interop
                    .BeginDraw(None, offset.as_mut_ptr())
                    .unwrap()
            };
            let offset = unsafe { offset.assume_init() };

            let r = 'drawing_block: {
                let brush = match unsafe {
                    dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                        None,
                    )
                } {
                    Ok(b) => b,
                    Err(e) => break 'drawing_block Err(e),
                };

                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: -pixels_to_dip(offset.x as _, init.dpi),
                            y: -pixels_to_dip(offset.y as _, init.dpi),
                        },
                        &tl,
                        &brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                    );
                }

                Ok(())
            };

            unsafe {
                surface_interop.EndDraw().unwrap();
            }
            r.unwrap();
        }
        let text_vis = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        text_vis
            .SetBrush(
                &init
                    .subsystem
                    .compositor
                    .CreateSurfaceBrushWithSurface(&text_surface)
                    .unwrap(),
            )
            .unwrap();
        text_vis
            .SetSize(Vector2 {
                X: dip_to_pixels(text_metrics.width, init.dpi),
                Y: dip_to_pixels(text_metrics.height, init.dpi),
            })
            .unwrap();
        text_vis
            .SetOffset(Vector3 {
                X: dip_to_pixels(8.0, init.dpi),
                Y: dip_to_pixels(8.0, init.dpi),
                Z: 0.0,
            })
            .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtBottom(&bg).unwrap();
        children.InsertAtTop(&text_vis).unwrap();

        let ht_root = init.ht.alloc(HitTestTreeData {
            left: -init_width,
            top: 0.0,
            left_adjustment_factor: 1.0,
            top_adjustment_factor: 0.0,
            width: init_width,
            height: 0.0,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            ht_root,
            dpi: Cell::new(init.dpi),
            width: Cell::new(init_width),
        }
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_root: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_root, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn set_width(&self, ht: &mut HitTestTreeContext, width: f32) {
        let width_px = dip_to_pixels(width, self.dpi.get());

        self.root
            .SetSize(Vector2 {
                X: width_px,
                Y: 0.0,
            })
            .unwrap();
        self.root
            .SetOffset(Vector3 {
                X: -width_px,
                Y: 0.0,
                Z: 0.0,
            })
            .unwrap();
        ht.get_mut(self.ht_root).width = width;
        ht.get_mut(self.ht_root).left = -width;

        self.width.set(width);
    }

    pub fn set_dpi(&self, dpi: f32) {
        let width_px = dip_to_pixels(self.width.get(), dpi);

        self.root
            .SetSize(Vector2 {
                X: width_px,
                Y: 0.0,
            })
            .unwrap();
        self.root
            .SetOffset(Vector3 {
                X: -width_px,
                Y: 0.0,
                Z: 0.0,
            })
            .unwrap();

        self.dpi.set(dpi);
    }

    pub fn drop_context(&self, ht: &mut HitTestTreeContext) {
        ht.free_rec(self.ht_root);
    }
}

pub struct RightPaneContentDpiHandler {
    view: Rc<RightPaneContentView>,
}
impl DpiHandler for RightPaneContentDpiHandler {
    fn on_dpi_changed(&self, new_dpi: f32) {
        self.view.set_dpi(new_dpi);
    }
}

pub struct RightPaneContentPresenter {
    view: Rc<RightPaneContentView>,
    sprite_list: SpriteListPresenter,
    dpi_handler: Rc<RightPaneContentDpiHandler>,
}
impl RightPaneContentPresenter {
    pub fn new(init: &mut PresenterInitContext, init_width: f32) -> Self {
        let view = Rc::new(RightPaneContentView::new(&mut init.for_view, init_width));
        let sprite_list = SpriteListPresenter::new(init);

        sprite_list.set_width(init.for_view.ht, (init_width - 16.0).max(16.0));
        sprite_list.mount(
            init.for_view.ht,
            &view.root.Children().unwrap(),
            view.ht_root,
        );

        let dpi_handler = Rc::new(RightPaneContentDpiHandler { view: view.clone() });
        init.dpi_handlers.push(Rc::downgrade(&dpi_handler) as _);

        Self {
            view,
            sprite_list,
            dpi_handler,
        }
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_root: HitTestTreeRef,
    ) {
        self.view.mount(ht, children, ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.view.unmount(ht);
    }

    pub fn width(&self) -> f32 {
        self.view.width.get()
    }

    pub fn set_width(&self, ht: &mut HitTestTreeContext, width: f32) {
        self.view.set_width(ht, width);
        self.sprite_list.set_width(ht, (width - 16.0).max(16.0));
    }

    pub fn drop_context(
        &self,
        ht: &mut HitTestTreeContext,
        dpi_handlers: &mut Vec<std::rc::Weak<dyn DpiHandler>>,
    ) {
        dpi_handlers.retain(|e| !core::ptr::addr_eq(e.as_ptr(), Rc::as_ptr(&self.dpi_handler)));
        self.sprite_list.drop_context(ht);
        self.view.drop_context(ht);
    }
}

pub enum VerticalSplitterFixedSide {
    Left,
    Right,
}

pub struct VerticalSplitterView {
    root: ContainerVisual,
    overlay: SpriteVisual,
    enter_animation: ScalarKeyFrameAnimation,
    leave_animation: ScalarKeyFrameAnimation,
    ht_root: HitTestTreeRef,
    fixed_side: VerticalSplitterFixedSide,
    dpi: Cell<f32>,
    hoffs: Cell<f32>,
}
impl VerticalSplitterView {
    pub fn new(init: &mut ViewInitContext, fixed_side: VerticalSplitterFixedSide) -> Self {
        let root = init.subsystem.compositor.CreateContainerVisual().unwrap();
        root.SetSize(Vector2 {
            X: dip_to_pixels(8.0, init.dpi),
            Y: 0.0,
        })
        .unwrap();
        root.SetRelativeSizeAdjustment(Vector2 { X: 0.0, Y: 1.0 })
            .unwrap();
        root.SetRelativeOffsetAdjustment(Vector3 {
            X: match fixed_side {
                VerticalSplitterFixedSide::Left => 0.0,
                VerticalSplitterFixedSide::Right => 1.0,
            },
            Y: 0.0,
            Z: 0.0,
        })
        .unwrap();

        let line = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        line.SetBrush(
            &init
                .subsystem
                .compositor
                .CreateColorBrushWithColor(SEPARATOR_COLOR)
                .unwrap(),
        )
        .unwrap();
        line.SetSize(Vector2 {
            X: dip_to_pixels(1.0, init.dpi),
            Y: 0.0,
        })
        .unwrap();
        line.SetRelativeSizeAdjustment(Vector2 { X: 0.0, Y: 1.0 })
            .unwrap();
        line.SetOffset(Vector3 {
            X: -dip_to_pixels(0.5, init.dpi),
            Y: 0.0,
            Z: 0.0,
        })
        .unwrap();
        line.SetRelativeOffsetAdjustment(Vector3 {
            X: 0.5,
            Y: 0.0,
            Z: 0.0,
        })
        .unwrap();

        let overlay = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        overlay
            .SetBrush(
                &init
                    .subsystem
                    .compositor
                    .CreateColorBrushWithColor(Color {
                        A: 128,
                        ..SEPARATOR_COLOR
                    })
                    .unwrap(),
            )
            .unwrap();
        overlay.SetRelativeSizeAdjustment(Vector2::one()).unwrap();
        overlay.SetOpacity(0.0).unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&line).unwrap();
        children.InsertAtTop(&overlay).unwrap();

        let linear_easing = init
            .subsystem
            .compositor
            .CreateLinearEasingFunction()
            .unwrap();
        let enter_animation = init
            .subsystem
            .compositor
            .CreateScalarKeyFrameAnimation()
            .unwrap();
        enter_animation.SetDuration(timespan_ms(100)).unwrap();
        enter_animation.InsertKeyFrame(0.0, 0.0).unwrap();
        enter_animation
            .InsertKeyFrameWithEasingFunction(1.0, 1.0, &linear_easing)
            .unwrap();
        let leave_animation = init
            .subsystem
            .compositor
            .CreateScalarKeyFrameAnimation()
            .unwrap();
        leave_animation.SetDuration(timespan_ms(100)).unwrap();
        leave_animation.InsertKeyFrame(0.0, 1.0).unwrap();
        leave_animation
            .InsertKeyFrameWithEasingFunction(1.0, 0.0, &linear_easing)
            .unwrap();

        let ht_root = init.ht.alloc(HitTestTreeData {
            left: 0.0,
            top: 0.0,
            left_adjustment_factor: match fixed_side {
                VerticalSplitterFixedSide::Left => 0.0,
                VerticalSplitterFixedSide::Right => 1.0,
            },
            top_adjustment_factor: 0.0,
            width: 8.0,
            height: 0.0,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });

        Self {
            root,
            overlay,
            enter_animation,
            leave_animation,
            ht_root,
            fixed_side,
            dpi: Cell::new(init.dpi),
            hoffs: Cell::new(0.0),
        }
    }

    pub fn set_horizontal_offset(&self, ht: &mut HitTestTreeContext, offs: f32) {
        self.root
            .SetOffset(Vector3 {
                X: match self.fixed_side {
                    VerticalSplitterFixedSide::Left => dip_to_pixels(offs - 4.0, self.dpi.get()),
                    VerticalSplitterFixedSide::Right => dip_to_pixels(-offs - 4.0, self.dpi.get()),
                },
                Y: 0.0,
                Z: 0.0,
            })
            .unwrap();
        ht.get_mut(self.ht_root).left = match self.fixed_side {
            VerticalSplitterFixedSide::Left => offs - 4.0,
            VerticalSplitterFixedSide::Right => -offs - 4.0,
        };

        self.hoffs.set(offs);
    }

    pub fn set_dpi(&self, dpi: f32) {
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(8.0, dpi),
                Y: 0.0,
            })
            .unwrap();
        self.root
            .SetOffset(Vector3 {
                X: match self.fixed_side {
                    VerticalSplitterFixedSide::Left => dip_to_pixels(self.hoffs.get() - 4.0, dpi),
                    VerticalSplitterFixedSide::Right => dip_to_pixels(-self.hoffs.get() - 4.0, dpi),
                },
                Y: 0.0,
                Z: 0.0,
            })
            .unwrap();

        self.dpi.set(dpi);
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn cursor(&self) -> HCURSOR {
        // TODO: 必要ならキャッシュする
        unsafe { LoadCursorW(None, IDC_SIZEWE).unwrap() }
    }

    pub fn activate_hover_overlay(&self) {
        self.overlay
            .StartAnimation(h!("Opacity"), &self.enter_animation)
            .unwrap();
    }

    pub fn deactivate_hover_overlay(&self) {
        self.overlay
            .StartAnimation(h!("Opacity"), &self.leave_animation)
            .unwrap();
    }

    pub fn drop_context(&self, ht: &mut HitTestTreeContext) {
        ht.free_rec(self.ht_root);
    }
}

pub struct MainHitActionHandler {
    right_pane_content: Rc<RightPaneContentPresenter>,
    splitter_view: Rc<VerticalSplitterView>,
    drag_start_state: RefCell<Option<(f32, f32)>>,
}
impl HitTestTreeActionHandler for MainHitActionHandler {
    fn cursor(&self, sender: HitTestTreeRef) -> Option<HCURSOR> {
        if sender == self.splitter_view.ht_root {
            return Some(self.splitter_view.cursor());
        }

        None
    }

    fn on_pointer_enter(&self, sender: HitTestTreeRef) -> EventContinueControl {
        if sender == self.splitter_view.ht_root {
            self.splitter_view.activate_hover_overlay();

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_leave(&self, sender: HitTestTreeRef) -> EventContinueControl {
        if sender == self.splitter_view.ht_root {
            self.splitter_view.deactivate_hover_overlay();

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_down(
        &self,
        sender: HitTestTreeRef,
        _ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.splitter_view.ht_root {
            *self.drag_start_state.borrow_mut() = Some((client_x, self.right_pane_content.width()));
            return EventContinueControl::STOP_PROPAGATION | EventContinueControl::CAPTURE_ELEMENT;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_up(
        &self,
        sender: HitTestTreeRef,
        ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.splitter_view.ht_root {
            if let Some((base_x, base_width)) = self.drag_start_state.replace(None) {
                let new_width = 10.0f32.max(base_width + (base_x - client_x));
                self.right_pane_content.set_width(ht, new_width);
                self.splitter_view.set_horizontal_offset(ht, new_width);

                return EventContinueControl::STOP_PROPAGATION
                    | EventContinueControl::RELEASE_CAPTURE_ELEMENT;
            }
        }

        EventContinueControl::empty()
    }

    fn on_pointer_move(
        &self,
        sender: HitTestTreeRef,
        ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.splitter_view.ht_root {
            if let Some((base_x, base_width)) = *self.drag_start_state.borrow() {
                let new_width = 10.0f32.max(base_width + (base_x - client_x));
                self.right_pane_content.set_width(ht, new_width);
                self.splitter_view.set_horizontal_offset(ht, new_width);

                return EventContinueControl::STOP_PROPAGATION;
            }
        }

        EventContinueControl::empty()
    }
}

pub struct MainDpiHandler {
    splitter_view: Rc<VerticalSplitterView>,
}
impl DpiHandler for MainDpiHandler {
    fn on_dpi_changed(&self, new_dpi: f32) {
        self.splitter_view.set_dpi(new_dpi);
    }
}

pub struct MainPresenter {
    right_pane_content: Rc<RightPaneContentPresenter>,
    splitter_view: Rc<VerticalSplitterView>,
    _ht_action_handler: Rc<MainHitActionHandler>,
    dpi_handler: Rc<MainDpiHandler>,
}
impl MainPresenter {
    pub fn new(init: &mut PresenterInitContext, init_right_width: f32) -> Self {
        let splitter_view = Rc::new(VerticalSplitterView::new(
            &mut init.for_view,
            VerticalSplitterFixedSide::Right,
        ));
        let right_pane_content = Rc::new(RightPaneContentPresenter::new(init, init_right_width));

        splitter_view.set_horizontal_offset(&mut init.for_view.ht, init_right_width);

        let ht_action_handler = Rc::new(MainHitActionHandler {
            right_pane_content: right_pane_content.clone(),
            splitter_view: splitter_view.clone(),
            drag_start_state: RefCell::new(None),
        });
        init.for_view
            .ht
            .get_mut(splitter_view.ht_root)
            .action_handler = Some(Rc::downgrade(&ht_action_handler) as _);
        let dpi_handler = Rc::new(MainDpiHandler {
            splitter_view: splitter_view.clone(),
        });
        init.dpi_handlers.push(Rc::downgrade(&dpi_handler) as _);

        Self {
            right_pane_content,
            splitter_view,
            _ht_action_handler: ht_action_handler,
            dpi_handler,
        }
    }

    pub fn mount(
        &self,
        ht: &mut HitTestTreeContext,
        children: &VisualCollection,
        ht_parent: HitTestTreeRef,
    ) {
        self.right_pane_content.mount(ht, children, ht_parent);
        self.splitter_view.mount(ht, children, ht_parent);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.splitter_view.unmount(ht);
        self.right_pane_content.unmount(ht);
    }

    pub fn drop_context(
        &self,
        ht: &mut HitTestTreeContext,
        dpi_handlers: &mut Vec<std::rc::Weak<dyn DpiHandler>>,
    ) {
        dpi_handlers.retain(|e| !core::ptr::addr_eq(e.as_ptr(), Rc::as_ptr(&self.dpi_handler)));
        self.right_pane_content.drop_context(ht, dpi_handlers);
        self.splitter_view.drop_context(ht);
    }
}

#[repr(C, align(16))]
pub struct AtlasBaseGridRenderParams {
    pub pixel_size: [f32; 2],
    pub grid_size: f32,
}

pub struct AtlasBaseGridView {
    root: SpriteVisual,
    swapchain: IDXGISwapChain2,
    vsh: ID3D11VertexShader,
    psh: ID3D11PixelShader,
    render_params_cb: ID3D11Buffer,
    size_pixels: Cell<(u32, u32)>,
    resize_order: Cell<Option<(u32, u32)>>,
}
impl AtlasBaseGridView {
    pub fn new(
        init: &mut ViewInitContext,
        init_width_pixels: u32,
        init_height_pixels: u32,
    ) -> Self {
        let sc = unsafe {
            init.subsystem
                .d3d11_device
                .cast::<IDXGIDevice2>()
                .unwrap()
                .GetParent::<IDXGIAdapter>()
                .unwrap()
                .GetParent::<IDXGIFactory2>()
                .unwrap()
                .CreateSwapChainForComposition(
                    &init.subsystem.d3d11_device,
                    &DXGI_SWAP_CHAIN_DESC1 {
                        Width: init_width_pixels,
                        Height: init_height_pixels,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Stereo: BOOL(0),
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                        BufferCount: 3,
                        // Composition向けのSwapchainはStretchじゃないとだめらしい
                        Scaling: DXGI_SCALING_STRETCH,
                        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                        AlphaMode: DXGI_ALPHA_MODE_IGNORE,
                        Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as _,
                    },
                    None,
                )
                .unwrap()
        };
        let sc = sc.cast::<IDXGISwapChain2>().unwrap();
        let sc_surface = unsafe {
            init.subsystem
                .compositor_interop
                .CreateCompositionSurfaceForSwapChain(&sc)
                .unwrap()
        };

        let root = SpriteVisualParams {
            brush: &CompositionSurfaceBrushParams {
                surface: &sc_surface,
                stretch: None,
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
            offset: None,
            relative_offset_adjustment: None,
            size: Some(Vector2 {
                X: init_width_pixels as _,
                Y: init_height_pixels as _,
            }),
            relative_size_adjustment: None,
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let mut vsh = core::mem::MaybeUninit::uninit();
        let mut psh = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateVertexShader(
                    &std::fs::read("./resources/grid/vsh.fxc").unwrap(),
                    None,
                    Some(vsh.as_mut_ptr()),
                )
                .unwrap();
            init.subsystem
                .d3d11_device
                .CreatePixelShader(
                    &std::fs::read("./resources/grid/psh.fxc").unwrap(),
                    None,
                    Some(psh.as_mut_ptr()),
                )
                .unwrap();
        }
        let vsh = unsafe { vsh.assume_init().unwrap() };
        let psh = unsafe { psh.assume_init().unwrap() };

        let mut render_params_cb = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateBuffer(
                    &D3D11_BUFFER_DESC {
                        ByteWidth: core::mem::size_of::<AtlasBaseGridRenderParams>() as _,
                        Usage: D3D11_USAGE_DYNAMIC,
                        BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as _,
                        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as _,
                        MiscFlags: 0,
                        StructureByteStride: core::mem::size_of::<AtlasBaseGridRenderParams>() as _,
                    },
                    Some(&D3D11_SUBRESOURCE_DATA {
                        pSysMem: (&AtlasBaseGridRenderParams {
                            pixel_size: [init_width_pixels as _, init_height_pixels as _],
                            grid_size: 64.0,
                        }) as *const _ as _,
                        SysMemPitch: 0,
                        SysMemSlicePitch: 0,
                    }),
                    Some(render_params_cb.as_mut_ptr()),
                )
                .unwrap();
        }
        let render_params_cb = unsafe { render_params_cb.assume_init().unwrap() };

        let bb = unsafe { sc.GetBuffer::<ID3D11Texture2D>(0).unwrap() };
        let mut rtv = core::mem::MaybeUninit::uninit();
        unsafe {
            init.subsystem
                .d3d11_device
                .CreateRenderTargetView(
                    &bb,
                    Some(&D3D11_RENDER_TARGET_VIEW_DESC {
                        ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
                            Texture2D: D3D11_TEX2D_RTV { MipSlice: 0 },
                        },
                    }),
                    Some(rtv.as_mut_ptr()),
                )
                .unwrap()
        };
        let rtv = unsafe { rtv.assume_init().unwrap() };
        unsafe {
            init.subsystem
                .d3d11_imm_context
                .OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            init.subsystem
                .d3d11_imm_context
                .RSSetViewports(Some(&[D3D11_VIEWPORT {
                    TopLeftX: 0.0,
                    TopLeftY: 0.0,
                    Width: init_width_pixels as _,
                    Height: init_height_pixels as _,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                }]));
            init.subsystem
                .d3d11_imm_context
                .RSSetScissorRects(Some(&[RECT {
                    left: 0,
                    top: 0,
                    right: init_width_pixels as _,
                    bottom: init_height_pixels as _,
                }]));
            init.subsystem.d3d11_imm_context.VSSetShader(&vsh, None);
            init.subsystem.d3d11_imm_context.PSSetShader(&psh, None);
            init.subsystem
                .d3d11_imm_context
                .PSSetConstantBuffers(0, Some(&[Some(render_params_cb.clone())]));
            init.subsystem
                .d3d11_imm_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            init.subsystem.d3d11_imm_context.Draw(4, 0);
            init.subsystem.d3d11_imm_context.PSSetShader(None, None);
            init.subsystem.d3d11_imm_context.VSSetShader(None, None);
            init.subsystem.d3d11_imm_context.Flush();
        }

        unsafe {
            sc.Present(0, DXGI_PRESENT(0)).ok().unwrap();
        }

        Self {
            root,
            swapchain: sc,
            vsh,
            psh,
            render_params_cb,
            size_pixels: Cell::new((init_width_pixels, init_height_pixels)),
            resize_order: Cell::new(None),
        }
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

    pub fn resize(&self, new_width_px: u32, new_height_px: u32) {
        self.root
            .SetSize(Vector2 {
                X: new_width_px as _,
                Y: new_height_px as _,
            })
            .unwrap();

        self.resize_order.set(Some((new_width_px, new_height_px)));
    }

    pub fn update_content(&self, subsystem: &Subsystem) {
        if let Some((req_width_px, req_height_px)) = self.resize_order.replace(None) {
            unsafe {
                self.swapchain
                    .ResizeBuffers(
                        3,
                        req_width_px,
                        req_height_px,
                        DXGI_FORMAT_B8G8R8A8_UNORM,
                        DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
                    )
                    .unwrap();
            }

            self.size_pixels.set((req_width_px, req_height_px));
        }

        let (width_px, height_px) = self.size_pixels.get();

        let mut mapped = core::mem::MaybeUninit::uninit();
        unsafe {
            subsystem
                .d3d11_imm_context
                .Map(
                    &self.render_params_cb,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    Some(mapped.as_mut_ptr()),
                )
                .unwrap();
        }
        let mapped = unsafe { mapped.assume_init() };
        unsafe {
            core::ptr::write(
                mapped.pData as _,
                AtlasBaseGridRenderParams {
                    pixel_size: [width_px as _, height_px as _],
                    grid_size: 64.0,
                },
            );
        }
        unsafe {
            subsystem.d3d11_imm_context.Unmap(&self.render_params_cb, 0);
        }

        let bb = unsafe { self.swapchain.GetBuffer::<ID3D11Texture2D>(0).unwrap() };
        let mut rtv = core::mem::MaybeUninit::uninit();
        unsafe {
            subsystem
                .d3d11_device
                .CreateRenderTargetView(
                    &bb,
                    Some(&D3D11_RENDER_TARGET_VIEW_DESC {
                        ViewDimension: D3D11_RTV_DIMENSION_TEXTURE2D,
                        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        Anonymous: D3D11_RENDER_TARGET_VIEW_DESC_0 {
                            Texture2D: D3D11_TEX2D_RTV { MipSlice: 0 },
                        },
                    }),
                    Some(rtv.as_mut_ptr()),
                )
                .unwrap()
        };
        let rtv = unsafe { rtv.assume_init().unwrap() };
        unsafe {
            subsystem
                .d3d11_imm_context
                .OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            subsystem
                .d3d11_imm_context
                .RSSetViewports(Some(&[D3D11_VIEWPORT {
                    TopLeftX: 0.0,
                    TopLeftY: 0.0,
                    Width: width_px as _,
                    Height: height_px as _,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                }]));
            subsystem.d3d11_imm_context.RSSetScissorRects(Some(&[RECT {
                left: 0,
                top: 0,
                right: width_px as _,
                bottom: height_px as _,
            }]));
            subsystem.d3d11_imm_context.VSSetShader(&self.vsh, None);
            subsystem.d3d11_imm_context.PSSetShader(&self.psh, None);
            subsystem
                .d3d11_imm_context
                .PSSetConstantBuffers(0, Some(&[Some(self.render_params_cb.clone())]));
            subsystem
                .d3d11_imm_context
                .IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
            subsystem.d3d11_imm_context.Draw(4, 0);
            subsystem.d3d11_imm_context.PSSetShader(None, None);
            subsystem.d3d11_imm_context.VSSetShader(None, None);
            subsystem.d3d11_imm_context.Flush();
        }

        unsafe {
            self.swapchain.Present(0, DXGI_PRESENT(0)).ok().unwrap();
        }
    }
}

#[inline]
fn create_instant_effect_brush(
    subsystem: &Subsystem,
    effect: impl windows::core::Param<IGraphicsEffect>,
    source_params: &[(&HSTRING, CompositionBrush)],
) -> windows::core::Result<CompositionEffectBrush> {
    let x = subsystem
        .compositor
        .CreateEffectFactory(effect)?
        .CreateBrush()?;
    for (n, s) in source_params {
        x.SetSourceParameter(n, s)?;
    }

    Ok(x)
}

const D2D1_COLOR_F_WHITE: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub struct SpriteListPaneView {
    root: ContainerVisual,
    ht_root: HitTestTreeRef,
    ht_adjust_area: HitTestTreeRef,
    dpi: f32,
    top: Cell<f32>,
    width: Cell<f32>,
}
impl SpriteListPaneView {
    const CORNER_RADIUS: f32 = 12.0;
    const FRAME_TEX_SIZE: f32 = 32.0;
    const BLUR_AMOUNT: f32 = 24.0;
    const SURFACE_COLOR: windows::UI::Color = windows::UI::Color {
        R: 255,
        G: 255,
        B: 255,
        A: 128,
    };
    const SPACING: f32 = 16.0;
    const ADJUST_AREA_THICKNESS: f32 = 4.0;

    fn gen_frame_tex(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let s = subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
                    Height: dip_to_pixels(Self::FRAME_TEX_SIZE, dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
            .unwrap();
        let interop: ICompositionDrawingSurfaceInterop = s.cast().unwrap();
        let mut offs = core::mem::MaybeUninit::uninit();
        let dc: ID2D1DeviceContext = unsafe { interop.BeginDraw(None, offs.as_mut_ptr()).unwrap() };
        let offs = unsafe { offs.assume_init() };
        let r = 'drawing: {
            let brush = scoped_try!('drawing, unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F_WHITE, None) });

            let offs_dip = D2D_POINT_2F {
                x: signed_pixels_to_dip(offs.x, dpi),
                y: signed_pixels_to_dip(offs.y, dpi),
            };

            unsafe {
                dc.SetDpi(dpi, dpi);
                dc.Clear(None);
                dc.FillRoundedRectangle(
                    &D2D1_ROUNDED_RECT {
                        rect: D2D_RECT_F {
                            left: offs_dip.x,
                            top: offs_dip.y,
                            right: offs_dip.x + Self::FRAME_TEX_SIZE,
                            bottom: offs_dip.y + Self::FRAME_TEX_SIZE,
                        },
                        radiusX: Self::CORNER_RADIUS,
                        radiusY: Self::CORNER_RADIUS,
                    },
                    &brush,
                );
            }

            Ok(())
        };
        unsafe {
            interop.EndDraw().unwrap();
        }
        r.unwrap();

        s
    }

    pub fn new(init: &mut ViewInitContext) -> Self {
        let frame_surface = Self::gen_frame_tex(init.subsystem, init.dpi);

        let root = ContainerVisualParams {
            offset: Some(Vector3 {
                X: dip_to_pixels(8.0, init.dpi),
                Y: 0.0,
                Z: 0.0,
            }),
            size: Some(Vector2 { X: 192.0, Y: 0.0 }),
            relative_size_adjustment: Some(Vector2 { X: 0.0, Y: 1.0 }),
            ..Default::default()
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let bg_base_brush = create_instant_effect_brush(
            init.subsystem,
            &CompositeEffectParams {
                sources: &[
                    GaussianBlurEffectParams {
                        source: &CompositionEffectSourceParameter::Create(h!("source")).unwrap(),
                        blur_amount: Some(Self::BLUR_AMOUNT / 3.0),
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
                mode: None,
            }
            .instantiate()
            .unwrap(),
            &[(
                h!("source"),
                init.subsystem
                    .compositor
                    .CreateBackdropBrush()
                    .unwrap()
                    .cast()
                    .unwrap(),
            )],
        )
        .unwrap();

        let bg = SpriteVisualParams {
            brush: &CompositionMaskBrushParams {
                source: &bg_base_brush,
                mask: &CompositionNineGridBrushParams {
                    source: &CompositionSurfaceBrushParams {
                        surface: &frame_surface,
                        stretch: Some(CompositionStretch::Fill),
                    }
                    .instantiate(&init.subsystem.compositor)
                    .unwrap(),
                    insets: Some(dip_to_pixels(Self::CORNER_RADIUS, init.dpi)),
                }
                .instantiate(&init.subsystem.compositor)
                .unwrap(),
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: Some(Vector2::one()),
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let tl = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &"Sprites".encode_utf16().collect::<Vec<_>>(),
                    &init.subsystem.default_ui_format,
                    f32::MAX,
                    f32::MAX,
                )
                .unwrap()
        };
        unsafe {
            tl.SetFontWeight(
                DWRITE_FONT_WEIGHT_MEDIUM,
                DWRITE_TEXT_RANGE {
                    startPosition: 0,
                    length: 7,
                },
            )
            .unwrap();
            tl.cast::<IDWriteTextLayout1>()
                .unwrap()
                .SetCharacterSpacing(
                    0.2,
                    0.2,
                    0.1,
                    DWRITE_TEXT_RANGE {
                        startPosition: 0,
                        length: 7,
                    },
                )
                .unwrap();
        }
        let mut tm = core::mem::MaybeUninit::uninit();
        unsafe {
            tl.GetMetrics(tm.as_mut_ptr()).unwrap();
        }
        let tm = unsafe { tm.assume_init() };
        let header_surface = init
            .subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(tm.width, init.dpi),
                    Height: dip_to_pixels(tm.height, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
            .unwrap();
        {
            let interop = header_surface
                .cast::<ICompositionDrawingSurfaceInterop>()
                .unwrap();
            let mut offset = core::mem::MaybeUninit::uninit();
            let dc: ID2D1DeviceContext =
                unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
            let offset = unsafe { offset.assume_init() };
            let r = 'drawing: {
                let brush = scoped_try!(
                    'drawing,
                    unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }, None) }
                );

                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: signed_pixels_to_dip(offset.x, init.dpi),
                            y: signed_pixels_to_dip(offset.y, init.dpi),
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

        let header = SpriteVisualParams {
            brush: &CompositionSurfaceBrushParams {
                surface: &header_surface,
                stretch: None,
            }
            .instantiate(&init.subsystem.compositor)
            .unwrap(),
            offset: Some(Vector3 {
                X: dip_to_pixels(-tm.width * 0.5, init.dpi),
                Y: dip_to_pixels(Self::CORNER_RADIUS, init.dpi),
                Z: 0.0,
            }),
            relative_offset_adjustment: Some(Vector3 {
                X: 0.5,
                Y: 0.0,
                Z: 0.0,
            }),
            size: Some(Vector2 {
                X: dip_to_pixels(tm.width, init.dpi),
                Y: dip_to_pixels(tm.height, init.dpi),
            }),
            relative_size_adjustment: None,
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let children = root.Children().unwrap();
        children.InsertAtTop(&bg).unwrap();
        children.InsertAtTop(&header).unwrap();

        let ht_root = init.ht.alloc(HitTestTreeData {
            left: Self::SPACING,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 192.0,
            height: -Self::SPACING,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });
        let ht_adjust_area = init.ht.alloc(HitTestTreeData {
            left: -Self::ADJUST_AREA_THICKNESS * 0.5,
            top: 0.0,
            left_adjustment_factor: 1.0,
            top_adjustment_factor: 0.0,
            width: Self::ADJUST_AREA_THICKNESS,
            height: 0.0,
            width_adjustment_factor: 0.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: Vec::new(),
            action_handler: None,
        });
        init.ht.add_child(ht_root, ht_adjust_area);

        Self {
            root,
            ht_root,
            ht_adjust_area,
            dpi: init.dpi,
            top: Cell::new(0.0),
            width: Cell::new(192.0),
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut HitTestTreeContext,
        ht_parent: HitTestTreeRef,
    ) {
        children.InsertAtTop(&self.root).unwrap();
        ht.add_child(ht_parent, self.ht_root);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
        ht.remove_child(self.ht_root);
    }

    pub fn shutdown(&self, ht: &mut HitTestTreeContext) {
        ht.free_rec(self.ht_root);
    }

    pub fn set_top(&self, ht: &mut HitTestTreeContext, top: f32) {
        self.root
            .SetOffset(Vector3 {
                X: dip_to_pixels(Self::SPACING, self.dpi),
                Y: dip_to_pixels(top, self.dpi),
                Z: 0.0,
            })
            .unwrap();
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(self.width.get(), self.dpi),
                Y: dip_to_pixels(-top - Self::SPACING, self.dpi),
            })
            .unwrap();
        ht.get_mut(self.ht_root).top = top;
        ht.get_mut(self.ht_root).height = -top - Self::SPACING;

        self.top.set(top);
    }

    pub fn set_width(&self, ht: &mut HitTestTreeContext, width: f32) {
        self.root
            .SetSize(Vector2 {
                X: dip_to_pixels(width, self.dpi),
                Y: dip_to_pixels(-self.top.get() - Self::SPACING, self.dpi),
            })
            .unwrap();
        ht.get_mut(self.ht_root).width = width;

        self.width.set(width);
    }
}

pub struct SpriteListPaneHitActionHandler {
    pub view: Rc<SpriteListPaneView>,
    adjust_drag_state: Cell<Option<(f32, f32)>>,
}
impl HitTestTreeActionHandler for SpriteListPaneHitActionHandler {
    fn cursor(&self, sender: HitTestTreeRef) -> Option<HCURSOR> {
        if sender == self.view.ht_adjust_area {
            // TODO: 必要そうならキャッシュする
            return Some(unsafe { LoadCursorW(None, IDC_SIZEWE).unwrap() });
        }

        None
    }

    fn on_pointer_down(
        &self,
        sender: HitTestTreeRef,
        _ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_adjust_area {
            self.adjust_drag_state
                .set(Some((client_x, self.view.width.get())));

            return EventContinueControl::CAPTURE_ELEMENT | EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_move(
        &self,
        sender: HitTestTreeRef,
        ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_adjust_area {
            if let Some((base_x, base_width)) = self.adjust_drag_state.get() {
                let new_width = (base_width + (client_x - base_x)).max(10.0);
                self.view.set_width(ht, new_width);
            }

            return EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }

    fn on_pointer_up(
        &self,
        sender: HitTestTreeRef,
        ht: &mut HitTestTreeContext,
        client_x: f32,
        _client_y: f32,
    ) -> EventContinueControl {
        if sender == self.view.ht_adjust_area {
            if let Some((base_x, base_width)) = self.adjust_drag_state.replace(None) {
                let new_width = (base_width + (client_x - base_x)).max(10.0);
                self.view.set_width(ht, new_width);
            }

            return EventContinueControl::RELEASE_CAPTURE_ELEMENT
                | EventContinueControl::STOP_PROPAGATION;
        }

        EventContinueControl::empty()
    }
}

pub struct SpriteListPanePresenter {
    view: Rc<SpriteListPaneView>,
    _ht_action_handler: Rc<SpriteListPaneHitActionHandler>,
}
impl SpriteListPanePresenter {
    pub fn new(init: &mut PresenterInitContext) -> Self {
        let view = Rc::new(SpriteListPaneView::new(&mut init.for_view));

        let ht_action_handler = Rc::new(SpriteListPaneHitActionHandler {
            view: view.clone(),
            adjust_drag_state: Cell::new(None),
        });
        init.for_view.ht.get_mut(view.ht_adjust_area).action_handler =
            Some(Rc::downgrade(&ht_action_handler) as _);

        Self {
            view,
            _ht_action_handler: ht_action_handler,
        }
    }

    pub fn mount(
        &self,
        children: &VisualCollection,
        ht: &mut HitTestTreeContext,
        ht_parent: HitTestTreeRef,
    ) {
        self.view.mount(children, ht, ht_parent);
    }

    pub fn unmount(&self, ht: &mut HitTestTreeContext) {
        self.view.unmount(ht);
    }

    pub fn shutdown(&self, ht: &mut HitTestTreeContext) {
        self.view.shutdown(ht);
    }

    pub fn set_top(&self, ht: &mut HitTestTreeContext, top: f32) {
        self.view.set_top(ht, top);
    }
}

pub struct AppCloseButtonView {
    root: ContainerVisual,
}
impl AppCloseButtonView {
    const BUTTON_SIZE: f32 = 24.0;
    const ICON_SIZE: f32 = 6.0;

    fn build_icon_surface(subsystem: &Subsystem, dpi: f32) -> CompositionDrawingSurface {
        let icon_surface = subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(Self::ICON_SIZE, dpi),
                    Height: dip_to_pixels(Self::ICON_SIZE, dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
            .unwrap();

        let interop = icon_surface
            .cast::<ICompositionDrawingSurfaceInterop>()
            .unwrap();
        let mut offset = core::mem::MaybeUninit::uninit();
        let dc: ID2D1DeviceContext =
            unsafe { interop.BeginDraw(None, offset.as_mut_ptr()).unwrap() };
        let offset = unsafe { offset.assume_init() };
        let r = 'drawing: {
            let brush = scoped_try!(
                'drawing,
                unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0, }, None) }
            );

            let offset_x_dip = signed_pixels_to_dip(offset.x, dpi);
            let offset_y_dip = signed_pixels_to_dip(offset.y, dpi);

            unsafe {
                dc.SetDpi(dpi, dpi);
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

        icon_surface
    }

    pub fn new(init: &mut ViewInitContext) -> Self {
        let icon_surface = Self::build_icon_surface(init.subsystem, init.dpi);

        let circle_mask_surface = init
            .subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
                    Height: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
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

                let offset_x_dip = signed_pixels_to_dip(offset.x, init.dpi);
                let offset_y_dip = signed_pixels_to_dip(offset.y, init.dpi);

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

        let root = ContainerVisualParams {
            size: Some(Vector2 {
                X: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
                Y: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
            }),
            ..Default::default()
        }
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
                        color: Some(windows::UI::Color {
                            A: 128,
                            R: 255,
                            G: 255,
                            B: 255,
                        }),
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

        let bg = SpriteVisualParams {
            brush: &CompositionMaskBrushParams {
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
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: Some(Vector2::one()),
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let icon = SpriteVisualParams {
            brush: &init
                .subsystem
                .compositor
                .CreateSurfaceBrushWithSurface(&icon_surface)
                .unwrap(),
            offset: Some(Vector3 {
                X: dip_to_pixels(-Self::ICON_SIZE * 0.5, init.dpi),
                Y: dip_to_pixels(-Self::ICON_SIZE * 0.5, init.dpi),
                Z: 0.0,
            }),
            relative_offset_adjustment: Some(Vector3 {
                X: 0.5,
                Y: 0.5,
                Z: 0.0,
            }),
            size: Some(Vector2 {
                X: dip_to_pixels(Self::ICON_SIZE, init.dpi),
                Y: dip_to_pixels(Self::ICON_SIZE, init.dpi),
            }),
            relative_size_adjustment: None,
        }
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

    pub fn unmount(&self) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
            .unwrap();
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
        let icon_surface = init
            .subsystem
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(Self::ICON_SIZE, init.dpi),
                    Height: dip_to_pixels(Self::ICON_SIZE, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
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
                let brush = scoped_try!(
                    'drawing,
                    unsafe { dc.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.1, g: 0.1, b: 0.1, a: 1.0, }, None) }
                );

                let offset_x_dip = signed_pixels_to_dip(offset.x, init.dpi);
                let offset_y_dip = signed_pixels_to_dip(offset.y, init.dpi);

                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
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
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
                    Height: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
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

                let offset_x_dip = signed_pixels_to_dip(offset.x, init.dpi);
                let offset_y_dip = signed_pixels_to_dip(offset.y, init.dpi);

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

        let root = ContainerVisualParams {
            size: Some(Vector2 {
                X: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
                Y: dip_to_pixels(Self::BUTTON_SIZE, init.dpi),
            }),
            ..Default::default()
        }
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

        let bg = SpriteVisualParams {
            brush: &CompositionMaskBrushParams {
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
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: Some(Vector2::one()),
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let icon = SpriteVisualParams {
            brush: &init
                .subsystem
                .compositor
                .CreateSurfaceBrushWithSurface(&icon_surface)
                .unwrap(),
            offset: Some(Vector3 {
                X: dip_to_pixels(-Self::ICON_SIZE * 0.5, init.dpi),
                Y: dip_to_pixels(-Self::ICON_SIZE * 0.5, init.dpi),
                Z: 0.0,
            }),
            relative_offset_adjustment: Some(Vector3 {
                X: 0.5,
                Y: 0.5,
                Z: 0.0,
            }),
            size: Some(Vector2 {
                X: dip_to_pixels(Self::ICON_SIZE, init.dpi),
                Y: dip_to_pixels(Self::ICON_SIZE, init.dpi),
            }),
            relative_size_adjustment: None,
        }
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

    pub fn unmount(&self) {
        self.root
            .Parent()
            .unwrap()
            .Children()
            .unwrap()
            .Remove(&self.root)
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
            .composition_2d_graphics_device
            .CreateDrawingSurface(
                Size {
                    Width: dip_to_pixels(tm.width, init.dpi),
                    Height: dip_to_pixels(tm.height, init.dpi),
                },
                DirectXPixelFormat::B8G8R8A8UIntNormalized,
                DirectXAlphaMode::Premultiplied,
            )
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
                let brush = match unsafe {
                    dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        },
                        None,
                    )
                } {
                    Ok(x) => x,
                    Err(e) => break 'drawing Err(e),
                };

                unsafe {
                    dc.SetDpi(init.dpi, init.dpi);
                    dc.Clear(None);
                    dc.DrawTextLayout(
                        D2D_POINT_2F {
                            x: signed_pixels_to_dip(offset.x, init.dpi),
                            y: signed_pixels_to_dip(offset.y, init.dpi),
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
        let root = ContainerVisualParams {
            size: Some(Vector2 {
                X: 0.0,
                Y: dip_to_pixels(height, init.dpi),
            }),
            relative_size_adjustment: Some(Vector2 { X: 1.0, Y: 0.0 }),
            ..Default::default()
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let bg_brush = init
            .subsystem
            .compositor
            .CreateLinearGradientBrush()
            .unwrap();
        bg_brush
            .ColorStops()
            .unwrap()
            .Append(
                &init
                    .subsystem
                    .compositor
                    .CreateColorGradientStopWithOffsetAndColor(
                        0.0,
                        Color {
                            A: 128,
                            R: 0,
                            G: 0,
                            B: 0,
                        },
                    )
                    .unwrap(),
            )
            .unwrap();
        bg_brush
            .ColorStops()
            .unwrap()
            .Append(
                &init
                    .subsystem
                    .compositor
                    .CreateColorGradientStopWithOffsetAndColor(
                        1.0,
                        Color {
                            A: 32,
                            R: 0,
                            G: 0,
                            B: 0,
                        },
                    )
                    .unwrap(),
            )
            .unwrap();
        bg_brush.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 }).unwrap();
        bg_brush.SetEndPoint(Vector2 { X: 0.0, Y: 1.0 }).unwrap();
        let bg = SpriteVisualParams {
            brush: &bg_brush,
            offset: None,
            relative_offset_adjustment: None,
            size: None,
            relative_size_adjustment: Some(Vector2::one()),
        }
        .instantiate(&init.subsystem.compositor)
        .unwrap();

        let label = init.subsystem.compositor.CreateSpriteVisual().unwrap();
        label
            .SetBrush(
                &init
                    .subsystem
                    .compositor
                    .CreateSurfaceBrushWithSurface(&label_surface)
                    .unwrap(),
            )
            .unwrap();
        label
            .SetSize(Vector2 {
                X: dip_to_pixels(tm.width, init.dpi),
                Y: dip_to_pixels(tm.height, init.dpi),
            })
            .unwrap();
        label
            .SetOffset(Vector3 {
                X: dip_to_pixels(24.0, init.dpi),
                Y: dip_to_pixels(16.0, init.dpi),
                Z: 0.0,
            })
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
                X: dip_to_pixels(-spc - AppCloseButtonView::BUTTON_SIZE, init.dpi),
                Y: dip_to_pixels(spc, init.dpi),
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
                X: dip_to_pixels(
                    close_button_rect_rel.left - (6.0 + AppMinimizeButtonView::BUTTON_SIZE),
                    init.dpi,
                ),
                Y: dip_to_pixels(spc, init.dpi),
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

struct AppWindowStateModel {
    ht: HitTestTreeContext,
    ht_root: HitTestTreeRef,
    client_size_pixels: SizePixels,
    dpi: f32,
    dpi_handlers: Vec<std::rc::Weak<dyn DpiHandler>>,
    pointer_input_manager: PointerInputManager,
    composition_target: DesktopWindowTarget,
    composition_root: ContainerVisual,
    header_view: AppHeaderView,
    grid_view: AtlasBaseGridView,
    sprite_list_pane: SpriteListPanePresenter,
}
impl AppWindowStateModel {
    pub fn new(subsystem: &Subsystem, bound_hwnd: HWND) -> Self {
        let mut ht = HitTestTreeContext::new();
        let ht_root = ht.alloc(HitTestTreeData {
            left: 0.0,
            top: 0.0,
            left_adjustment_factor: 0.0,
            top_adjustment_factor: 0.0,
            width: 0.0,
            height: 0.0,
            width_adjustment_factor: 1.0,
            height_adjustment_factor: 1.0,
            parent: None,
            children: vec![],
            action_handler: None,
        });
        let mut client_size_pixels = core::mem::MaybeUninit::uninit();
        unsafe {
            GetClientRect(bound_hwnd, client_size_pixels.as_mut_ptr()).unwrap();
        }
        let client_size_pixels = unsafe { client_size_pixels.assume_init_ref() };
        let client_size_pixels = SizePixels {
            width: (client_size_pixels.right - client_size_pixels.left)
                .try_into()
                .expect("negative size?"),
            height: (client_size_pixels.bottom - client_size_pixels.top)
                .try_into()
                .expect("negative size?"),
        };
        let dpi = unsafe { GetDpiForWindow(bound_hwnd) as f32 };
        println!("init dpi: {dpi}");
        let mut dpi_handlers = Vec::new();
        let pointer_input_manager = PointerInputManager::new();

        let composition_target = unsafe {
            subsystem
                .compositor_desktop_interop
                .CreateDesktopWindowTarget(bound_hwnd, true)
                .unwrap()
        };

        let composition_root = subsystem.compositor.CreateContainerVisual().unwrap();
        composition_root
            .SetRelativeSizeAdjustment(Vector2::one())
            .unwrap();
        composition_target.SetRoot(&composition_root).unwrap();

        let bg = subsystem
            .compositor
            .CreateSpriteVisual()
            .expect("Failed to create bg visual");
        bg.SetBrush(
            &subsystem
                .compositor
                .CreateColorBrushWithColor(BG_COLOR)
                .expect("Failed to create bg brush"),
        )
        .expect("Failed to set bg brush");
        bg.SetRelativeSizeAdjustment(Vector2::one())
            .expect("Failed to set bg size");
        composition_root
            .Children()
            .expect("Failed to get children")
            .InsertAtBottom(&bg)
            .expect("Failed to insert bg");

        let mut presenter_init_context = PresenterInitContext {
            for_view: ViewInitContext {
                subsystem,
                ht: &mut ht,
                dpi,
            },
            dpi_handlers: &mut dpi_handlers,
        };

        let grid_view = AtlasBaseGridView::new(&mut presenter_init_context.for_view, 128, 128);
        grid_view.mount(&composition_root.Children().unwrap());
        grid_view.resize(client_size_pixels.width, client_size_pixels.height);

        let sprite_list_pane = SpriteListPanePresenter::new(&mut presenter_init_context);
        sprite_list_pane.mount(
            &composition_root.Children().unwrap(),
            &mut presenter_init_context.for_view.ht,
            ht_root,
        );

        let header_view = AppHeaderView::new(
            &mut presenter_init_context.for_view,
            "Peridot SpriteAtlas Visualizer/Editor",
        );
        header_view.mount(&composition_root.Children().unwrap());

        sprite_list_pane.set_top(&mut ht, header_view.height);

        ht.dump(ht_root);

        Self {
            ht,
            ht_root,
            client_size_pixels,
            dpi,
            dpi_handlers,
            pointer_input_manager,
            composition_target,
            composition_root,
            header_view,
            grid_view,
            sprite_list_pane,
        }
    }

    pub fn shutdown(&mut self) {
        self.sprite_list_pane.shutdown(&mut self.ht);
    }

    pub fn client_size_dip(&self) -> Size {
        self.client_size_pixels.to_dip(self.dpi)
    }

    pub fn nc_hittest(&self, hwnd: HWND, _wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
        let resize_h = unsafe { GetSystemMetrics(SM_CYSIZEFRAME) };

        let (x, y) = (
            (lparam.0 & 0xffff) as i16 as i32,
            ((lparam.0 >> 16) & 0xffff) as i16 as i32,
        );
        let mut p = [POINT { x, y }];
        unsafe {
            MapWindowPoints(None, Some(hwnd), &mut p);
        }
        let [POINT { x, y }] = p;

        if 0 > x
            || x > self.client_size_pixels.width as i32
            || 0 > y
            || y > self.client_size_pixels.height as i32
        {
            // ウィンドウ範囲外はシステムにおまかせ
            return None;
        }

        if y < resize_h {
            // global override
            return Some(LRESULT(HTTOP as _));
        }

        let p = PointDIP {
            x: signed_pixels_to_dip(x, self.dpi),
            y: signed_pixels_to_dip(y, self.dpi),
        };
        let size = self.client_size_dip();

        if let Some(ht) = self.header_view.nc_hittest(&p, &size) {
            return Some(LRESULT(ht as _));
        }

        Some(LRESULT(HTCLIENT as _))
    }

    pub fn on_dpi_changed(&mut self, new_dpi: u16) {
        self.dpi = new_dpi as _;
        for x in self.dpi_handlers.iter() {
            if let Some(x) = x.upgrade() {
                x.on_dpi_changed(new_dpi as _);
            }
        }
    }

    pub fn resize(&mut self, new_width: u16, new_height: u16) {
        self.client_size_pixels.width = new_width as _;
        self.client_size_pixels.height = new_height as _;

        self.grid_view.resize(
            self.client_size_pixels.width,
            self.client_size_pixels.height,
        );
    }

    pub fn on_mouse_move(&mut self, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_move(
            &mut self.ht,
            self.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );

        // WM_SETCURSORが飛ばないことがあるのでここで設定する
        if let Some(c) = self.pointer_input_manager.cursor(&self.ht) {
            unsafe {
                SetCursor(Some(c));
            }
        }
    }

    pub fn on_mouse_left_down(&mut self, hwnd: HWND, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_left_down(
            hwnd,
            &mut self.ht,
            self.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );
    }

    pub fn on_mouse_left_up(&mut self, hwnd: HWND, x_pixels: i16, y_pixels: i16) {
        self.pointer_input_manager.on_mouse_left_up(
            hwnd,
            &mut self.ht,
            self.ht_root,
            self.client_size_pixels.to_dip(self.dpi),
            signed_pixels_to_dip(x_pixels as _, self.dpi),
            signed_pixels_to_dip(y_pixels as _, self.dpi),
        );
    }

    pub fn handle_set_cursor(&mut self) -> bool {
        if let Some(c) = self.pointer_input_manager.cursor(&self.ht) {
            unsafe {
                SetCursor(Some(c));
            }

            return true;
        }

        false
    }
}

fn main() {
    let _ = AppRuntime::init().expect("Failed to initialize app runtime");

    let _dispatcher_queue_controller = unsafe {
        CreateDispatcherQueueController(DispatcherQueueOptions {
            dwSize: core::mem::size_of::<DispatcherQueueOptions>() as _,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_ASTA,
        })
        .expect("Failed to create dispatcher queue")
    };
    let subsystem = Subsystem::new();

    let cls = WNDCLASSEXW {
        cbSize: core::mem::size_of::<WNDCLASSEXW>() as _,
        style: WNDCLASS_STYLES(0),
        lpfnWndProc: Some(wndproc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe {
            core::mem::transmute(GetModuleHandleW(None).expect("Failed to current module handle"))
        },
        hIcon: unsafe { LoadIconW(None, IDI_APPLICATION).expect("Failed to load default icon") },
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).expect("Failed to load default cursor") },
        hbrBackground: HBRUSH(core::ptr::null_mut()),
        lpszMenuName: PCWSTR::null(),
        lpszClassName: w!("AppWindow"),
        hIconSm: unsafe { LoadIconW(None, IDI_APPLICATION).expect("Failed to load default icon") },
    };
    let ca = unsafe { RegisterClassExW(&cls) };
    if ca == 0 {
        panic!(
            "RegisterClassEx failed: {:?}",
            windows::core::Error::from_win32()
        );
    }

    let hw = unsafe {
        CreateWindowExW(
            WS_EX_APPWINDOW | WS_EX_OVERLAPPEDWINDOW | WS_EX_NOREDIRECTIONBITMAP,
            PCWSTR(ca as _),
            w!("Peridot SpriteAtlas Visualizer/Editor"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            None,
            None,
            Some(cls.hInstance),
            None,
        )
        .expect("Failed to create app window")
    };
    unsafe {
        // set dark mode preference
        DwmSetWindowAttribute(
            hw,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &BOOL(true as _) as *const _ as _,
            core::mem::size_of::<BOOL>() as _,
        )
        .expect("Failed to set dark mode preference");
    }

    let mut app_window_state_model = AppWindowStateModel::new(&subsystem, hw);

    let grid_view_render_waits = unsafe {
        app_window_state_model
            .grid_view
            .swapchain
            .GetFrameLatencyWaitableObject()
    };

    unsafe {
        SetWindowLongPtrW(
            hw,
            GWLP_USERDATA,
            &mut app_window_state_model as *mut _ as _,
        );
    }

    unsafe {
        let _ = ShowWindow(hw, SW_SHOW);
    }

    let mut msg = core::mem::MaybeUninit::uninit();
    'app: loop {
        let r = unsafe {
            MsgWaitForMultipleObjects(
                Some(&[grid_view_render_waits]),
                false,
                INFINITE,
                QS_ALLINPUT,
            )
        };

        if r == WAIT_OBJECT_0 {
            // update grid view
            app_window_state_model.grid_view.update_content(&subsystem);
            continue;
        }
        if r.0 == WAIT_OBJECT_0.0 + 1 {
            while unsafe { PeekMessageW(msg.as_mut_ptr(), None, 0, 0, PM_REMOVE).as_bool() } {
                if unsafe { msg.assume_init_ref().message == WM_QUIT } {
                    break 'app;
                }

                unsafe {
                    let _ = TranslateMessage(msg.as_ptr());
                    DispatchMessageW(msg.as_ptr());
                }
            }

            continue;
        }

        unreachable!();
    }

    unsafe {
        SetWindowLongPtrW(hw, GWLP_USERDATA, 0);
    }
    app_window_state_model.shutdown();
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CREATE {
        // notify frame changed
        let mut rc = core::mem::MaybeUninit::uninit();
        unsafe {
            GetWindowRect(hwnd, rc.as_mut_ptr()).unwrap();
        }
        let rc = unsafe { rc.assume_init() };
        unsafe {
            SetWindowPos(
                hwnd,
                None,
                rc.left,
                rc.top,
                rc.right - rc.left,
                rc.bottom - rc.top,
                SWP_FRAMECHANGED,
            )
            .expect("Failed to reset window frame");
        }

        return LRESULT(0);
    }

    if msg == WM_DESTROY {
        unsafe {
            PostQuitMessage(0);
        }
        return LRESULT(0);
    }

    if msg == WM_ACTIVATE {
        unsafe {
            DwmExtendFrameIntoClientArea(
                hwnd,
                &MARGINS {
                    cxLeftWidth: 1,
                    cxRightWidth: 1,
                    cyTopHeight: 1,
                    cyBottomHeight: 1,
                },
            )
            .expect("Failed to extend dwm frame");
        }

        return LRESULT(0);
    }

    if msg == WM_NCCALCSIZE {
        if wparam.0 == 1 {
            // remove non-client area

            let params = unsafe {
                core::mem::transmute::<_, *mut NCCALCSIZE_PARAMS>(lparam.0)
                    .as_mut()
                    .unwrap()
            };
            let w = unsafe { GetSystemMetrics(SM_CXSIZEFRAME) };
            let h = unsafe { GetSystemMetrics(SM_CYSIZEFRAME) };
            params.rgrc[0].left += w;
            params.rgrc[0].right -= w;
            params.rgrc[0].bottom -= h;
            // topはいじらない（他アプリもそんな感じになってるのでtopは自前でNCHITTESTしてリサイズ判定する）

            return LRESULT(0);
        }
    }

    if msg == WM_NCHITTEST {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        if let Some(x) = state.nc_hittest(hwnd, wparam, lparam) {
            return x;
        }
    }

    if msg == WM_DPICHANGED {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_dpi_changed((wparam.0 & 0xffff) as u16);
    }

    if msg == WM_SIZE {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.resize(
            (lparam.0 & 0xffff) as u16,
            ((lparam.0 >> 16) & 0xffff) as u16,
        );
        return LRESULT(0);
    }

    if msg == WM_MOUSEMOVE {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_mouse_move(
            (lparam.0 & 0xffff) as i16,
            ((lparam.0 >> 16) & 0xffff) as i16,
        );
        return LRESULT(0);
    }

    if msg == WM_LBUTTONDOWN {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_mouse_left_down(
            hwnd,
            (lparam.0 & 0xffff) as i16,
            ((lparam.0 >> 16) & 0xffff) as i16,
        );
        return LRESULT(0);
    }

    if msg == WM_LBUTTONUP {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        state.on_mouse_left_up(
            hwnd,
            (lparam.0 & 0xffff) as i16,
            ((lparam.0 >> 16) & 0xffff) as i16,
        );
        return LRESULT(0);
    }

    if msg == WM_SETCURSOR {
        let Some(state) = (unsafe {
            (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowStateModel).as_mut()
        }) else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        if state.handle_set_cursor() {
            return LRESULT(1);
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
