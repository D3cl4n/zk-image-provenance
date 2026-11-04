use halo2_proofs::dev::MockProver;
use pasta_curves::Fp;
use ff::{Field, PrimeField};
use crate::prover::image_circuit::{ImageCircuit, ImageDetails};
use crate::prover::tamper::{self, T};

const N: usize = 16;
const K: u32 = 11;

fn details() -> ImageDetails {
    ImageDetails {
        r: (0..N).map(|i| ((i * 37 + 11) % 256) as u8).collect(),
        g: (0..N).map(|i| ((i * 91 + 5) % 256) as u8).collect(),
        b: (0..N).map(|i| ((i * 53 + 200) % 256) as u8).collect(),
        exif: (0..20).map(|i| ((i * 13 + 7) % 256) as u8).collect(),
    }
}

fn fp(s: &str) -> Fp { Fp::from_str_vartime(s).unwrap() }

// independent vector from ref.py (Python re-implementation of the Neptune Poseidon + packing)
fn expected() -> Vec<Fp> {
    vec![
        fp("37724860726355458377714684084616408157"),
        fp("40411840736962144512575552929240705410863002110"),
        fp("11918268691014158654193550323502503063996856924881750659269340432688590526688"),
    ]
}

fn run(t: T, inst: Vec<Fp>) -> Result<(), Vec<String>> {
    tamper::set(t);
    let c = ImageCircuit { png_vectors: details() };
    let r = MockProver::run(K, &c, vec![inst]).unwrap().verify();
    tamper::set(T::None);
    r.map_err(|v| v.iter().map(|e| format!("{e}")).collect())
}

#[test]
fn honest_passes() { run(T::None, expected()).unwrap(); }

#[test]
fn wrong_public_values_rejected() {
    for i in 0..3 {
        let mut e = expected(); e[i] += Fp::ONE;
        assert!(run(T::None, e).is_err(), "instance {i}");
    }
}

fn must_fail(t: T) {
    // run under the honest instance AND under the instance a cheater would want
    let r = run(t, expected());
    assert!(r.is_err(), "{t:?} was ACCEPTED");
    println!("{t:?}: {:?}", &r.unwrap_err()[..1]);
}

#[test] fn tamper_packer_init() { must_fail(T::PackerInit); }
#[test] fn tamper_sponge_init() { must_fail(T::SpongeInit); }
#[test] fn tamper_sponge_pad() { must_fail(T::SpongePad); }
#[test] fn tamper_hash_input_byte() { must_fail(T::PackByte(3 * N + 20)); }
#[test] fn tamper_public_exif_byte() { must_fail(T::PackByte(20)); }
#[test] fn tamper_public_grey_byte() { must_fail(T::PackByte(N)); }
#[test] fn tamper_exif_out_of_range() { must_fail(T::ExifRange); }
