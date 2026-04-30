use image::{ImageReader, Pixel};

fn main() {
    println!("cargo:rerun-if-changed=icons/");

    let path = std::path::PathBuf::from("icons");
    let glyph_path = std::path::PathBuf::from("glyphs");

    // Generate home_nano_nbgl.png from the 14x14 icon + mask
    let reader = ImageReader::open(path.join("squads_14x14.gif")).unwrap();
    let img = reader.decode().unwrap();
    let mut gray = img.into_luma8();

    let mask = ImageReader::open(path.join("mask_14x14.gif"))
        .unwrap()
        .decode()
        .unwrap()
        .into_luma8();

    for (x, y, mask_pixel) in mask.enumerate_pixels() {
        let mask_value = mask_pixel[0];
        let mut gray_pixel = *gray.get_pixel(x, y);
        if mask_value == 0 {
            gray_pixel = image::Luma([0]);
        } else {
            gray_pixel.invert();
        }
        gray.put_pixel(x, y, gray_pixel);
    }

    gray.save_with_format(
        glyph_path.join("home_nano_nbgl.png"),
        image::ImageFormat::Png,
    )
    .unwrap();
}
