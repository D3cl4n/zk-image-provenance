use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, TableColumn, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};


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


// structure to store numbers in cells
#[derive(Clone)]
pub struct Number<F: PrimeField>(pub AssignedCell<F, F>);


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
    meta.create_gate("packing_accumulation_gate" |meta| {
        let s_pack = meta.query_selector(s_pack);
    })
}