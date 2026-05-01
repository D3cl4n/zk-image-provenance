use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, TableColumn, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};
use crate::prover::number::Number;


// structure for the ciruit's greyscale chip config
#[derive(Clone, Debug)]
pub struct GreyscaleChipConfig {
    advice: [Column<Advice>; 5], // advice columns for: [r, g, b, g] values where g = greyscale(r, g, b)
    pub byte_table: TableColumn, // one fixed column for byte values for lookups
    pub rem_table: TableColumn,
    instance: Column<Instance>,
    s_greyscale: Selector
}

// structure for the Greyscale chip
#[derive(Clone, Debug)]
pub struct GreyscaleChip<F: PrimeField> {
    config: GreyscaleChipConfig, 
    _marker: PhantomData<F>
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

// helper function to create the greyscale gate 
fn create_greyscale_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 5],
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
        let remainder = meta.query_advice(advice[4], Rotation::cur());

        // constants for greyscale formula coefficients
        let r_coeff = Expression::Constant(F::from(30));
        let g_coeff = Expression::Constant(F::from(58));
        let b_coeff = Expression::Constant(F::from(11));

        // greyscale value for constraint checks
        let sum = r_coeff.clone()*r.clone() + g_coeff.clone()*g.clone() + b_coeff.clone()*b.clone();

        // constraints
        vec![
            s_greyscale * ((Expression::Constant(F::from(100))*y + remainder) - sum)
        ]
    });
}

// implementation of additional methods for GreyscaleChip
impl<F: PrimeField> GreyscaleChip<F> {
    // constructor
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        GreyscaleChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates, constraints, and selectors
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 5],
        instance: Column<Instance>,
        byte_table: TableColumn,
        rem_table: TableColumn
    ) -> <Self as Chip<F>>::Config {

        let s_greyscale = meta.complex_selector();
        create_greyscale_gate(meta, advice, s_greyscale);

        meta.lookup(|meta| {
            let s = meta.query_selector(s_greyscale);
            let r = meta.query_advice(advice[0], Rotation::cur());
            vec![(s * r, byte_table)]
        });

        meta.lookup(|meta| {
            let s = meta.query_selector(s_greyscale);
            let g = meta.query_advice(advice[1], Rotation::cur());
            vec![(s * g, byte_table)]
        });

        meta.lookup(|meta| {
            let s = meta.query_selector(s_greyscale);
            let b = meta.query_advice(advice[2], Rotation::cur());
            vec![(s * b, byte_table)]
        });

        meta.lookup(|meta| {
            let s = meta.query_selector(s_greyscale);
            let y = meta.query_advice(advice[3], Rotation::cur());
            vec![(s * y, byte_table)]
        });

        meta.lookup(|meta| {
            let s = meta.query_selector(s_greyscale);
            let rem = meta.query_advice(advice[4], Rotation::cur());
            vec![(s * rem, rem_table)]
        });

        GreyscaleChipConfig {
            advice, 
            byte_table,
            rem_table, 
            instance,
            s_greyscale
        }
    }
}

// trait for sub-functions of the circuit
pub trait GreyscaleInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // expose output as public
    fn expose_as_public(
        &self, 
        layouter: &mut impl Layouter<F>, 
        num: Self::Num, 
        row: usize
    ) -> Result<(), Error>;

    // greyscale
    fn greyscale(
        &self, 
        layouter: impl Layouter<F>,
        r: &Vec<u8>,
        g: &Vec<u8>,
        b: &Vec<u8>
    ) -> Result<Vec<Self::Num>, Error>;
}

impl<F: PrimeField> GreyscaleInstructions<F> for GreyscaleChip<F> {
    type Num = Number<F>;

    // expose output as public
    fn expose_as_public(
        &self, 
        layouter: &mut impl Layouter<F>, 
        num: Self::Num, 
        row: usize
    ) -> Result<(), Error> {
        let config = self.config();
        layouter.constrain_instance(num.0.cell(), config.instance, row)
    }

    // greyscale transformation
    fn greyscale(
        &self, 
        mut layouter: impl Layouter<F>,
        r: &Vec<u8>,
        g: &Vec<u8>, 
        b: &Vec<u8>
    ) -> Result<Vec<Self::Num>, Error> {
        let config = self.config();

        // create a region for the lookup table
        layouter.assign_region(
            || "greyscale_region", |mut region| {
                let mut offset: usize = 0;

                // loop over r, g, b values and compute greyscale 
                let mut y_values: Vec<Self::Num> = vec![];
                for i in 0..r.len() {
                    // enable greyscale selector - triggering lookup constraint on all row values
                    config.s_greyscale.enable(&mut region, offset)?;

                    // greyscale computation for writing to fourth advice column
                    let r_curr = r[i] as u32;
                    let g_curr = g[i] as u32;
                    let b_curr = b[i] as u32;
                    let sum = 30 * r_curr + 58 * g_curr + 11 * b_curr;
                    let rem = sum % 100;
                    let y = (sum / 100) as u8;
                    
                    // writing unedited pixels to first three advice columns
                    region.assign_advice(
                        || "r_init", 
                        config.advice[0], 
                        offset, 
                        || Value::known(F::from(r[i] as u64))
                    )?;

                    region.assign_advice(
                        || "g_init", 
                        config.advice[1], 
                        offset, 
                        || Value::known(F::from(g[i] as u64))
                    )?;

                    region.assign_advice(
                        || "b_init", 
                        config.advice[2], 
                        offset, 
                        || Value::known(F::from(b[i] as u64))
                    )?;

                    // map greyscale value to field element and write to fourth advice column
                    let grey_cell = region.assign_advice(
                        || "y", 
                        config.advice[3], 
                        offset, 
                        || Value::known(F::from(y as u64))
                    )?;

                    region.assign_advice(
                        || "y", 
                        config.advice[4], 
                        offset, 
                        || Value::known(F::from(rem as u64))
                    )?;

                    // add cell to return vector
                    y_values.push(Number(grey_cell.clone()));

                    // increase row offset 
                    offset += 1;
                }

                // return value
                Ok(y_values)
            }
        )
    }
}