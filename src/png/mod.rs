use image::Pixel;
use std::fs::File;
use std::io::{Read, BufReader};
use crc32fast::Hasher;


// structure to hold a png chunk
struct PngChunk {
    chunk_length: u32,
    chunk_type: [u8; 4],
    chunk_data: Vec<u8>,
    crc: u32
}


// read a single chunk from the png
fn read_single_chunk<R: Read>(buf_reader: &mut R) -> PngChunk {
    let mut len_buf: [u32; 4] = [0u8; 4];
    let mut type_buf: [u32; 4] = [0u8; 4];

    // start reading the chunk fields and data
    buf_reader.read_exact(&mut len_buf)?;
}


// open the image and read for a specific chunk by type, return (length, type, data, crc)
pub fn get_image_chunks(image_path: &String) -> Vec<PngChunk> {
    println!("[*] Reading all chunks from image: {}", image_path);

    let png_signature = b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A";
    let file = File::open(image_path).expect("[!] Failed to open image"); 
    let mut buf_reader = BufReader::new(file);

    // check the signature on the opened file
    let mut expected_signature_buf: [u32; 8] = [0u8; 8];
    buf_reader.read_exact(&mut signature_buf)?;
    assert_eq!(&png_signature, expected_signature_buf, "[!] PNG signature is invalid");

    // read all chunks from png into a vector of PngChunks
    let mut chunks: Vec<PngChunk> = vec![];
    chunks.push(read_single_chunk(&mut buf_reader)); // TODO: turn this into a loop
}



// open the image and store the greyscale values
pub fn get_png_greyscale_values(edited_img: &String) -> Vec<u8> {
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
pub fn get_png_rgb_values(original_img: &String) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
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
