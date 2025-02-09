use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::Hash,
    rc::Rc,
};

use hittest::HitTestTreeActionHandler;
use windows::{
    core::{h, w, Interface, HRESULT, HSTRING, PCWSTR},
    Foundation::{
        Numerics::{Vector2, Vector3},
        Size, TimeSpan,
    },
    Graphics::DirectX::{DirectXAlphaMode, DirectXPixelFormat},
    Win32::{
        Foundation::{BOOL, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::{
            Direct2D::{
                CLSID_D2D1AlphaMask, CLSID_D2D1ColorMatrix, CLSID_D2D1ConvolveMatrix,
                Common::{
                    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_BORDER_MODE, D2D1_BORDER_MODE_SOFT,
                    D2D1_COLOR_F, D2D1_COMPOSITE_MODE_SOURCE_OVER, D2D1_PIXEL_FORMAT,
                    D2D_MATRIX_5X4_F, D2D_MATRIX_5X4_F_0, D2D_MATRIX_5X4_F_0_0, D2D_POINT_2F,
                    D2D_RECT_F,
                },
                D2D1CreateFactory, ID2D1Device, ID2D1DeviceContext, ID2D1Factory1,
                D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1,
                D2D1_COLORMATRIX_PROP_COLOR_MATRIX, D2D1_CONVOLVEMATRIX_PROP_BORDER_MODE,
                D2D1_CONVOLVEMATRIX_PROP_DIVISOR, D2D1_CONVOLVEMATRIX_PROP_KERNEL_MATRIX,
                D2D1_CONVOLVEMATRIX_PROP_KERNEL_SIZE_X, D2D1_CONVOLVEMATRIX_PROP_KERNEL_SIZE_Y,
                D2D1_DEBUG_LEVEL_WARNING, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_OPTIONS,
                D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
                D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR, D2D1_PROPERTY_TYPE_ENUM,
                D2D1_PROPERTY_TYPE_MATRIX_5X4, D2D1_PROPERTY_TYPE_UINT32,
                D2D1_PROPERTY_TYPE_UNKNOWN, D2D1_RENDER_TARGET_PROPERTIES,
                D2D1_RENDER_TARGET_TYPE_HARDWARE, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
            },
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_BIND_RENDER_TARGET,
                D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
                D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
            },
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
                DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
            },
            Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
            Dxgi::{
                Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
                IDXGIDevice, IDXGISurface,
            },
            Gdi::HBRUSH,
        },
        Storage::Packaging::Appx::PACKAGE_VERSION,
        System::{
            LibraryLoader::GetModuleHandleW,
            WinRT::{
                Composition::{
                    ICompositionDrawingSurfaceInterop, ICompositorDesktopInterop,
                    ICompositorInterop,
                },
                CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_ASTA,
                DQTYPE_THREAD_CURRENT,
            },
        },
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetMessageW,
                GetWindowLongPtrW, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassExW,
                SetCursor, SetWindowLongPtrW, ShowWindow, TranslateMessage, CW_USEDEFAULT,
                GWLP_USERDATA, HCURSOR, IDC_ARROW, IDC_SIZEWE, IDI_APPLICATION, SW_SHOW,
                WM_DESTROY, WM_DPICHANGED, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
                WM_SETCURSOR, WM_SIZE, WNDCLASSEXW, WNDCLASS_STYLES, WS_EX_APPWINDOW,
                WS_EX_NOREDIRECTIONBITMAP, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW,
            },
        },
    },
    UI::{
        Color,
        Composition::{
            CompositionDrawingSurface, CompositionGraphicsDevice, CompositionStretch, Compositor,
            ContainerVisual, ScalarKeyFrameAnimation, SpriteVisual, VisualCollection,
        },
    },
};

mod hittest;
mod input;

use crate::hittest::*;
use crate::input::*;

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

#[derive(Debug, Clone)]
pub struct TextFormatCacheKey {
    family_name: HSTRING,
    size: f32,
}
impl PartialEq for TextFormatCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.family_name.eq(&other.family_name) && self.size.to_bits().eq(&other.size.to_bits())
    }
}
impl Eq for TextFormatCacheKey {}
impl Hash for TextFormatCacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.family_name.hash(state);
        self.size.to_bits().hash(state);
    }
}

