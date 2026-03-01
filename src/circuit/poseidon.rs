use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Fixed, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};

// PARAMETERS TO USE TO MATCH PYTHON MODULE
// def case_neptune():
//     security_level = 128
//     input_rate = 3
//     t = 4
//     full_round = 8
//     partial_round = 56
//     alpha = 5
//     poseidon = OptimizedPoseidon(HashType.CONSTINPUTLEN, prime_255, security_level, alpha, input_rate, t,
//                                  full_round=full_round, partial_round=partial_round,
//                                  rc_list=round_constants_neptune, mds_matrix=matrix_neptune)
//     return poseidon, t

// prime_255 = 0x73EDA753299D7D483339D80809A1D80553BDA402FFFE5BFEFFFFFFFF00000001

// structure for the configuration for the poseidon permutation chip
#[derive(Clone, Debug)]
struct PoseidonChipConfig {
    advice: [Column<Advice>; 4],
    fixed: [Column<Fixed>; 4],
    instance: Column<Instance>,
    full_rounds: usize,
    partial_rounds: usize,
    s_add_rcs: Selector,
    s_sub_bytes_full: Selector,
    s_sub_bytes_partial: Selector,
    s_mix_layer: Selector,
}

// structure for the poseidon permutation chip
struct PoseidonChip<F: PrimeField> {
    config: PoseidonChipConfig,
    _marker: PhantomData<F>,
}

// structure to store numbers in cells
struct Number<F: PrimeField>(AssignedCell<F, F>);

// circuit structure
#[derive(Default)]
struct PoseidonCircuit<F: PrimeField> {
    s0: Value<F>, 
    s1: Value<F>, 
    s2: Value<F>,
    s3: Value<F>,
    full_rounds: usize,
    partial_rounds: usize
}

// implement the Chip trait for PoseidonChip
impl<F: PrimeField> Chip<F> for PoseidonChip<F> {
    type Config = PoseidonChipConfig;
    type Loaded = ();

    // getter for the chip config
    fn config(&self) -> &Self::Config {
        &self.config
    }

    // getter for the loaded field
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

// help function to create the ARC gate
fn create_arc_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 4], 
    fixed: [Column<Fixed>; 4], 
    s_add_rcs: Selector
) {
    meta.create_gate("ARC_Gate", |meta| {
        let s_add_rcs = meta.query_selector(s_add_rcs);
        let a0 = meta.query_advice(advice[0], Rotation::cur());
        let a1 = meta.query_advice(advice[1], Rotation::cur());
        let a2 = meta.query_advice(advice[2], Rotation::cur());
        let a3 = meta.query_advice(advice[3], Rotation::cur());
        let a0_next = meta.query_advice(advice[0], Rotation::next());
        let a1_next = meta.query_advice(advice[1], Rotation::next());
        let a2_next = meta.query_advice(advice[2], Rotation::next());
        let a3_next = meta.query_advice(advice[3], Rotation::next());
        let rc0 = meta.query_fixed(fixed[0]); // query_fixed reads from current row when gate is active
        let rc1 = meta.query_fixed(fixed[1]);
        let rc2 = meta.query_fixed(fixed[2]);
        let rc3 = meta.query_fixed(fixed[3]);

        // constraint should be vec![0, 0, 0, 0]
        vec![
            s_add_rcs.clone() * (a0_next - (a0 + rc0)), 
            s_add_rcs.clone() * (a1_next - (a1 + rc1)), 
            s_add_rcs.clone() * (a2_next - (a2 + rc2)),
            s_add_rcs * (a3_next - (a3 + rc3))
        ]
    });
}