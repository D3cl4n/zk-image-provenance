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
    s_init: Selector,               // ensures correct starting values at start of permutation
    s_e1: Selector,                 // selector computes RF/2 external rounds in one row (row 0)
    s_i: [Selector; 4],             // each selector computes 14 RP internal rounds giving us 14*4 = 56 internal rounds in 4 rows 
    s_e2: Selector,                 // selector computes the last RF/2 external rounds in one row (row 5)
    s_digest: Selector              // 
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


// helper function to query an advice column at a rotation
fn adv<F: PrimeField>(meta: &mut VirtualCells<F>, col: Column<Advice>, rot: Rotation)
    -> Expression<F> { meta.query_advice(col, rot) }


// helper function for 


/*
* Gate Purpose: bake in the initial absorb for the t lanes to values [0, 0, IV] and constrain the values
* Columns used: adv0 -> lane0, adv1 -> lane1, adv2 -> lane2
* Constraints: [adv0_curr - 0, adv1_curr - 0, adv2_curr - IV]
*/
fn create_init_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 3], // pass A19, A21, A23 here 
    s_init: Selector
) {
    meta.create_gate("ps_init", |meta| {
        let s_init = meta.query_selector(s_init);
    });
}


/*
* Gate Purpose: compute RF/2 external rounds in the 2nd row using the e1 selector
* Columns used: A0, ..., A23 for the 24 cells of SB q, b values
* Constraints
*/
fn create_e1_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 31],
    s_e1: Selector,
    rc: [[F; 3]; 4] // pass the round constants needed (3 per round, 4 rounds total)
) {
    meta.create_gate("e1_gate", |meta| {
        let s_e1 = meta.query_selector(s_e1);
        let b_prev = [
            adv(advice[19], Rotation::prev()), 
            adv(advice[21], Rotation::prev()), 
            adv(advice[23], Rotation::prev())];
        let m0 = adv(meta, advice[24], Rotation::cur());
        let m1 = adv(meta, advice[25], Rotation::cur());
    });
}