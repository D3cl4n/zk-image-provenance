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