use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector, Expression},
    poly::Rotation,
};
use crate::prover::number::Number;


// structure for the packing chip config
#[derive(Clone, Debug)]
pub struct PackingChipConfig {
    advice: [Column<Advice>; 2], // one advice column for the accumulator (encodes position) one column for the byte being packed
    s_pack: Selector
}


// structure for the packing chip
#[derive(Clone, Debug)]
pub struct PackingChip<F: PrimeField> {
    config: PackingChipConfig,
    _marker: PhantomData<F>
}


// implement the chip trait for PackingChip
impl<F: PrimeField> Chip<F> for PackingChip<F> {
    type Config = PackingChipConfig;
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


// helper function to create the packing gate (accumulator_{i+1} = accumulator_{i}*256+byte)
fn create_packing_accumulation_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 2],
    s_pack: Selector
) {
    meta.create_gate("packing_accumulation_gate", |meta| {
        let s_pack = meta.query_selector(s_pack);
        let accumulator_curr = meta.query_advice(advice[0], Rotation::cur());
        let accumulator_next = meta.query_advice(advice[0], Rotation::next());
        let byte = meta.query_advice(advice[1], Rotation::cur());

        vec![
            s_pack * (accumulator_next - (accumulator_curr * Expression::Constant(F::from(256 as u64)) + byte))
        ]
    });
}


// implementation of additional methods for the PackingChip
// TODO: figure out if I need a lookup table here to constrain byte values
impl<F: PrimeField> PackingChip<F> {
    // constructor
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        PackingChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates, constraints and selectors
    pub fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 2]
    ) -> <Self as Chip<F>>::Config {
        let s_pack = meta.selector();
        create_packing_accumulation_gate(meta, advice, s_pack);

        PackingChipConfig {
            advice,
            s_pack
        }
    }
}


// trait for sub-functions of this chip
pub trait PackingChipInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // function signature for pack
    fn pack(
        &self, 
        layouter: &mut impl Layouter<F>,
        bytes: &Vec<Self::Num>
    ) -> Result<Vec<Number<F>>, Error>;
}


// implement the PackingChipInstructions trait for PackingChip
// TODO: debug this
impl<F: PrimeField> PackingChipInstructions<F> for PackingChip<F> {
    type Num = Number<F>;

    // pack function definition
    fn pack(
        &self,
        layouter: &mut impl Layouter<F>, 
        bytes: &Vec<Self::Num>
    ) -> Result<Vec<Number<F>>, Error> {
        let config = self.config();
        let bytes_per_element: usize = 31;
        let mut packed_elements: Vec<Number<F>> = vec![];
    
        layouter.assign_region(|| "packing_region", |mut region| {
            let mut row_offset = 0;
            let base_256: F = F::from(256 as u64);
            // iterate over each chunk - adds one packed element to packed_elements vector each iterations
            for chunk in bytes.chunks(bytes_per_element) {
                let mut last_cell = None;
                let mut accumulator_val: Value<F> = Value::known(F::ZERO);
                // iterate over each of the 31 bytes in the chunk and pack
                for byte in chunk.iter() {
                    config.s_pack.enable(&mut region, row_offset)?;
                    let accumulator_cell: AssignedCell<F, F> = region.assign_advice(|| "acc_curr", config.advice[0], row_offset, || accumulator_val)?;
                    let byte_cell: AssignedCell<F, F> = region.assign_advice(|| "byte_cell", config.advice[1], row_offset, || byte.0.value().copied())?;
                    let accumulator_next: Value<F> = accumulator_cell.value().zip(byte_cell.value()).map(|(a, b)| *a * base_256 + *b);
                    let accumulator_next_cell: AssignedCell<F, F> = region.assign_advice(|| "acc_next", config.advice[0], row_offset + 1, || accumulator_next)?;
                    last_cell = Some(accumulator_next_cell);
                    accumulator_val = accumulator_next;
                    row_offset += 1;
                }
                packed_elements.push(Number(last_cell.expect("[!] Fauled to unwrap")));
            }
            Ok(())
        })?; // end of region assignment
        Ok(packed_elements)
    } // end of the pack function need to return Vec<Number<F>> above here
} // end of implementation