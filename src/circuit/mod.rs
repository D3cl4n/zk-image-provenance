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
    fixed: [Column<Fixed>; 1], // one fixed column for byte values for lookups
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
    b_elements: Vec<Value<F>>,
    y_elements: Vec<Value<F>>
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

// helper function to create the greyscale gate TODO: ADD RANGE CHECKS ON r,g,b,y for 0, ..., 255
fn create_greyscale_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 4],
    fixed: Column<Fixed>,
    s_greyscale: Selector
) {
    meta.create_gate("greyscale_gate", |meta| {
        let s_greyscale = meta.query_selector(s_greyscale);
        // current rgb values
        let r = meta.query_advice(advice[0], Rotation::cur());
        let g = meta.query_advice(advice[1], Rotation::cur());
        let b = meta.query_advice(advice[2], Rotation::cur());

        // greyscaled values from formula y = (30*r + 58*g + 11*b)/100
        let y = meta.query_advice(advice[3], Rotation::cur());

        // constants for greyscale formula coefficients
        let r_coeff = Expression::Constant(F::from(30));
        let g_coeff = Expression::Constant(F::from(58));
        let b_coeff = Expression::Constant(F::from(11));

        // greyscale value for constraint checks
        let sum = r_coeff.clone()*r.clone() + g_coeff.clone()*g.clone() + b_coeff.clone()*b.clone();

        // constraints
        vec![
            s_greyscale * (Expression::Constant(F::from(100))*y - sum) // enforce 100r' = 100g' = 100b' = 30r+58g+11b
        ]
    });
}

