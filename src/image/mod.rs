use image::{Pixel};


// open the image
pub fn get_image_rgb_values() {
    let img = image::open("/home/cdeclan/CryptoHack/image_provenance/image.png").expect("[!] Failed to open image");
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    // iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in 0..height { // y=0 -> top row
        for x in 0..width { // x=0 -> leftmost column
            let pixel_channels = rgb.get_pixel(x, y).channels();
            let r = pixel_channels[0]; // u8 (will need to round greyscale coefficients to use ints not floats)
            let g = pixel_channels[1];
            let b = pixel_channels[2];
        }
    }
}