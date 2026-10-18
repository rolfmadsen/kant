use std::fs;
use std::path::PathBuf;

fn main() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let svg_path = repo_root.join("assets/icons/kant.svg");
    let out_dir = repo_root.join("assets/windows/images");

    fs::create_dir_all(&out_dir).expect("Kunne ikke oprette assets/windows/images mappe");

    let svg_data = fs::read(&svg_path).expect("Kunne ikke læse kant.svg");
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&svg_data, &opt).expect("Kunne ikke parse kant.svg");

    // Definer de påkrævede aktiver og dimensioner
    let targets = [
        ("Square44x44Logo.png", 44, 44, 44),
        ("Square50x50Logo.png", 50, 50, 50),
        ("Square150x150Logo.png", 150, 150, 150),
        ("Wide310x150Logo.png", 310, 150, 130), // 310x150 canvas med 130x130 centreret logo
        ("SplashScreen.png", 620, 300, 200),    // 620x300 canvas med 200x200 centreret logo
    ];

    let svg_w = tree.size().width();
    let svg_h = tree.size().height();

    for (filename, canvas_w, canvas_h, icon_size) in targets {
        let mut pixmap =
            tiny_skia::Pixmap::new(canvas_w, canvas_h).expect("Kunne ikke allokere Pixmap");

        // Skaleringsfaktor og centrering
        let scale = icon_size as f32 / svg_w.max(svg_h);
        let offset_x = (canvas_w as f32 - (svg_w * scale)) / 2.0;
        let offset_y = (canvas_h as f32 - (svg_h * scale)) / 2.0;

        let transform =
            tiny_skia::Transform::from_translate(offset_x, offset_y).pre_scale(scale, scale);

        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let out_path = out_dir.join(filename);
        pixmap.save_png(&out_path).expect("Kunne ikke gemme PNG");
        println!("Genereret: {:?} ({}x{})", out_path, canvas_w, canvas_h);
    }

    println!("Alle Windows Store billedaktiver er genereret med succes!");
}
