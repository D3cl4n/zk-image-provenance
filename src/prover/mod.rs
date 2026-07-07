pub mod image_circuit;
pub mod poseidon;
pub mod greyscale;
pub mod sponge;
pub mod number;

use crate::png;


// construct the byte sequence for the private witness r||g||b||exif from original image
fn construct_witness(original_img: &String) -> image_circuit::ImageDetails {
    let (r, g, b): (Vec<u8>, Vec<u8>, Vec<u8>) = png::get_png_rgb_values(original_img);
    let image_chunks: Vec<png::PngChunk> = png::get_image_chunks(original_img);
    
    // extract the exif data from the chunks
    let exif_data: Vec<u8> = png::extract_chunk_data(original_img, b"eXIf");
    println!("[*] Exifdata length: {}", exif_data.len());

    image_circuit::ImageDetails {
        r, 
        g,
        b,
        exif: exif_data
    }
}


// construct the circuit structure for prover
pub fn construct_circuit_struct(original_img: &String) -> image_circuit::ImageCircuit {
    let png_vectors: image_circuit::ImageDetails = construct_witness(original_img);

    image_circuit::ImageCircuit {
        png_vectors
    }
}