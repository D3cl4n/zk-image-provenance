pub mod image_circuit;
pub mod poseidon;
pub mod greyscale;
pub mod sponge;

use crate::png;


// construct the byte sequence for the private witness r||g||b||exif from original image
pub fn construct_witness(original_img: &String) -> Vec<u8> {
    let (r, g, b): (Vec<u8>, Vec<u8>, Vec<u8>) = png::get_png_rgb_values(original_img);
    let image_chunks: Vec<png::PngChunk> = png::get_image_chunks(original_img);
    
    // extract the exif data from the chunks
    let exif_data: Vec<u8> = image_chunks.iter()
        .find(|c| &c.chunk_type == b"eXIf")
        .expect("[!] No eXIf chunk found")
        .chunk_data
        .clone();

    let mut witness = vec![];
    witness.extend(r);
    witness.extend(g);
    witness.extend(b);
    witness.extend(exif_data);

    witness
}


// construct the circuit structure for prover
// fn construct_circuit_struct() -> ImageCircuit {

//}