use crate::theme::lerp;
use winapi::shared::windef::HICON;
use winapi::um::winuser::{
    CreateIcon, LoadIconW, LoadImageW, IDI_APPLICATION, IMAGE_ICON, LR_LOADFROMFILE,
};

fn point_in_poly(x: f32, y: f32, poly: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn make_icon_rgba(size: usize, active: bool) -> Vec<u8> {
    let n = 4usize;
    let s = size as f32;
    let bolt: [(f32, f32); 7] = [
        (0.58, 0.12),
        (0.28, 0.55),
        (0.46, 0.55),
        (0.40, 0.88),
        (0.72, 0.43),
        (0.54, 0.43),
        (0.64, 0.12),
    ];
    let (c0, c1, bolt_col) = if active {
        ((246.0, 248.0, 252.0), (150.0, 158.0, 174.0), (24.0, 26.0, 32.0))
    } else {
        ((116.0, 118.0, 126.0), (66.0, 68.0, 76.0), (200.0, 203.0, 210.0))
    };
    let mut out = vec![0u8; size * size * 4];
    for py in 0..size {
        for px in 0..size {
            let (mut r_sum, mut g_sum, mut b_sum, mut a_sum) = (0f32, 0f32, 0f32, 0f32);
            for sy in 0..n {
                for sx in 0..n {
                    let x = (px as f32 + (sx as f32 + 0.5) / n as f32) / s;
                    let y = (py as f32 + (sy as f32 + 0.5) / n as f32) / s;
                    let half = 0.5 - 0.03;
                    let rad = 0.24;
                    let dx = ((x - 0.5).abs() - (half - rad)).max(0.0);
                    let dy = ((y - 0.5).abs() - (half - rad)).max(0.0);
                    if dx * dx + dy * dy <= rad * rad {
                        let (r, g, b) = if point_in_poly(x, y, &bolt) {
                            bolt_col
                        } else {
                            let t = ((x + y) * 0.5).clamp(0.0, 1.0);
                            (
                                lerp(c0.0, c1.0, t),
                                lerp(c0.1, c1.1, t),
                                lerp(c0.2, c1.2, t),
                            )
                        };
                        r_sum += r;
                        g_sum += g;
                        b_sum += b;
                        a_sum += 1.0;
                    }
                }
            }
            if a_sum > 0.0 {
                let i = (py * size + px) * 4;
                out[i] = (r_sum / a_sum) as u8;
                out[i + 1] = (g_sum / a_sum) as u8;
                out[i + 2] = (b_sum / a_sum) as u8;
                out[i + 3] = (a_sum / (n * n) as f32 * 255.0) as u8;
            }
        }
    }
    out
}

pub fn rgba_to_hicon(rgba: &[u8], size: usize) -> HICON {
    let mut and_mask: Vec<u8> = Vec::with_capacity(size * size);
    let mut bgra = rgba.to_vec();
    for px in bgra.chunks_exact_mut(4) {
        and_mask.push(px[3].wrapping_sub(255));
        px.swap(0, 2);
    }
    unsafe {
        CreateIcon(
            std::ptr::null_mut(),
            size as i32,
            size as i32,
            1,
            32,
            and_mask.as_ptr(),
            bgra.as_ptr(),
        )
    }
}

pub fn load_icon_on(size: i32) -> HICON {
    unsafe {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let icon_path = dir.join("icon.ico");
                if icon_path.exists() {
                    let path_wide: Vec<u16> = icon_path
                        .to_string_lossy()
                        .encode_utf16()
                        .chain(std::iter::once(0))
                        .collect();
                    let hicon = LoadImageW(
                        std::ptr::null_mut(),
                        path_wide.as_ptr(),
                        IMAGE_ICON,
                        size,
                        size,
                        LR_LOADFROMFILE,
                    ) as HICON;
                    if !hicon.is_null() {
                        return hicon;
                    }
                }
            }
        }
        let hicon = rgba_to_hicon(&make_icon_rgba(size as usize, true), size as usize);
        if !hicon.is_null() {
            return hicon;
        }
        LoadIconW(std::ptr::null_mut(), IDI_APPLICATION)
    }
}
