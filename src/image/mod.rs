use image::{Pixel, ImageDecoder};
use image::io::Reader as ImageReader;
use std::fs::File;
use std::io::{Write, BufWriter};


// write the pixels to a csv for checks against python editor scriot
fn write_pixels_to_csv(r: &Vec<u8>, g: &Vec<u8>, b: &Vec<u8>) {
    let output_csv = File::create("output/pixels_rust.csv").expect("Failed to create csv");
    let mut writer = BufWriter::new(output_csv);

    // loop over the three vectors and write rows [r, g, b]
    for i in 0..r.len() {
        writeln!(writer, "{},{},{}", r[i], g[i], b[i]).unwrap();
    }
}


// open the image and read the metadata (return vector of bytes for metadata values)
// TODO: finish this
pub fn get_image_exifdata(original_img: &String) -> Vec<u8> {
    println!("[*] Opening image and reading exifdata...");
    let img_reader = ImageReader::open(original_img).expect("[!] Failed to open image");
    let mut decoder = img_reader.into_decoder().expect("[!] Failed to create image decoder");
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
            r.push(pixel_channels[0] as u8); // u8 (will need to round greyscale coefficients to use ints not floats)
            g.push(pixel_channels[1] as u8);
            b.push(pixel_channels[2] as u8);
        }
    }

    // write the pixel rgb values to csv
    write_pixels_to_csv(&r, &g, &b);

    (r, g, b)
}
