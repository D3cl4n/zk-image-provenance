use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{Chip, Layouter, AssignedCell, Region, Value, Error},
    plonk::{Advice, Column, ConstraintSystem, Instance, Selector, Expression, VirtualCells},
    poly::Rotation,
};
use crate::prover::number::Number;

// load round constants at compile time using macro
include!("rc_limbs.rs");

// chip configuration for Poseidon2 sponge and permutation
#[derive(Clone, Debug)]
pub struct Poseidon2ChipConfig {
    advice: [Column<Advice>; 31],   // 31 advice columns for a wide construction
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


// helper function for MixLayer (ML) using the circular matrix formed by (2, 1, 1) and the state vector x
fn mul_me<F: PrimeField>(x: &[Expression<F>; 3]) -> [Expression<F>; 3] {
    let s = x[0].clone() + x[1].clone() + x[2].clone();
    [x[0].clone() + s.clone(), x[1].clone() + s.clone(), x[2].clone() + s]
}


// helper fuction of MixLayer (ML) using the circuit matrix for internal rounds
fn mul_mi<F: PrimeField>(x: &[Expression<F>; 3]) -> [Expression<F>; 3] {
    let s = x[0].clone() + x[1].clone() + x[2].clone();
    [x[0].clone() + s.clone(), x[1].clone() + s.clone(), x[2].clone() + x[2].clone() + s.clone()]
}


// helper function to recompute lane values given starting lanes, and b cells (from S-Box outputs)
fn recompute_end_lanes<F: PrimeField>(
    meta: &mut VirtualCells<F>, 
    advice: [Column<Advice>; 31], 
    rot: Rotation
) -> [Expression<F>; 3] {
    let mut lanes = [adv(meta, advice[0], rot), adv(meta, advice[1], rot), adv(meta, advice[2], rot)];
    for i in 0..14 {
        let b = adv(meta, advice[4 + 2*i], rot);
        lanes = mul_mi(&[b, lanes[1].clone(), lanes[2].clone()]);
    }
    lanes
}


/*
* Gate Purpose: bake in the initial absorb for the t lanes to values [0, 0, IV] and constrain the values
* Columns used: adv0 -> lane0, adv1 -> lane1, adv2 -> lane2
* Constraints: [adv0_curr - 0, adv1_curr - 0, adv2_curr - IV]
*/
fn create_init_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 3], // pass A19, A21, A23 here 
    s_init: Selector,
    iv: F
) {
    meta.create_gate("ps_init", |meta| {
        let s_init = meta.query_selector(s_init);
        let b_prev = [
            adv(meta, advice[0], Rotation::cur()),
            adv(meta, advice[1], Rotation::cur()),
            adv(meta, advice[2], Rotation::cur())
        ];
        let lanes = mul_me(&b_prev);

        vec![
            s_init.clone() * lanes[0].clone(), 
            s_init.clone() * lanes[1].clone(), 
            s_init.clone() * (lanes[2].clone() - Expression::Constant(iv))
        ]
    });
}


