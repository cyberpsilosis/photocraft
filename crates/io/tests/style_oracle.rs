//! A layer-style document rendered beside its maker's Photoshop render of the same document, the
//! oracle. Both come from a paid asset pack, so they stay out of git: the test is ignored by
//! default and reads their paths from the environment.
//!
//! ```sh
//! PHOTOCRAFT_STYLE_PSB=<the document> PHOTOCRAFT_STYLE_RENDER=<Photoshop's render, a PNG> \
//! PHOTOCRAFT_STYLE_AT=<x,y of the render's top-left in the document> \
//!   cargo test --release -p photocraft-io --test style_oracle -- --ignored --nocapture
//! ```
//!
//! The render is aligned on its alpha within 8 px of `PHOTOCRAFT_STYLE_AT`, then four measures
//! are taken over the pixels both cover: the share of near-black pixels (luma under 20 of 255),
//! the share of white ones (over 235), the mean luma and the mean saturation (HSV). On a chrome
//! style (Satin, Inner Shadow, Bevel and Colour Overlay on the art, Overlay layers clipped to
//! it, a Gradient Map and a Color Balance above) Photoshop's render measures 31.3 %, 7.6 %, 83.7
//! and 0.62. With the clipped layers beneath every effect PhotoCraft measured 10.8 %, 13.3 %,
//! 121.6 and 0.30; with them between the fill-type interior effects and the Inner Shadow, 31.4 %,
//! 9.0 %, 89.8 and 0.36. The bounds below sit between the two. The saturation bound is loose: the
//! rest of that gap is the Color Balance adjustment, which this test does not isolate.

use std::path::Path;

fn render(path: &str) -> (Vec<[f32; 4]>, usize, usize) {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let name = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path);
    let doc = photocraft_io::import(name, &bytes).unwrap_or_else(|e| panic!("{path}: {e}")).document;
    // Measured as 8-bit pixels, as the render is stored: a float near-black pixel can carry a
    // large saturation that 8 bits round away.
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() / 255.0;
    let px = photocraft_compose::flatten(&doc).px.into_iter().map(|p| [q(p[0]), q(p[1]), q(p[2]), q(p[3])]).collect();
    (px, doc.size.width as usize, doc.size.height as usize)
}

fn luma(p: &[f32; 4]) -> f32 {
    (0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2]) * 255.0
}

fn saturation(p: &[f32; 4]) -> f32 {
    let (mx, mn) = (p[0].max(p[1]).max(p[2]), p[0].min(p[1]).min(p[2]));
    if mx > 0.0 { (mx - mn) / mx } else { 0.0 }
}

/// Near-black %, white %, mean luma, mean saturation over `pixels`.
fn measures(pixels: &[[f32; 4]]) -> [f32; 4] {
    let n = pixels.len().max(1) as f32;
    let ls: Vec<f32> = pixels.iter().map(luma).collect();
    [
        ls.iter().filter(|l| **l < 20.0).count() as f32 / n * 100.0,
        ls.iter().filter(|l| **l > 235.0).count() as f32 / n * 100.0,
        ls.iter().sum::<f32>() / n,
        pixels.iter().map(saturation).sum::<f32>() / n,
    ]
}

#[test]
#[ignore = "needs a paid style document and its Photoshop render (PHOTOCRAFT_STYLE_PSB, _RENDER, _AT)"]
fn style_matches_its_photoshop_render() {
    let (Ok(psb), Ok(png), Ok(at)) = (std::env::var("PHOTOCRAFT_STYLE_PSB"), std::env::var("PHOTOCRAFT_STYLE_RENDER"), std::env::var("PHOTOCRAFT_STYLE_AT"))
    else {
        panic!("set PHOTOCRAFT_STYLE_PSB, PHOTOCRAFT_STYLE_RENDER and PHOTOCRAFT_STYLE_AT");
    };
    let at: Vec<i64> = at.split(',').filter_map(|v| v.trim().parse().ok()).collect();
    let [x0, y0] = at[..] else { panic!("PHOTOCRAFT_STYLE_AT is x,y") };
    let (ours, w, h) = render(&psb);
    let (theirs, tw, th) = render(&png);
    let ours_at = |x: i64, y: i64| if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 { None } else { ours.get(y as usize * w + x as usize) };
    // Alignment: the offset with the best overlap of the two alphas.
    let mut best = (0.0f32, 0i64, 0i64);
    for dy in -8..=8i64 {
        for dx in -8..=8i64 {
            let (mut both, mut either) = (0u64, 0u64);
            for ty in 0..th {
                for tx in 0..tw {
                    let a = theirs[ty * tw + tx][3] > 0.5;
                    let b = ours_at(x0 + dx + tx as i64, y0 + dy + ty as i64).is_some_and(|p| p[3] > 0.5);
                    both += u64::from(a && b);
                    either += u64::from(a || b);
                }
            }
            let iou = both as f32 / either.max(1) as f32;
            if iou > best.0 {
                best = (iou, dx, dy);
            }
        }
    }
    let (iou, dx, dy) = best;
    assert!(iou > 0.9, "the render does not line up with the document (best overlap {iou:.3} at {dx},{dy})");
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for ty in 0..th {
        for tx in 0..tw {
            let t = theirs[ty * tw + tx];
            if let Some(o) = ours_at(x0 + dx + tx as i64, y0 + dy + ty as i64)
                && t[3] > 0.5
                && o[3] > 0.5
            {
                a.push(t);
                b.push(*o);
            }
        }
    }
    let (ps, pc) = (measures(&a), measures(&b));
    println!("overlap {iou:.3} at {dx},{dy}, {} pixels", a.len());
    for (k, name) in ["near-black %", "white %", "mean luma", "saturation"].iter().enumerate() {
        println!("{name:>13}: Photoshop {:7.2}  PhotoCraft {:7.2}", ps[k], pc[k]);
    }
    let bounds = [3.0, 3.0, 10.0, 0.29];
    let mut off = Vec::new();
    for (k, name) in ["near-black %", "white %", "mean luma", "saturation"].iter().enumerate() {
        if (ps[k] - pc[k]).abs() > bounds[k] {
            off.push(format!("{name}: {:.2} against Photoshop's {:.2} (bound {})", pc[k], ps[k], bounds[k]));
        }
    }
    assert!(off.is_empty(), "{}", off.join("; "));
}
