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
    let original_img = String::from("original_256.png");
    let edited_img = String::from("greyscale.png");
    let public_key = String::from("verifying_key.bin");

    // verifier functionality off-circuit and prover circuit struct creation
    verifier::verify_ecdsa_signature(&edited_img, &public_key);
    let expected: Vec<Fp> = verifier::construct_expected_value(&edited_img);

    // start timer for measuring setup time
    let start_setup = Instant::now();

    // setup the proof generation
    let circuit = prover::construct_circuit_struct(&original_img);
    let k: u32 = 19;
    let params: Params<EqAffine> = Params::new(k);
    let vk = keygen_vk(&params, &circuit).expect("[!] keygen_vk failed");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("[!] keygen_pk failed");
    let end_setup = start_setup.elapsed();
    println!("[*] Setup runtime: {} ms", end_setup.as_millis());

    // vectors storing the prover and verifier runtimes as well as proof sizes
    let mut prover_runtimes = vec![];
    let mut verifier_runtimes = vec![];
    let mut proof_sizes = vec![];

    // for loop of 30 iterations to get accurate runtimes for prover and verifier
    for i in 0..30 {
        // fresh witness assignment per iteration of prover
        let circuit_trial = prover::construct_circuit_struct(&original_img);
        // creating the proof and starting the time for prover runtime measurement
        let start_prover = Instant::now();
        let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
        create_proof(&params, &pk, &[circuit_trial], &[&[&expected]], OsRng, &mut transcript).expect("[!] Proof generation failed");
        let proof = transcript.finalize();
        proof_sizes.push(proof.len());

        // end time for measuring prover time
        prover_runtimes.push(start_prover.elapsed().as_millis());

        // write the proof to a .bin file
        std::fs::write("proof.bin", &proof).expect("[!] Failed to write proof to file");

        // verify the proof and time the verifier 
        let start_verifier = Instant::now();
        let strategy = SingleVerifier::new(&params);
        let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
        verify_proof(&params, &vk, strategy, &[&[&expected]], &mut transcript).expect("[!] Proof verification failed");
        verifier_runtimes.push(start_verifier.elapsed().as_millis());
    }

    // print all the runtimes neatly
    for i in 0..30 {
        println!("[*] Trial {}: prover {} ms, verifier {} ms, proof size {} bytes", i, prover_runtimes[i], verifier_runtimes[i], proof_sizes[i]);
    }
}
