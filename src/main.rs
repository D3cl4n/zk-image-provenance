mod png;
mod prover;
mod verifier;


// main function
fn main() {
    // start with the MockProver and then move to real prover
    use crate::prover::image_circuit::{ImageDetails, ImageCircuit};
    use halo2curves::bls12381::Fr;
    use halo2_proofs::{
        plonk::{create_proof, verify_proof, keygen_vk, keygen_pk},
        poly::commitment::Params,
        transcript::{Blake2bRead, Blake2bWrite, Challenge255}
    };
    use halo2_proofs::dev::MockProver;

    // original image as private witness, edited image as public, verifying key is public but used off-circuit
    let original_img = String::from("original.png");
    let edited_img = String::from("greyscale.png");
    let public_key = String::from("verifying_key.bin");

    // verifier functionality off-circuit and prover circuit struct creation
    verifier::verify_ecdsa_signature(&edited_img, &public_key);
    let expected: Vec<Fr> = verifier::construct_expected_value(&edited_img);
    let circuit = prover::construct_circuit_struct(&original_img);

    // setup the proof generation
    println!("[*] Generating IPA parameters");

    
    // MockProver for now
    let k: u32 = 15;
    let prover = MockProver::run(k, &circuit, vec![expected.clone()]).unwrap();
    assert_eq!(prover.verify(), Ok(()));
}
