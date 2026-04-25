pub mod image_circuit;
pub mod poseidon;
pub mod greyscale;
pub mod sponge;


// construct the byte sequence for the private witness r||g||b||exif from original image
pub fn extract_satisfying_witness(original_img: &String) -> Vec<u8> {
    vec![0u8]
}


// construct the circuit structure for prover