pub struct TextFormatStore {
    factory: IDWriteFactory,
    cache: RefCell<HashMap<TextFormatCacheKey, IDWriteTextFormat>>,
}
impl TextFormatStore {
    pub fn new(factory: IDWriteFactory) -> Self {
        Self {
            factory,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(
        &self,
        family_name: &HSTRING,
        size: f32,
    ) -> windows::core::Result<IDWriteTextFormat> {
        let key = TextFormatCacheKey {
            family_name: family_name.clone(),
            size,
        };

        if let Some(x) = self.cache.borrow().get(&key).cloned() {
            // ある
            return Ok(x);
        }

        let x = unsafe {
            self.factory.CreateTextFormat(
                &key.family_name,
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                key.size,
                w!("ja-JP"),
            )?
        };
        self.cache.borrow_mut().insert(key, x.clone());
        Ok(x)
    }
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

        let tf = init
            .subsystem
            .text_format_store
            .get(h!("system-ui"), 12.0)
            .unwrap();
        let tl = unsafe {
            init.subsystem
                .dwrite_factory
                .CreateTextLayout(
                    &init_label.encode_utf16().collect::<Vec<_>>(),
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
                    dc.Clear(Some(&D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }));
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

struct AppWindowData {
    ht: HitTestTreeContext,
    ht_root: HitTestTreeRef,
    client_size_pixels: SizePixels,
    dpi: f32,
    dpi_handlers: Vec<std::rc::Weak<dyn DpiHandler>>,
    pointer_input_manager: PointerInputManager,
}
impl AppWindowData {
    pub fn new(init_client_size_pixels: SizePixels, init_dpi: f32) -> Self {
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

        println!("init dpi: {init_dpi}");

        Self {
            ht,
            ht_root,
            client_size_pixels: init_client_size_pixels,
            dpi: init_dpi,
            dpi_handlers: Vec::new(),
            pointer_input_manager: PointerInputManager::new(),
        }
    }
}

pub struct Subsystem {
    d3d11_device: ID3D11Device,
    d3d11_imm_context: ID3D11DeviceContext,
    d2d1_device: ID2D1Device,
    dwrite_factory: IDWriteFactory,
    text_format_store: TextFormatStore,
    compositor: Compositor,
    compositor_desktop_interop: ICompositorDesktopInterop,
    composition_2d_graphics_device: CompositionGraphicsDevice,
}
impl Subsystem {
    pub fn new() -> Self {
        let mut d3d11_device = core::mem::MaybeUninit::uninit();
        let mut feature_level = core::mem::MaybeUninit::uninit();
        let mut d3d11_imm_context = core::mem::MaybeUninit::uninit();
        unsafe {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE(core::ptr::null_mut()),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(d3d11_device.as_mut_ptr()),
                Some(feature_level.as_mut_ptr()),
                Some(d3d11_imm_context.as_mut_ptr()),
            )
            .expect("Failed to create D3D11 Device");
        }
        let d3d11_device = unsafe {
            d3d11_device
                .assume_init()
                .expect("no d3d11 device provided?")
        };
        let feature_level = unsafe { feature_level.assume_init() };
        let d3d11_imm_context = unsafe {
            d3d11_imm_context
                .assume_init()
                .expect("no d3d11 imm context provided?")
        };
        println!("d3d11 feature level = {feature_level:?}");

        let d2d1_factory: ID2D1Factory1 = unsafe {
            D2D1CreateFactory(
                D2D1_FACTORY_TYPE_SINGLE_THREADED,
                Some(&D2D1_FACTORY_OPTIONS {
                    debugLevel: D2D1_DEBUG_LEVEL_WARNING,
                }),
            )
            .expect("Failed to create d2d1 factory")
        };
        let d2d1_device = unsafe {
            d2d1_factory
                .CreateDevice(&d3d11_device.cast::<IDXGIDevice>().expect("no dxgi device?"))
                .expect("Failed to create d2d1 device")
        };

        let dwrite_factory: IDWriteFactory = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .expect("Failed to create dwrite factory")
        };

        let compositor = Compositor::new().expect("Failed to create ui compositor");
        let compositor_interop = compositor
            .cast::<ICompositorInterop>()
            .expect("no compositor interop support");
        let compositor_desktop_interop = compositor
            .cast::<ICompositorDesktopInterop>()
            .expect("no compositor desktop interop");
        let composition_2d_graphics_device = unsafe {
            compositor_interop
                .CreateGraphicsDevice(&d2d1_device)
                .expect("Failed to create composition 2d graphics device")
        };

        Self {
            d3d11_device,
            d3d11_imm_context,
            d2d1_device,
            dwrite_factory: dwrite_factory.clone(),
            text_format_store: TextFormatStore::new(dwrite_factory),
            compositor,
            compositor_desktop_interop,
            composition_2d_graphics_device,
        }
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
            w!("Peridot Sprite Atlas Visualizer/Editor"),
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

    let desktop_window_target = unsafe {
        subsystem
            .compositor_desktop_interop
            .CreateDesktopWindowTarget(hw, true)
            .expect("Failed to create desktop window target")
    };

    let mut cr = core::mem::MaybeUninit::uninit();
    unsafe {
        GetClientRect(hw, cr.as_mut_ptr()).expect("Failed to get initial client rect size");
    }
    let cr = unsafe { cr.assume_init() };

    let mut app_window_data = AppWindowData::new(
        SizePixels {
            width: (cr.right - cr.left) as _,
            height: (cr.bottom - cr.top) as _,
        },
        unsafe { GetDpiForWindow(hw) as f32 },
    );
    unsafe {
        SetWindowLongPtrW(hw, GWLP_USERDATA, &mut app_window_data as *mut _ as _);
    }

    let composition_root = subsystem
        .compositor
        .CreateContainerVisual()
        .expect("Failed to create composition root visual");
    composition_root
        .SetRelativeSizeAdjustment(Vector2::one())
        .expect("Failed to set root visual sizing");
    desktop_window_target
        .SetRoot(&composition_root)
        .expect("Failed to set composition root visual");

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

    let main_presenter = MainPresenter::new(
        &mut PresenterInitContext {
            for_view: ViewInitContext {
                subsystem: &subsystem,
                ht: &mut app_window_data.ht,
                dpi: app_window_data.dpi,
            },
            dpi_handlers: &mut app_window_data.dpi_handlers,
        },
        400.0,
    );
    main_presenter.mount(
        &mut app_window_data.ht,
        &composition_root.Children().unwrap(),
        app_window_data.ht_root,
    );

    app_window_data.ht.dump(app_window_data.ht_root);

    unsafe {
        let _ = ShowWindow(hw, SW_SHOW);
    }

    let mut msg = core::mem::MaybeUninit::uninit();
    while unsafe { GetMessageW(msg.as_mut_ptr(), None, 0, 0).0 > 0 } {
        unsafe {
            let _ = TranslateMessage(msg.as_ptr());
            DispatchMessageW(msg.as_ptr());
        }
    }

    unsafe {
        SetWindowLongPtrW(hw, GWLP_USERDATA, 0);
    }

    main_presenter.unmount(&mut app_window_data.ht);
    main_presenter.drop_context(&mut app_window_data.ht, &mut app_window_data.dpi_handlers);
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_DESTROY {
        unsafe {
            PostQuitMessage(0);
        }
        return LRESULT(0);
    }

    if msg == WM_DPICHANGED {
        let Some(app_window_data) =
            (unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowData).as_mut() })
        else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        let new_dpi = (wparam.0 & 0xffff) as u16;
        app_window_data.dpi = new_dpi as _;
        for x in app_window_data.dpi_handlers.iter() {
            if let Some(x) = x.upgrade() {
                x.on_dpi_changed(new_dpi as _);
            }
        }
    }

    if msg == WM_SIZE {
        let Some(app_window_data) =
            (unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowData).as_mut() })
        else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        app_window_data.client_size_pixels.width = (lparam.0 & 0xffff) as _;
        app_window_data.client_size_pixels.height = ((lparam.0 >> 16) & 0xffff) as _;

