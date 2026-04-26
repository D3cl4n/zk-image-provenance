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

    // prover functionality
    prover::construct_witness(&original_img);

    // verifier functionality
}
