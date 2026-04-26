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
    let circuit = prover::construct_circuit_struct(&original_img);

    // verifier functionality
    let expected: Vec<Fr> = verifier::construct_expected_value(&edited_img);
}
