use image::{Pixel, ImageDecoder};
use image::ImageReader as ImageReader;
use std::fs;
use std::io::{Write, Read, BufWriter};


// open the image and read the metadata (return vector of bytes for metadata values)
// TODO: rewrite this to extract based on what python writes in before IEND block
pub fn get_image_exifdata(original_img: &String) -> Vec<u8> {
    println!("[*] Opening image and reading exifdata...");
    let png_data = fs::read(original_img).expect("[!] Failed to read image contents");
    let png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A";
    
    // check if image has the correct signature
    if !png_data.starts_with(png_signature) {
        println!("[!] Image is not a png");
        return vec![];
    }

    println!("[*] According to magic bytes image is a png");
    // parse the png bytes for the slice that contains the exif data
    for i in 8..png_data.len() { // start at 8 to skip the signature
        let length = u32::from_be_bytes(png_data[i..i+4].try_into().expect("[!] Failed to convert to bytes")) as usize;
        let chunk_type = &png_data[i+4..i+8];

        if chunk_type == b"eXIf" {
            println!("[*] Located the eXIf chunk");
        }
    }

    vec![0u8]
}


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
