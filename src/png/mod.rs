use image::Pixel;
use std::fs;


// open the image and read for a specific chunk by type, return (length, type, data, crc)




// open the image and store the greyscale values
pub fn get_image_greyscale_values(edited_img: &String) -> Vec<u8> {
    println!("[*] Opening edited image and parsing out pixel values...");
    let grey_img = image::open(edited_img).expect("[!] Failed to open image");
    let rgb = grey_img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut grey_vals: Vec<u8> = vec![];
    for y in 0..height {
        for x in 0..width {
            let grey_pixel_channels = rgb.get_pixel(x, y).channels();
            if grey_pixel_channels[0] == grey_pixel_channels[1] && grey_pixel_channels[0] == grey_pixel_channels[2] {
                grey_vals.push(grey_pixel_channels[0]);
            }
        }
    }

    grey_vals
}


// open the image and store the rgb values
pub fn get_image_rgb_values(original_img: &String) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
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
            r.push(pixel_channels[0] as u8);
            g.push(pixel_channels[1] as u8);
            b.push(pixel_channels[2] as u8);
        }
    }

    (r, g, b)
}
