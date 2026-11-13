//! A simple edge-preserving denoiser: a-trous wavelet filter guided by the normals, depths and colors of the picture.

use super::buffers::FrameBuffers;
use super::parallel::for_each_row;

const KERNEL: [f32; 5] = [1.0 / 16.0, 0.25, 0.375, 0.25, 1.0 / 16.0];
const PASSES: usize = 3;

fn display_luma(r: f32, g: f32, b: f32) -> f32 {
    let l = (0.2126 * r + 0.7152 * g + 0.0722 * b).max(0.0);
    l / (1.0 + l)
}

/// Smooths the sampling noise of `frame.color` without blurring across edges of geometry or strong color
/// changes. Alpha is left alone.
pub fn denoise(frame: &mut FrameBuffers) {
    let (w, h) = (frame.width, frame.height);
    if w < 3 || h < 3 {
        return;
    }
    let mut source = frame.color.clone();
    let mut target = frame.color.clone();
    for pass in 0..PASSES {
        let step = 1isize << pass;
        let color_sigma = 0.10 / (1.0 + pass as f32 * 0.8);
        let (src, depth, normal) = (&source, &frame.depth, &frame.normal);
        for_each_row(&mut target, w * 4, |y, row| {
            for x in 0..w {
                let center = y * w + x;
                let c = &src[center * 4..center * 4 + 4];
                let lc = display_luma(c[0], c[1], c[2]);
                let (dc, nc) = (depth[center], &normal[center * 3..center * 3 + 3]);
                let sky = nc == [0.0, 0.0, 0.0];
                let mut sum = [0.0f32; 3];
                let mut total = 0.0f32;
                for (j, kj) in KERNEL.iter().enumerate() {
                    let ny =
                        (y as isize + (j as isize - 2) * step).clamp(0, h as isize - 1) as usize;
                    for (i, ki) in KERNEL.iter().enumerate() {
                        let nx = (x as isize + (i as isize - 2) * step).clamp(0, w as isize - 1)
                            as usize;
                        let at = ny * w + nx;
                        let n = &normal[at * 3..at * 3 + 3];
                        let weight_n = if sky {
                            f32::from(n == [0.0, 0.0, 0.0])
                        } else {
                            (nc[0] * n[0] + nc[1] * n[1] + nc[2] * n[2])
                                .max(0.0)
                                .powi(24)
                        };
                        if weight_n <= 0.0 {
                            continue;
                        }
                        let rel = (depth[at] - dc).abs() / dc.max(0.2);
                        let weight_d = (-(rel / 0.12).powi(2)).exp();
                        let p = &src[at * 4..at * 4 + 4];
                        let dl = (display_luma(p[0], p[1], p[2]) - lc).abs();
                        let weight_c = (-dl / color_sigma).exp();
                        let weight = ki * kj * weight_n * weight_d * weight_c;
                        sum[0] += p[0] * weight;
                        sum[1] += p[1] * weight;
                        sum[2] += p[2] * weight;
                        total += weight;
                    }
                }
                let out = &mut row[x * 4..x * 4 + 4];
                if total > 1e-6 {
                    out[..3].copy_from_slice(&[sum[0] / total, sum[1] / total, sum[2] / total]);
                } else {
                    out[..3].copy_from_slice(&c[..3]);
                }
                out[3] = c[3];
            }
        });
        std::mem::swap(&mut source, &mut target);
    }
    frame.color = source;
}
