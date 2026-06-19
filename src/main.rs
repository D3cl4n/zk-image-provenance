mod png;
mod prover;
mod verifier;


// main function
fn main() {
    // start with the MockProver and then move to real prover
    use crate::prover::image_circuit::{ImageDetails, ImageCircuit};
    use halo2_proofs::{
        pasta::{EqAffine, Fp},
        plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier},
        poly::commitment::Params,
        transcript::{Blake2bRead, Blake2bWrite, Challenge255},
    };
    use rand::rngs::OsRng;

    // original image as private witness, edited image as public, verifying key is public but used off-circuit
    let original_img = String::from("original_exif_swap.png");
    let edited_img = String::from("greyscale.png");
    let public_key = String::from("verifying_key.bin");

    // verifier functionality off-circuit and prover circuit struct creation
    verifier::verify_ecdsa_signature(&edited_img, &public_key);
    let expected: Vec<Fp> = verifier::construct_expected_value(&edited_img);
    let circuit = prover::construct_circuit_struct(&original_img);

    // setup the proof generation
    println!("[*] Generating IPA parameters");
    let k: u32 = 15;
    let params: Params<EqAffine> = Params::new(k);
    let vk = keygen_vk(&params, &circuit).expect("[!] keygen_vk failed");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("[!] keygen_pk failed");

    // creating the proof
    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
    create_proof(&params, &pk, &[circuit], &[&[&expected]], OsRng, &mut transcript).expect("[!] Proof generation failed");
    let proof = transcript.finalize();
    println!("[*] Proof size: {}", proof.len());

    // write the proof to a .bin file
    std::fs::write("proof.bin", &proof).expect("[!] Failed to write proof to file");

    // verify the proof
    println!("[*] Verifying proof");
    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
    verify_proof(&params, &vk, strategy, &[&[&expected]], &mut transcript).expect("[!] Proof verification failed");

    println!("[*] Proof verified successfully");
}