        return LRESULT(0);
    }

    if msg == WM_MOUSEMOVE {
        let Some(app_window_data) =
            (unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowData).as_mut() })
        else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        app_window_data.pointer_input_manager.on_mouse_move(
            &mut app_window_data.ht,
            app_window_data.ht_root,
            app_window_data
                .client_size_pixels
                .to_dip(app_window_data.dpi),
            signed_pixels_to_dip((lparam.0 & 0xffff) as i16 as i32, app_window_data.dpi),
            signed_pixels_to_dip(((lparam.0 >> 16) & 0xffff) as i16 as _, app_window_data.dpi),
        );

        // WM_SETCURSORが飛ばないことがあるのでここで設定する
        if let Some(c) = app_window_data
            .pointer_input_manager
            .cursor(&app_window_data.ht)
        {
            unsafe {
                SetCursor(Some(c));
            }
        }

        return LRESULT(0);
    }

    if msg == WM_LBUTTONDOWN {
        let Some(app_window_data) =
            (unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowData).as_mut() })
        else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        app_window_data.pointer_input_manager.on_mouse_left_down(
            hwnd,
            &mut app_window_data.ht,
            app_window_data.ht_root,
            app_window_data
                .client_size_pixels
                .to_dip(app_window_data.dpi),
            signed_pixels_to_dip((lparam.0 & 0xffff) as i16 as i32, app_window_data.dpi),
            signed_pixels_to_dip(((lparam.0 >> 16) & 0xffff) as i16 as _, app_window_data.dpi),
        );

        return LRESULT(0);
    }

    if msg == WM_LBUTTONUP {
        let Some(app_window_data) =
            (unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowData).as_mut() })
        else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        app_window_data.pointer_input_manager.on_mouse_left_up(
            hwnd,
            &mut app_window_data.ht,
            app_window_data.ht_root,
            app_window_data
                .client_size_pixels
                .to_dip(app_window_data.dpi),
            signed_pixels_to_dip((lparam.0 & 0xffff) as i16 as i32, app_window_data.dpi),
            signed_pixels_to_dip(((lparam.0 >> 16) & 0xffff) as i16 as _, app_window_data.dpi),
        );

        return LRESULT(0);
    }

    if msg == WM_SETCURSOR {
        let Some(app_window_data) =
            (unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppWindowData).as_mut() })
        else {
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        };

        if let Some(c) = app_window_data
            .pointer_input_manager
            .cursor(&app_window_data.ht)
        {
            unsafe {
                SetCursor(Some(c));
            }
            return LRESULT(1);
        }
    }

    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
