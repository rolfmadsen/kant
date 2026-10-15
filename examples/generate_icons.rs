use std::path::Path;

fn main() {
    let svg_path = Path::new("assets/kant_logo.svg");
    let svg_data = std::fs::read(svg_path).expect("skal kunne læse assets/kant_logo.svg");

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &opt).expect("skal kunne parse SVG");

    // 1. Generer 32x32 RGBA til kant-32.rgba
    let pixmap_32 = render_svg(&tree, 32, 32);
    let mut rgba_32 = Vec::with_capacity(32 * 32 * 4);
    for pixel in pixmap_32.pixels() {
        let c = pixel.demultiply();
        rgba_32.push(c.red());
        rgba_32.push(c.green());
        rgba_32.push(c.blue());
        rgba_32.push(c.alpha());
    }
    std::fs::write("assets/icons/kant-32.rgba", &rgba_32)
        .expect("skal kunne skrive assets/icons/kant-32.rgba");
    println!("Skrev assets/icons/kant-32.rgba ({} bytes)", rgba_32.len());

    // 2. Generer multi-resolution ICO (16, 32, 48, 256) samt standalone PNG ikoner
    let sizes = [16u32, 32, 48, 128, 256];
    let mut png_blobs: Vec<(u32, Vec<u8>)> = Vec::new();

    for &s in &sizes {
        let pm = render_svg(&tree, s, s);
        let png_bytes = pm.encode_png().expect("skal kunne encode PNG");
        png_blobs.push((s, png_bytes.clone()));

        if s == 48 || s == 128 || s == 256 {
            let path = format!("assets/icons/kant-{}.png", s);
            std::fs::write(&path, &png_bytes).expect("skal kunne skrive PNG");
            println!("Skrev {} ({} bytes)", path, png_bytes.len());
        }
        if s == 256 {
            std::fs::write("assets/icons/kant.png", &png_bytes).expect("skal kunne skrive kant.png");
            println!("Skrev assets/icons/kant.png ({} bytes)", png_bytes.len());
        }
    }

    // Byg ICO med standardstørrelser (16, 32, 48, 256)
    let ico_sizes = [16u32, 32, 48, 256];
    let ico_blobs: Vec<_> = png_blobs
        .iter()
        .filter(|(s, _)| ico_sizes.contains(s))
        .collect();

    let mut ico = Vec::new();
    // ICO Header
    ico.extend_from_slice(&[0x00, 0x00]); // Reserved
    ico.extend_from_slice(&[0x01, 0x00]); // Type 1 = Icon
    ico.extend_from_slice(&(ico_blobs.len() as u16).to_le_bytes()); // Number of images

    let mut offset = 6 + (ico_blobs.len() * 16) as u32;

    for (s, png) in &ico_blobs {
        let width_byte = if *s >= 256 { 0u8 } else { *s as u8 };
        let height_byte = if *s >= 256 { 0u8 } else { *s as u8 };

        ico.push(width_byte);
        ico.push(height_byte);
        ico.push(0); // Color palette
        ico.push(0); // Reserved
        ico.extend_from_slice(&1u16.to_le_bytes()); // Color planes
        ico.extend_from_slice(&32u16.to_le_bytes()); // Bits per pixel
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes()); // Size of image data
        ico.extend_from_slice(&offset.to_le_bytes()); // Offset of image data

        offset += png.len() as u32;
    }

    for (_, png) in &ico_blobs {
        ico.extend_from_slice(png);
    }

    std::fs::write("assets/icons/kant.ico", &ico).expect("skal kunne skrive assets/icons/kant.ico");
    println!("Skrev assets/icons/kant.ico ({} bytes)", ico.len());
}

fn render_svg(tree: &usvg::Tree, width: u32, height: u32) -> tiny_skia::Pixmap {
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    let size = tree.size().to_int_size();
    let sx = width as f32 / size.width() as f32;
    let sy = height as f32 / size.height() as f32;
    let transform = tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(tree, transform, &mut pixmap.as_mut());
    pixmap
}
