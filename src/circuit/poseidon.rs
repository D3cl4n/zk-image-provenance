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
pub struct PoseidonChipConfig {
    advice: [Column<Advice>; 4],
    fixed: [Column<Fixed>; 4],
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

// helper function to create the MDS multiplication gate
fn create_mds_mul_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 4], 
    s_mds_mul: Selector,
    mds: &[[F; 4]; 4]
) {
    meta.create_gate("ML_gate", |meta| {
        let s_mds_mul = meta.query_selector(s_mds_mul);
        let a0 = meta.query_advice(advice[0], Rotation::cur());
        let a1 = meta.query_advice(advice[1], Rotation::cur());
        let a2 = meta.query_advice(advice[2], Rotation::cur());
        let a3 = meta.query_advice(advice[3], Rotation::cur());
        let a0_next = meta.query_advice(advice[0], Rotation::next());
        let a1_next = meta.query_advice(advice[1], Rotation::next());
        let a2_next = meta.query_advice(advice[2], Rotation::next());
        let a3_next = meta.query_advice(advice[3], Rotation::next());

        // MDS matrix elements from row in column 0 -> column 2 order, use Expression:Constant to embed into polynomial
        let mds_0_0 = Expression::Constant(mds[0][0]);
        let mds_0_1 = Expression::Constant(mds[0][1]);
        let mds_0_2 = Expression::Constant(mds[0][2]);
        let mds_0_3 = Expression::Constant(mds[0][3]);
        let mds_1_0 = Expression::Constant(mds[1][0]);
        let mds_1_1 = Expression::Constant(mds[1][1]);
        let mds_1_2 = Expression::Constant(mds[1][2]);
        let mds_1_3 = Expression::Constant(mds[1][3]);
        let mds_2_0 = Expression::Constant(mds[2][0]);
        let mds_2_1 = Expression::Constant(mds[2][1]);
        let mds_2_2 = Expression::Constant(mds[2][2]);
        let mds_2_3 = Expression::Constant(mds[2][3]);
        let mds_3_0 = Expression::Constant(mds[3][0]);
        let mds_3_1 = Expression::Constant(mds[3][1]);
        let mds_3_2 = Expression::Constant(mds[3][2]);
        let mds_3_3 = Expression::Constant(mds[3][3]);
        
        // constraint - computes vector matrix product
        vec![
            s_mds_mul.clone() * (a0_next - (a0.clone()*mds_0_0 + a1.clone()*mds_0_1 + a2.clone()*mds_0_2) + a3.clone()*mds_0_3),
            s_mds_mul.clone() * (a1_next - (a0.clone()*mds_1_0 + a1.clone()*mds_1_1 + a2.clone()*mds_1_2) + a3.clone()*mds_1_3),
            s_mds_mul.clone() * (a2_next - (a0.clone()*mds_2_0 + a1.clone()*mds_2_1 + a2.clone()*mds_2_2) + a3.clone()*mds_2_3),
            s_mds_mul * (a3_next - (a0*mds_3_0 + a1*mds_3_1 + a2*mds_3_2 + a3*mds_3_3))
        ]
    });
}

// helper function for creating the partial SB gate (poseidon-hash module computes state[0]^5 for partial rounds)
fn create_partial_sbox_gate_ps<F: PrimeField>(
    meta: &mut ConstraintSystem<F>,
    advice: Column<Advice>,
    s_sub_bytes_partial: Selector, 
) {
    meta.create_gate("PS_partial_sbox_gate", |meta| {
        let s_sub_bytes_partial = meta.query_selector(s_sub_bytes_partial);
        let a0 = meta.query_advice(advice, Rotation::cur()); // state[0] = state[0]**5, alpha = 5
        let a0_next = meta.query_advice(advice, Rotation::next());

        vec![s_sub_bytes_partial* (a0_next - (a0.clone()*a0.clone()*a0.clone()*a0.clone()*a0))]
    });
}

// helper function for creating the full SB gate (poseidon-hash module uses alpha=5 in neptune parameters)
fn create_full_sbox_gate_ps<F: PrimeField>(
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 4],
    s_sub_bytes_full: Selector, 
) {
    meta.create_gate("PS_full_sbox_gate", |meta| {
        let s_sub_bytes_full = meta.query_selector(s_sub_bytes_full);
        let a0 = meta.query_advice(advice[0], Rotation::cur());
        let a1 = meta.query_advice(advice[1], Rotation::cur());
        let a2 = meta.query_advice(advice[2], Rotation::cur()); 
        let a3 = meta.query_advice(advice[3], Rotation::cur());
        let a0_next = meta.query_advice(advice[0], Rotation::next());
        let a1_next = meta.query_advice(advice[1], Rotation::next());
        let a2_next = meta.query_advice(advice[2], Rotation::next()); 
        let a3_next = meta.query_advice(advice[3], Rotation::next());

        vec![
            s_sub_bytes_full.clone() * (a0_next - (a0.clone()*a0.clone()*a0.clone()*a0.clone()*a0)),
            s_sub_bytes_full.clone() * (a1_next - (a1.clone()*a1.clone()*a1.clone()*a1.clone()*a1)),
            s_sub_bytes_full.clone() * (a2_next - (a2.clone()*a2.clone()*a2.clone()*a2.clone()*a2)),
            s_sub_bytes_full * (a3_next - (a3.clone()*a3.clone()*a3.clone()*a3.clone()*a3))
        ]
    });
}

// implementation of additional methods for the PoseidonChip
impl<F: PrimeField> PoseidonChip<F> {
    // constructor
    fn construct(config: <Self as Chip<F>>::Config) -> Self {
        PoseidonChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates and constraints TODO: add lookup argument here too
    fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 4],
        fixed: [Column<Fixed>; 4]
    ) -> <Self as Chip<F>>::Config {
        
    }
}
