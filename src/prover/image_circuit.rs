use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error,},
};
use crate::prover::greyscale::{GreyscaleChip, GreyscaleChipConfig, Number as GreyscaleNumber, GreyscaleInstructions};
use crate::prover::poseidon::{PoseidonChip, PoseidonChipConfig, PermutationInstructions};
use crate::prover::sponge::{SpongeChip, SpongeChipConfig, Number as SpongeNumber, SpongeInstructions};


// structure to store the image details
#[derive(Clone, Debug, Default)]
pub struct ImageDetails {
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>,
    pub exif: Vec<u8>
}

// structure storing all chip configurations
#[derive(Clone)]
pub struct ImageCircuitConfig<F: PrimeField> {
    pub greyscale: GreyscaleChipConfig,
    pub poseidon: PoseidonChipConfig<F>,
    pub sponge: SpongeChipConfig,
    pub packer: PackingChipConfig
}

// struct for the image provenance circuit config as a whole (hash + greyscale)
#[derive(Default)]
pub struct ImageCircuit {
    pub png_vectors: ImageDetails
}

// implement the Circuit trait for ImageCircuit
impl<F: PrimeField> Circuit<F> for ImageCircuit {
    type Config = ImageCircuitConfig<F>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    // configure function for the circuit
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = [meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column()];
        let fixed = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        let instance = meta.instance_column();
        let byte_table = meta.lookup_table_column();
        let rem_table = meta.lookup_table_column();

        for column in &advice {
            meta.enable_equality(*column);
        }

        for column in &fixed {
            meta.enable_constant(*column);
        }

        meta.enable_equality(instance);

        // construct the GreyscaleChipConfig, PoseidonChipConfig, and SpongeChipConfig
        ImageCircuitConfig {
            greyscale: GreyscaleChip::configure(meta, advice, instance, byte_table, rem_table),
            poseidon: PoseidonChip::configure(meta, [advice[0], advice[1], advice[2], advice[3]], fixed),
            sponge: SpongeChip::configure(meta, [advice[0], advice[1], advice[2], advice[3]], instance),
            packer: PackingChip::configure(meta, [advice[0], advice[1]])
        }
    }

    // synthesize the circuit
    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let greyscale_chip = GreyscaleChip::construct(config.greyscale.clone());
        let poseidon_chip = PoseidonChip::construct(config.poseidon.clone());
        let sponge_chip: SpongeChip<F> = SpongeChip::construct(config.sponge.clone());
        let packing_chip: PackingChip = PackingChip::construct(config.packer.clone());

        // // populate the lookup table for constraining pixel values to bytes (0-255)
        layouter.assign_table(
            || "byte_table", |mut table| {
                for i in 0..256 {
                    table.assign_cell(
                        || "byte_val",
                        config.greyscale.byte_table,
                        i, 
                        || Value::known(F::from(i as u64))
                    )?;
                }
                Ok(())
            }
        )?;

        // populate the remainder lookup table for constraining remainder values from [0, 99]
        layouter.assign_table(
            || "remainder_table", |mut table| {
                for i in 0..100 {
                    table.assign_cell(
                        || "byte_val",
                        config.greyscale.rem_table,
                        i, 
                        || Value::known(F::from(i as u64))
                    )?;
                }
                Ok(())
            }
        )?;

        let greyscale_result = greyscale_chip.greyscale(
            layouter.namespace(|| "greyscale_namespace"),
            &self.png_vectors.r,
            &self.png_vectors.g,
            &self.png_vectors.b
        )?;

        // pack the grey pixels into field elements inside circuit so it is constrained properly
        let mut result: Vec<SpongeNumber<F>> = vec![];

        // use the packing chip to pack greyscale results
        let packed_elements = packing_chip.pack(&mut layouter, &greyscale_result)?;
        for p in packed_pixels {
            result.push(SpongeNumber(p.0));
        }

        // compute Poseidon(r||g||b||exif) using the sponge and permutation chips
        let preimage: Vec<[Value<F>; 3]> = sponge_chip.pad(
            &self.png_vectors.r, 
            &self.png_vectors.g, 
            &self.png_vectors.b, 
            &self.png_vectors.exif,
            3
        )?;

        // compute the hash and expose as public 
        let mut state: [AssignedCell<F, F>; 4] = sponge_chip.initialize(&mut layouter)?;
        for i in 0..preimage.len() {
            // absorb the input block 
            state = sponge_chip.absorb(
                &mut layouter,
                state,
                preimage[i]
            )?;

            // compute the permutation once the block is absobed
            state = poseidon_chip.permute(
                &mut layouter,
                state
            )?;
        }

        // squeeze once all blocks are permuted and expose as public
        let digest_cell: AssignedCell<F, F> = sponge_chip.squeeze(state)?;
        // print the digest here for debugging
        println!("[+] Hash: {:?}", digest_cell.value().copied());
        result.push(SpongeNumber(digest_cell));

        // expose each field element in the result vector
        // for i in 0..result.len() {
        //     sponge_chip.expose_as_public(&mut layouter, result[i].clone(), i)?;
        // }

        Ok(())
    }
}

