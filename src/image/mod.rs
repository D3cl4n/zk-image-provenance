use image::{Pixel};


// open the image and store the rgb values
pub fn get_image_rgb_values(original_img: &String) -> (u32, u32, Vec<u8>, Vec<u8>, Vec<u8>) {
    // open image and convert to RGB8
    println!("[*] Opening image and parsing out rgb values...");
    let img = image::open(original_img).expect("[!] Failed to open image");
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    println!("[*] Original image is {}x{}", width, height);

    // vectors for r, g, b bytes
    let mut r: Vec<u8> = vec![];
    let mut g: Vec<u8> = vec![];
    let mut b: Vec<u8> = vec![];

    // iterate over the pixels and extract the 3 bytes for color channels (R, G, B)
    for y in 0..height { // y=0 -> top row
        for x in 0..width { // x=0 -> leftmost column
            let pixel_channels = rgb.get_pixel(x, y).channels();
            r.push(pixel_channels[0] as u8); // u8 (will need to round greyscale coefficients to use ints not floats)
            g.push(pixel_channels[1] as u8);
            b.push(pixel_channels[2] as u8);
        }
    }

    (width, height, r, g, b)
}
