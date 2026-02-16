use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Fixed, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};


// structure to store the image details
#[derive(Clone, Debug)]
pub struct ImageDetails {
    pub width: u32,
    pub height: u32,
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>
}

// structure for the ciruit's greyscale chip config
#[derive(Clone, Debug)]
struct GreyscaleChipConfig {
    advice: [Column<Advice>; 4], // advice columns for: [r, g, b, g] values where g = greyscale(r, g, b)
    fixed: [Column<Fixed>; 3], // fixed column for each of the greyscale coefficients
    instance: Column<Instance>, // public output
    s_greyscale: Selector
}

// structure for the Greyscale chip
#[derive(Clone, Debug)]
struct GreyscaleChip<F: PrimeField> {
    config: GreyscaleChipConfig, 
    _marker: PhantomData<F>
} 

// structure for the Greyscale circuit
#[derive(Default)]
struct GreyscaleCircuit<F: PrimeField> {
    r_elements: Vec<Value<F>>,
    g_elements: Vec<Value<F>>, 
    b_elements: Vec<Value<F>>
}

// implement the chip trait for GreyscaleChip
impl<F: PrimeField> Chip<F> for GreyscaleChip<F> {
    type Config = GreyscaleChipConfig;
    type Loaded = ();

    // getter for the chip config 
    fn config(&self) -> &Self::Config {
        &self.config
    }

    // getter for the Loaded field
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}