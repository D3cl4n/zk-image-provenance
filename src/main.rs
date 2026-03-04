mod image;
mod circuit;
mod utils;


// main function
fn main() {
    // start with the MockProver and then move to real prover
    use crate::circuit::image_circuit::{ImageDetails};
    use halo2curves::bls12381::Fr;

    // original image as private witness
    let original_img = String::from("image.png");

    // parse out rbg values into three vectors and store dimensions
    let mut width: u32 = 0u32;
    let mut height: u32 = 0u32;
    let mut r: Vec<u8> = vec![];
    let mut g: Vec<u8> = vec![];
    let mut b: Vec<u8> = vec![];
    (width, height, r, g, b) = image::get_image_rgb_values(&original_img);

    // construct ImageDetails structure from parsed values
    let original_img_details = ImageDetails {
        width, 
        height, 
        r, 
        g, 
        b
    };

    // read greyscale values from editor as the public output
    let greyscale_values_file = String::from("output/greyscale.txt");
    let y_values: Vec<u8> = utils::read_greyscale_values(&greyscale_values_file).expect("invalid byte val");

    // MockProver for now - testing
    let mut expected: Vec<Fr> = vec![];
    for i in 0..y_values.len() {
        expected.push(Fr::from(y_values[i] as u64));
    }
}