/*
* Gate Purpose: compute RF/2 external rounds in the using the e1 selector
* Columns used: A0, ..., A23 for the 24 cells of SB q, b values, A24 and A25 for the elements to absorb
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_e1_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 31],
    s_e1: Selector,
    rc: [&[F]; 4] // pass the round constants needed (3 per round, 4 rounds total)
) {
    meta.create_gate("e1_gate", |meta| {
        let s_e1 = meta.query_selector(s_e1);
        let b_prev = [
            adv(meta, advice[19], Rotation::prev()), 
            adv(meta, advice[21], Rotation::prev()), 
            adv(meta, advice[23], Rotation::prev())
        ];
        let m0 = adv(meta, advice[24], Rotation::cur()); // this rows messsage elements to be absorbed 
        let m1 = adv(meta, advice[25], Rotation::cur());
        let mut state = mul_me(&b_prev); // this is the ML layer for the end of the last permutation
        state[0] = state[0].clone() + m0; // absorb the next block now that the last permutation is finished
        state[1] = state[1].clone() + m1; 
        let mut lanes = mul_me(&state); // initial ML transformation before any external round happens
        
        let mut constraints = vec![]; // vector to hold all constraints
        for i in 0..4 { // for loop with 4 iterations since we are doing RF / 2 = 4 rounds in this gate
            let mut sbox_out = vec![]; // vector to hold the S-Box outputs
            for j in 0..3 { // perform ARC and SB over all 3 lanes, storing S-Box output in sbox_out
                let a = lanes[j].clone() + Expression::Constant(rc[i][j]); 
                let a_sq = adv(meta, advice[6*i + 2*j], Rotation::cur()); // multiply by 6 to get to the correct start col, each RF round takes 6 cols
                let b = adv(meta, advice[6*i + 2*j + 1], Rotation::cur()); // a_sq = a*a, b = a_sq * a_sq * a = a^5
                sbox_out.push(b.clone());

                // 24 constraints over the 12 a_sq values and the 12 b values
                constraints.push(s_e1.clone() * (a_sq.clone() - a.clone() * a.clone()));
                constraints.push(s_e1.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));
            }
            if i < 3 { lanes = mul_me(&[sbox_out[0].clone(), sbox_out[1].clone(), sbox_out[2].clone()]); }
        }

        constraints // return the constraints as required by create_gate
    });
}


/*
* Gate Purpose: compute RP/4 = 14 internal rounds in the next row using the i0 selector
* Columns used: A0, A1, A2 for the current state (ML from previous row), q and b in the subsequent pairs of cols
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_i0_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 31],
    s_i0: Selector,
    rc: &[F] // 14 RP rounds, each RP round only has lane0 go into ARC
) {
    meta.create_gate("i0_gate", |meta| {
        let s_i0 = meta.query_selector(s_i0);
        let b_prev = [
            adv(meta, advice[19], Rotation::prev()), 
            adv(meta, advice[21], Rotation::prev()), 
            adv(meta, advice[23], Rotation::prev())
        ];
        let start = mul_me(&b_prev); // complete last ML which was deferred from E1 to here
        let mut lanes_cur = [ // A0, A1, A2 hold MDS(A19_prev, A21_prev, A23_prev)
            adv(meta, advice[0], Rotation::cur()),
            adv(meta, advice[1], Rotation::cur()),
            adv(meta, advice[2], Rotation::cur())
        ];

        // constraints for the I0 gate
        let mut constraints = vec![];
        for i in 0..3 { // 3 constraints for A0, A1, A2 <-> A19, A21, A23 (previous row, MDS multiplied)
            constraints.push(s_i0.clone() * (lanes_cur[i].clone() - start[i].clone()));
        }

        // 14 internal RP rounds 
        for i in 0..14 {
            let a = lanes_cur[0].clone() + Expression::Constant(rc[i]); // ARC
            let a_sq = adv(meta, advice[3 + 2*i], Rotation::cur()); // a_sq stored in A3, b stored in A4 and so on
            let b = adv(meta, advice[3 + 2*i + 1], Rotation::cur());

            // 28 constraints, 2 per RP round performed
            constraints.push(s_i0.clone() * (a_sq.clone() - a.clone() * a.clone()));
            constraints.push(s_i0.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));

            if i < 13 { lanes_cur = mul_mi(&[b, lanes_cur[1].clone(), lanes_cur[2].clone()]); } // deferred to I1 for last I0 RP round
        }
        constraints
    });
}


/*
* Gate Purpose: compute RP/4 = 14 internal rounds in the next row using the int selector
* This helper function creates all 3 internal gates but needs to be 3 distinct gates because the round constants are different
* Columns used: A0, A1, A2 for the state after ML from previous row, q and b in the subsequent pairs of cols
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_int_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 31],
    s_int: Selector,
    rc: &[F]
) {
    meta.create_gate("int_gate", |meta| {
        let s_int = meta.query_selector(s_int);
        let start = recompute_end_lanes(meta, advice, Rotation::prev());
        let mut lanes_cur = [ // A0, A1, A2 hold MDS(A19_prev, A21_prev, A23_prev)
            adv(meta, advice[0], Rotation::cur()),
            adv(meta, advice[1], Rotation::cur()),
            adv(meta, advice[2], Rotation::cur())
        ];

        // constraints for the internal gate(s)
        let mut constraints = vec![];
        for i in 0..3 { // 3 constraints for A0, A1, A2 <-> reconstruct_end_lanes()
            constraints.push(s_int.clone() * (lanes_cur[i].clone() - start[i].clone()));
        }

        // 14 internal RP rounds 
        for i in 0..14 {
            let a = lanes_cur[0].clone() + Expression::Constant(rc[i]); // ARC
            let a_sq = adv(meta, advice[3 + 2*i], Rotation::cur()); // a_sq stored in A3, b stored in A4 and so on
            let b = adv(meta, advice[3 + 2*i + 1], Rotation::cur());

            // 28 constraints, 2 per RP round performed
            constraints.push(s_int.clone() * (a_sq.clone() - a.clone() * a.clone()));
            constraints.push(s_int.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));

            if i < 13 { lanes_cur = mul_mi(&[b, lanes_cur[1].clone(), lanes_cur[2].clone()]); } // deferred to next gate for last RP round
        }
        constraints
    });
}


/*
* Gate Purpose: compute RF/2 external rounds in the using the e2 selector
* Columns used: A0, ..., A31: (A0, A1, A2) are the MDS outputs but the lanes need to be recomputed
* Constraints: q = a^{2} constrained, b = q^{2}*a constrained, a is an ARC/ML function of previous cells
*/
fn create_e2_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 31],
    s_e2: Selector,
    rc: [&[F]; 4] // pass the round constants needed (3 per round, 4 rounds total)
) {
    meta.create_gate("e2_gate", |meta| {
        let s_e2 = meta.query_selector(s_e2);
        let mut lanes = recompute_end_lanes(meta, advice, Rotation::prev()); // I3 end lane values (after ML)
        let mut constraints = vec![];

        // perform and constrain q, b over 4 RF rounds
        for i in 0..4 {
            let mut sbox_out = vec![];
            for j in 0..3 {
                let a = lanes[j].clone() + Expression::Constant(rc[i][j]); 
                let a_sq = adv(meta, advice[6*i + 2*j], Rotation::cur()); // multiply by 6 to get to the correct start col, each RF round takes 6 cols
                let b = adv(meta, advice[6*i + 2*j + 1], Rotation::cur()); // a_sq = a*a, b = a_sq * a_sq * a = a^5
                sbox_out.push(b.clone());

                // 24 constraints over the 12 a_sq values and the 12 b values
                constraints.push(s_e2.clone() * (a_sq.clone() - a.clone() * a.clone()));
                constraints.push(s_e2.clone() * (b.clone() - a_sq.clone() * a_sq.clone() * a.clone()));
            }
            if i < 3 { lanes = mul_me(&[sbox_out[0].clone(), sbox_out[1].clone(), sbox_out[2].clone()]); }
        }
        constraints
    });
}


