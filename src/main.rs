mod image;
mod circuit;
mod utils;


// main function
fn main() {
    // start with the MockProver and then move to real prover
    use crate::circuit::image_circuit::{ImageDetails, ImageCircuit};
    use ff::PrimeField;
    use halo2curves::bls12381::Fr;
    use halo2_proofs::dev::MockProver;

    // original image as private witness
    let original_img = String::from("original.png");

    // parse out rbg values into three vectors
    let mut r: Vec<u8> = vec![];
    let mut g: Vec<u8> = vec![];
    let mut b: Vec<u8> = vec![];
    (r, g, b) = image::get_image_rgb_values(&original_img);

    // parse the exifdata section from the jpeg
    //let exif: Vec<u8> = image::get_image_exifdata(&original_img);
    let exif: Vec<u8> = vec![1u8, 2u8, 3u8];
    
    if exif.len() == 0 {
        println!("[!] exifdata not present or an error has occured");
    }

    else {
        println!("[+] exifdata: {:?}", exif);
    }

    // construct ImageDetails structure from parsed values
    let original_img_details = ImageDetails {
        r, 
        g, 
        b,
        exif
    };

    // read greyscale values from editor as the public output
    let greyscale_values_file = String::from("output/greyscale.txt");
    let y_values: Vec<u8> = utils::read_greyscale_values(&greyscale_values_file).expect("invalid byte val");

    // MockProver for now - testing
    let mut expected = vec![];
    for i in 0..y_values.len() { 
        expected.push(Fr::from(y_values[i] as u64)); // TODO: save instance columns by packing the greyscale result?
    }
    expected.push(Fr::from_str_vartime("11843851977609009639066496039218132672586495175272142737210069213891967648847").unwrap());
    
    // make and run the circuit
    let k: u32 = 22;
    let circuit = ImageCircuit {
        jpeg_vectors: original_img_details
    };

    println!("[+] Running MockProver");
    let prover = MockProver::run(k, &circuit, vec![expected.clone()]).unwrap();
    println!("[+] Running verifier");
    assert_eq!(prover.verify(), Ok(()));
}
