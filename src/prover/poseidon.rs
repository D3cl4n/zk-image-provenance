use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, TableColumn, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};
use crate::prover::number::Number;


// chip configuration for Poseidon2 sponge and permutation
#[derive(Clone, Debug)]
pub struct Poseidon2ChipConfig {
    advice: [Column<Advice>; 31],   // 31 advice columns for a wide construction
    byte_table: TableColumn,        // lookup table to make sure bytes are within [0, 255]
    instance: Column<Instance>,     // for exposing public values
    s_init: Selector,               // selector for the prologue row: writing [0, 0, IV] into the t laes
    s_e1: Selector,                 // row 0 of a permutation (absorb fused into this gate)
    s_i: [Selector; 4],             // rows 1, 2, 3, 4 one per internal segment
    s_e2: Selector,                 // row 5
    s_digest: Selector              // last row for outputting digest value
}


// structure for the Poseidon2 chip that holds the configuration
#[derive(Clone, Debug)]
pub struct Poseidon2Chip<F: PrimeField> {
    config: Poseidon2ChipConfig,
    _marker: PhantomData<F>
}


// implement the Chip trait for the PoseidonChip
impl<F: PrimeField> Chip<F> for Poseidon2Chip<F> {
    type Config = Poseidon2ChipConfig;
    type Loaded = ();

    // getter for the configuration
    fn config(&self) -> &Self::Config {
        &self.config
    }

    // getter for the loaded field
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}


/*
* Gate Purpose: bake in the initial absorb for the t lanes to values [0, 0, IV] and constrain the values
* Columns used: adv0 -> lane0, adv1 -> lane1, adv2 -> lane2
* Constraints: [adv0_curr - 0, adv1_curr - 0, adv2_curr - IV]
*/
fn create_init_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 3],
    s_init: Selector
) {
    meta.create_gate("ps_init", |meta| {
        let s_init = meta.query_selector(s_init);
    });
}