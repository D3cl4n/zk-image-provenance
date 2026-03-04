use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, TableColumn, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};
use crate::circuit::greyscale::{GreyscaleChipConfig};
use crate::circuit::poseidon::{PoseidonChipConfig};


// structure to store the image details
#[derive(Clone, Debug)]
pub struct ImageDetails {
    pub width: u32,
    pub height: u32,
    pub r: Vec<u8>,
    pub g: Vec<u8>,
    pub b: Vec<u8>
}


// struct for the image provenance circuit config as a whole (hash + greyscale)
pub struct ImageCircuitConfig {
    greyscale: GreyscaleChipConfig,
    poseidon: PoseidonChipConfig,
    instance: Column<Instance>
}


