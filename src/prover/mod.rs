pub mod sponge;
pub mod number;
pub mod poseidon;
pub mod packer;
pub mod greyscale;
pub mod crop;

use crate::png;


// structure for the circuit which is returned to create_proof(params, pk, circuits, instances, rng, transcript)
pub struct ImageCircuit {
    idat: Vec<u8>,
    exif: Vec<u8>,
    idat_edited: Vec<u8>,
    exif_edited: Vec<u8>
}