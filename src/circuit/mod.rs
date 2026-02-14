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
    width: u32,
    height: u32,
    num_pixels: u64,
    r_vals: Vec<u8>,
    g_vals: Vec<u8>,
    b_vals: Vec<u8>
}

// structure for the ciruit's greyscale chip config
#[derive(Clone, Debug)]
pub struct GreyscaleChipConfig {
    advice: [Column<Advice>; 4], // advice columns for: [r, g, b, g] values where g = greyscale(r, g, b)
    fixed: [Column<Fixed>; 3], // fixed column for each of the greyscale coefficients
    instance: Column<Instance> // public output
}