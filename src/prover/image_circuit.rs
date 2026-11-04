use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, ConstraintSystem, Error,},
};

use crate::prover::number::Number;
use crate::prover::greyscale::{GreyscaleChip, GreyscaleChipConfig, GreyscaleInstructions};
use crate::prover::poseidon::{PoseidonChip, PoseidonChipConfig, PermutationInstructions};
use crate::prover::sponge::{SpongeChip, SpongeChipConfig, SpongeInstructions};


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
    pub sponge: SpongeChipConfig
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
        let advice = [meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column()];
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
            greyscale: GreyscaleChip::configure(meta, [advice[0], advice[1], advice[2], advice[3], advice[4]], instance, byte_table, rem_table),
            poseidon: PoseidonChip::configure(meta, [advice[0], advice[1], advice[2], advice[3]], fixed),
            sponge: SpongeChip::configure(meta, [advice[0], advice[1], advice[2], advice[3]], instance),
            packer: PackingChip::configure(meta, [advice[5], advice[6]], instance, byte_table)
        }
    }

    // synthesize the circuit
    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let greyscale_chip = GreyscaleChip::construct(config.greyscale.clone());
        let poseidon_chip = PoseidonChip::construct(config.poseidon.clone());
        let sponge_chip: SpongeChip<F> = SpongeChip::construct(config.sponge.clone());

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

        let grey = greyscale_chip.greyscale(
            layouter.namespace(|| "greyscale_namespace"),
            &self.png_vectors.r,
            &self.png_vectors.g,
            &self.png_vectors.b
        )?;

        // range-checked EXIF byte cells: used BOTH for the public packed EXIF and the hash input
        let exif_cells: Vec<Number<F>> = packing_chip.load_bytes(&mut layouter, &self.png_vectors.exif)?;

        // public outputs: pack(grey) || pack(exif) || digest
        let mut result: Vec<Number<F>> = vec![];
        result.extend(packing_chip.pack(&mut layouter, &grey.y)?);
        result.extend(packing_chip.pack(&mut layouter, &exif_cells)?);

        // hash input stream r||g||b||exif built from the SAME cells (copy constraints)
        let mut stream: Vec<Number<F>> = vec![];
        stream.extend(grey.r.iter().cloned());
        stream.extend(grey.g.iter().cloned());
        stream.extend(grey.b.iter().cloned());
        stream.extend(exif_cells.iter().cloned());
        let hash_input: Vec<AssignedCell<F, F>> = packing_chip
            .pack(&mut layouter, &stream)?
            .into_iter().map(|n| n.0).collect();

        let preimage = sponge_chip.pad(&mut layouter, hash_input, 3)?;

        let mut state: [AssignedCell<F, F>; 4] = sponge_chip.initialize(&mut layouter)?;
        for block in preimage.into_iter() {
            state = sponge_chip.absorb(&mut layouter, state, block)?;
            state = poseidon_chip.permute(&mut layouter, state)?;
        }
        let digest_cell: AssignedCell<F, F> = sponge_chip.squeeze(state)?;
        result.push(Number(digest_cell));

        for i in 0..result.len() {
            greyscale_chip.expose_as_public(&mut layouter, result[i].clone(), i)?;
        }

        Ok(())
    }
}

