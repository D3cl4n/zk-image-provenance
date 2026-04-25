mod png;
mod prover;
mod verifier;


// main function
fn main() {
    // start with the MockProver and then move to real prover
    use crate::prover::image_circuit::{ImageDetails, ImageCircuit};
    use ff::PrimeField;
    use halo2curves::bls12381::Fr;
    use halo2_proofs::dev::MockProver;

    // original image as private witness, edited image as public
    let original_img = String::from("original.png");
    let edited_img = String::from("greyscale.png");

    // parse out rbg values into three vectors
    let mut r: Vec<u8> = vec![];
    let mut g: Vec<u8> = vec![];
    let mut b: Vec<u8> = vec![];
    (r, g, b) = png::get_image_rgb_values(&original_img);

    // parse greyscale values from edited image
    let grey_values: Vec<u8> = png::get_image_greyscale_values(&edited_img);

    // parse the exifdata section from the png
    let exif: Vec<u8> = png::get_image_exifdata(&original_img); // TODO: change this to reference all chunks from the original image

    // construct ImageDetails structure from parsed values
    let original_img_details = ImageDetails {
        r, 
        g, 
        b,
        exif
    };

    // MockProver for now - testing
    let mut expected = vec![];
    for i in 0..grey_values.len() { 
        expected.push(Fr::from(grey_values[i] as u64));
    }
    expected.push(Fr::from_str_vartime("3566011618332057031339743203502844274745506337811929250184210502630497826651").unwrap());
    
    // make and run the circuit
    let k: u32 = 22;
    let circuit = ImageCircuit {
        png_vectors: original_img_details
    };

    println!("[+] Running MockProver");
    let prover = MockProver::run(k, &circuit, vec![expected.clone()]).unwrap();
    println!("[+] Running verifier");
    assert_eq!(prover.verify(), Ok(()));
}
