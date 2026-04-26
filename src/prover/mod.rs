pub mod image_circuit;
pub mod poseidon;
pub mod greyscale;
pub mod sponge;

use crate::png;


// construct the byte sequence for the private witness r||g||b||exif from original image
pub fn construct_witness(original_img: &String) -> Vec<u8> {
    let mut witness: Vec<u8> = vec![];
    let rgb: (Vec<u8>, Vec<u8>, Vec<u8>) = png::get_png_rgb_values(original_img);

    witness
}


// construct the circuit structure for prover
// fn construct_circuit_struct() -> ImageCircuit {

//}