/*
* Gate Purpose: maps the digest from the end of the permutation to the instance column
* Columns used: instance, SB outputs from E2 (A19, A21, A23)
* Constraints: ensure equality between the hash in the instance column and the squeezed element
*/
fn create_digest_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 4], // pass A19, A21, A23 here
    instance: Column<Instance>, 
    s_digest: Selector
) {
    meta.create_gate("digest_gate", |meta| {
        let s_digest = meta.query_selector(s_digest);
        let b_prev = [
            adv(meta, advice[0], Rotation::cur()), 
            adv(meta, advice[1], Rotation::cur()), 
            adv(meta, advice[2], Rotation::cur())
        ];
        let mut lanes = mul_me(&b_prev); // E2 end lane values (after ML)
        let hash = adv(meta, advice[3], Rotation::cur()); // gets the hash value written into the instance column

        // only constraint is ensuring equality between the rate element squeezed and the hash
        vec![
            s_digest * (lanes[0].clone() - hash)
        ]
    });
}


// implementation of additional methods for the Poseidon2 chip
impl<F: PrimeField> Poseidon2Chip<F> {
    // constructor
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        Poseidon2Chip { config, _marker: PhantomData }
    }

    // configure the chip with all the gates and constraints
    pub fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 31],
        instance: Column<Instance>,
        iv: F,
        rc: &[F; 80] // 24 RCs for RF rounds, 56 RCs for RP rounds, 80 total
    ) -> <Self as Chip<F>>::Config {
        // enable equality on instance column and A24 to copy constrain digest values
        meta.enable_equality(advice[24]);
        meta.enable_equality(advice[25]);
        meta.enable_equality(instance);

        // make the needed selectors
        let s_init = meta.selector();
        let s_digest = meta.selector();
        let s_e1 = meta.selector();
        let s_e2 = meta.selector();
        let s_i0 = meta.selector();
        let s_i1 = meta.selector();
        let s_i2 = meta.selector();
        let s_i3 = meta.selector();

        create_init_gate(meta, [advice[19], advice[21], advice[23]], s_init, iv);
        create_e1_gate(meta, advice, s_e1, [&rc[0..3], &rc[3..6], &rc[6..9], &rc[9..12]]);
        create_i0_gate(meta, advice, s_i0, &rc[24..38]);
        create_int_gate(meta, advice, s_i1, &rc[38..52]);
        create_int_gate(meta, advice, s_i2, &rc[52..66]);
        create_int_gate(meta, advice, s_i3, &rc[66..80]);
        create_e2_gate(meta, advice, s_e2, [&rc[12..15], &rc[15..18], &rc[18..21], &rc[21..24]]);
        create_digest_gate(meta, [advice[19], advice[21], advice[23], advice[24]], instance, s_digest);

        Poseidon2ChipConfig {
            advice,
            instance, 
            s_init,
            s_e1,
            s_i: [s_i0, s_i1, s_i2, s_i3],
            s_e2,
            s_digest
        }
    }
}


// trait for the sub-functions of the poseidon permutation
pub trait Poseidon2Instructions<F: PrimeField>: Chip<F> {
    type Num;

    // permutation
    fn permute(
        &self, 
        layouter: &mut impl Layouter<F>,
        state: [AssignedCell<F, F>; 4]
    ) -> Result<[AssignedCell<F, F>; 4], Error>;
}


// implement the needed functionality for synthesis with the chip
impl<F: PrimeField> Poseidon2Instructions<F> for Poseidon2Chip<F> {
    type Num = Number<F>;

    // permutation function
    fn permute(
        &self, 
        mut layouter: impl Layouter<F>,
        state: [AssignedCell<F, F>; 3]
    ) -> Result<[AssignedCell<F, F>; 4], Error> {
        let chip_config = self.config();
    }
} 