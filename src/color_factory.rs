use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

pub const fn ui_color_from_hex_rgb_with_alpha(hex: u32, alpha: u8) -> windows::UI::Color {
    windows::UI::Color {
        R: ((hex >> 16) & 0xff) as _,
        G: ((hex >> 8) & 0xff) as _,
        B: (hex & 0xff) as _,
        A: alpha,
    }
}

pub const fn ui_color_from_hex_rgb(hex: u32) -> windows::UI::Color {
    ui_color_from_hex_rgb_with_alpha(hex, 255)
}

pub const fn ui_color_from_websafe_hex_rgb_with_alpha(hex: u32, alpha: u8) -> windows::UI::Color {
    ui_color_from_hex_rgb_with_alpha(expand_websafe_hexcolor(hex), alpha)
}

pub const fn ui_color_from_websafe_hex_rgb(hex: u32) -> windows::UI::Color {
    ui_color_from_websafe_hex_rgb_with_alpha(hex, 255)
}

pub const fn d2d1_color_f_from_hex_argb(hex: u32) -> D2D1_COLOR_F {
    let au = ((hex >> 24) & 0xff) as u8;
    let ru = ((hex >> 16) & 0xff) as u8;
    let gu = ((hex >> 8) & 0xff) as u8;
    let bu = (hex & 0xff) as u8;

    D2D1_COLOR_F {
        r: ru as f32 / 255.0,
        g: gu as f32 / 255.0,
        b: bu as f32 / 255.0,
        a: au as f32 / 255.0,
    }
}

pub const fn d2d1_color_f_from_hex_rgb(hex: u32) -> D2D1_COLOR_F {
    d2d1_color_f_from_hex_argb(0xff00_0000 | (hex & 0x00ff_ffff))
}

pub const fn d2d1_color_f_from_websafe_hex_argb(hex: u32) -> D2D1_COLOR_F {
    d2d1_color_f_from_hex_argb(expand_websafe_hexcolor(hex))
}

pub const fn d2d1_color_f_from_websafe_hex_rgb(hex: u32) -> D2D1_COLOR_F {
    d2d1_color_f_from_websafe_hex_argb(0xf000 | (hex & 0x0fff))
}

const fn expand_websafe_hexcolor(hex: u32) -> u32 {
    const fn e(x: u32) -> u32 {
        x | (x << 4)
    }

    (e((hex >> 12) & 0x0f) << 24)
        | (e((hex >> 8) & 0x0f) << 16)
        | (e((hex >> 4) & 0x0f) << 8)
        | e(hex & 0x0f)
}
