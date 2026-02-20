use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, TableColumn, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
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
    table: TableColumn, // one fixed column for byte values for lookups
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

// structure to store numbers in cells
#[derive(Clone)]
struct Number<F: PrimeField>(AssignedCell<F, F>);

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

// implementation of additional methods for GreyscaleChip
impl<F: PrimeField> GreyscaleChip<F> {
    // constructor
    fn construct(config: <Self as Chip<F>>::Config) -> Self {
        GreyscaleChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates, constraints, and selectors
    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 4],
        table: TableColumn,
        instance: Column<Instance>,
    ) -> <Self as Chip<F>>::Config {
        meta.enable_equality(instance);

        for column in &advice {
            meta.enable_equality(*column);
        }

        let s_greyscale = meta.selector();
        create_greyscale_gate(meta, advice, s_greyscale);

        // lookups for byte range checks, since we don't use a selector it applies to every row
        meta.lookup(|meta| {
            let r = meta.query_advice(advice[0], Rotation::cur());
            vec![(r, table)]
        });

        meta.lookup(|meta| {
            let g = meta.query_advice(advice[1], Rotation::cur());
            vec![(g, table)]
        });

        meta.lookup(|meta| {
            let b = meta.query_advice(advice[2], Rotation::cur());
            vec![(b, table)]
        });

        meta.lookup(|meta| {
            let y = meta.query_advice(advice[3], Rotation::cur());
            vec![(y, table)]
        });

        GreyscaleChipConfig {
            advice, 
            table, 
            instance, 
            s_greyscale
        }
    }
}

// trait for sub-functions of the circuit
trait GreyscaleInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // expose a value as public
    fn expose_as_public(&self, layouter: impl Layouter<F>, num: Self::Num, row: usize) -> Result<(), Error>;

    // greyscale
    fn greyscale(
        &self, 
        layouter: impl Layouter<F>,
        r: Vec<u8>,
        g: Vec<u8>,
        b: Vec<u8>
    ) -> Result<Vec<u8>, Error>;
}

impl<F: PrimeField> GreyscaleInstructions<F> for GreyscaleChip<F> {
    type Num = Number<F>;

    // expose a value as public in the instance column
    fn expose_as_public(&self, mut layouter: impl Layouter<F>, num: Self::Num, row: usize) -> Result<(), Error> {
        let config = self.config();
        layouter.constrain_instance(num.0.cell(), config.instance, row)
    }

    // greyscale transformation
    fn greyscale(
        &self, 
        mut layouter: impl Layouter<F>,
        r: Vec<u8>,
        g: Vec<u8>, 
        b: Vec<u8>
    ) -> Result<Vec<u8>, Error> {
        let config = self.config();

        // create a region for the lookup table
        layouter.assign_region(
            || "lookup_table_region", |mut region| {
                let mut offset: usize = 0;
                // default return value
                Ok(vec![0u8])
            }
        )
    }
}