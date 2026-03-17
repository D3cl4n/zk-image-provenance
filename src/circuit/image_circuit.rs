use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, TableColumn, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};
use crate::circuit::greyscale::{GreyscaleChip, GreyscaleChipConfig, Number, GreyscaleInstructions};
use crate::circuit::poseidon::{PoseidonChip, PoseidonChipConfig, PermutationInstructions};
use crate::circuit::sponge::{SpongeChip, SpongeChipConfig, SpongeInstructions};


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
    jpeg_vectors: ImageDetails
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
        let advice = [meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column()];
        let fixed = [meta.fixed_column(), meta.fixed_column(), meta.fixed_column(), meta.fixed_column()];
        let instance = meta.instance_column();
        let table = meta.lookup_table_column();

        for column in &advice {
            meta.enable_equality(*column);
        }

        for column in &fixed {
            meta.enable_constant(*column);
        }

        meta.enable_equality(instance);

        // construct the GreyscaleChipConfig, PoseidonChipConfig, and SpongeChipConfig
        ImageCircuitConfig {
            greyscale: GreyscaleChip::configure(meta, advice, instance, table),
            poseidon: PoseidonChip::configure(meta, advice, fixed),
            sponge: SpongeChip::configure(meta, advice, instance)
        }
    }

    // synthesize the circuit
    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let greyscale_chip = GreyscaleChip::construct(config.greyscale.clone());
        let poseidon_chip = PoseidonChip::construct(config.poseidon.clone());
        let sponge_chip: SpongeChip<F> = SpongeChip::construct(config.sponge.clone());

        // populate the lookup table for constraining pixel values to bytes (0-255)
        layouter.assign_table(
            || "lookup_table", |mut table| {
                for i in 0..256 {
                    table.assign_cell(
                        || "byte_val",
                        config.greyscale.table,
                        i, 
                        || Value::known(F::from(i as u64))
                    )?;
                }
                Ok(())
            }
        )?;

        let greyscale_result = greyscale_chip.greyscale(
            layouter.namespace(|| "greyscale_namespace"),
            &self.jpeg_vectors.r,
            &self.jpeg_vectors.g,
            &self.jpeg_vectors.b
        )?;

        // expose the greyscale pixel values as public
        for i in 0..greyscale_result.len() {
            let grey_pixel = Number(greyscale_result[i].clone());
            greyscale_chip.expose_as_public(&mut layouter.namespace(|| "grey_pixel"), grey_pixel, i)?;
        }

        // compute Poseidon(r||g||b||exif) using the sponge and permutation chips
        let preimage: Vec<[Value<F>; 3]> = sponge_chip.pad(
            &self.jpeg_vectors.r, 
            &self.jpeg_vectors.g, 
            &self.jpeg_vectors.b, 
            &self.jpeg_vectors.exif,
            3 as usize
        )?;

        // compute the hash and expose as public 
        // TODO: figure out if I should be reusing layouters??
        let initial_state: [AssignedCell<F, F>; 4] = sponge_chip.initialize(&mut layouter.namespace(|| "sponge_init"))?;
        for i in 0..preimage.len() {

        }

        Ok(())
    }
}

