mod png;
mod prover;
mod verifier;


// main function
fn main() {
    use std::time::{Instant, Duration};
    use crate::prover::image_circuit::{ImageDetails, ImageCircuit};
    use halo2_proofs::{
        pasta::{EqAffine, Fp},
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier},
        poly::commitment::Params,
        transcript::{Blake2bRead, Blake2bWrite, Challenge255},
    };
    use rand::rngs::OsRng;

    // original image as private witness, edited image as public, verifying key is public but used off-circuit
    let original_img = String::from("photos/original_64.png");
    let edited_img = String::from("photos/greyscale.png");
    let public_key = String::from("verifying_key.bin");

    // verifier functionality off-circuit and prover circuit struct creation
    verifier::verify_ecdsa_signature(&edited_img, &public_key);
}



