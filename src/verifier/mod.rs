use ff::PrimeField;
use crate::png;


// extract the grey pixels from the edited image as the expected value - pack 31 pixels per field element
fn pack_grey_pixels<F: PrimeField>(edited_img: &String) -> Vec<F> {
    let bytes_per_element: usize = 31;
    let grey_pixels: Vec<u8> = png::get_png_greyscale_values(edited_img);
    
    grey_pixels
        .chunks(bytes_per_element)
        .map(|chunk| {
            let mut element: F = F::ZERO;
            let mut base: F = F::ONE;
            let base_256: F = F::from(256 as u64);

            // iterate over each byte in slice and pack into position based on powers of 256
            for &byte in chunk {
                element += F::from(byte as u64) * base; // pack
                base *= base_256;
            } 

            element
        })
        .collect()
}

// extract the Poseidon(r||g||b||exif) hash from the edited image as part of the expected value
fn extract_hash_from_png<F: PrimeField>(edited_img: &String) -> F {
    let image_chunks: Vec<png::PngChunk> = png::get_image_chunks(edited_img);

    // extract the hash value from the chunks
    let hash: Vec<u8> = image_chunks.iter()
        .find(|c| &c.chunk_type == b"hASh")
        .expect("[!] No hASh chunk found")
        .chunk_data
        .clone();

    // pack bytes of hash into field element- reversed to match big endian
    let mut element: F = F::ZERO;
    let mut base: F = F::ONE;
    let base_256: F = F::from(256u64);

    for byte in hash.iter().rev() {
        element += F::from(*byte as u64) * base;
        base *= base_256;
    }

    element
}

// combine the hash and packed grey pixels into vector of expected field elements