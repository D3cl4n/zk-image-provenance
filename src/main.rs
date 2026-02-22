mod image;
mod circuit;


// main function
fn main() {
    // start with the MockProver and then move to real prover
    use halo2_proofs::dev::MockProver;
    use halo2curves::bls12381::Fr;
    
    // original image as private witness
    let original_img = String::from("/home/cdeclan/CryptoHack/image_provenance/image.png");

    // parse out rbg values into three vectors and store dimensions
    let mut width: u32 = 0u32;
    let mut height: u32 = 0u32;
    let mut r: Vec<u8> = vec![];
    let mut g: Vec<u8> = vec![];
    let mut b: Vec<u8> = vec![];
    (width, height, r, g, b) = image::get_image_rgb_values(&original_img);

    // construct ImageDetails structure from parsed values
    let original_img_details = circuit::ImageDetails {
        width, 
        height, 
        r, 
        g, 
        b
    };
}
