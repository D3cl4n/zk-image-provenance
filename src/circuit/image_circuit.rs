use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, TableColumn, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};
use crate::circuit::greyscale::{GreyscaleChip, GreyscaleChipConfig};
use crate::circuit::poseidon::{PoseidonChip, PoseidonChipConfig};
use crate::circuit::sponge::{SpongeChip, SpongeChipConfig};


// structure to store the image details
#[derive(Clone, Debug, Default)]
pub struct ImageDetails {
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>
}

// structure storing all chip configurations
#[derive(Clone)]
pub struct ImageCircuitConfig<F> {
    pub greyscale: GreyscaleChipConfig,
    pub poseidon: PoseidonChipConfig<F>,
    pub sponge: SpongeChipConfig
}

// struct for the image provenance circuit config as a whole (hash + greyscale)
#[derive(Default)]
pub struct ImageCircuit {
    image_vectors: ImageDetails
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
            greyscale: GreyscaleChip::configure(meta, advice, table),
            poseidon: PoseidonChip::configure(meta, advice, fixed),
            sponge: SpongeChip::configure(meta, advice)
        }

    }
}

