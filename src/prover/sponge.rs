use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use crate::prover::number::Number;
use crate::prover::tamper::{self, T};


// structure for sponge construction chip configuration
#[derive(Clone, Debug)]
pub struct SpongeChipConfig {
    advice: [Column<Advice>; 4],
    s_sponge_absorb: Selector
}

// structure for the sponge construction chip
pub struct SpongeChip<F: PrimeField> {
    config: SpongeChipConfig, 
    _marker: PhantomData<F>
}


// implement the Chip trait for SpongeChip
impl<F: PrimeField> Chip<F> for SpongeChip<F> {
    type Config = SpongeChipConfig;
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

// helper function for creating the sponge absorb gate and constraints
fn create_sponge_absorb_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>, 
    advice: [Column<Advice>; 4],
    s_sponge_absorb: Selector
) {
    meta.create_gate("PS_sponge_absorb_gate", |meta| {
        let s_sponge_absorb = meta.query_selector(s_sponge_absorb);
        let a0_prev = meta.query_advice(advice[0], Rotation::prev());
        let a1_prev = meta.query_advice(advice[1], Rotation::prev());
        let a2_prev = meta.query_advice(advice[2], Rotation::prev());
        let a3_prev = meta.query_advice(advice[3], Rotation::prev());
        let input_0 = meta.query_advice(advice[0], Rotation::cur());
        let input_1 = meta.query_advice(advice[1], Rotation::cur());
        let input_2 = meta.query_advice(advice[2], Rotation::cur()); 
        let a0_next = meta.query_advice(advice[0], Rotation::next());
        let a1_next = meta.query_advice(advice[1], Rotation::next());
        let a2_next = meta.query_advice(advice[2], Rotation::next()); 
        let a3_next = meta.query_advice(advice[3], Rotation::next());

        vec![
            s_sponge_absorb.clone() * (a0_next - (a0_prev + input_0)),
            s_sponge_absorb.clone() * (a1_next - (a1_prev + input_1)),
            s_sponge_absorb.clone() * (a2_next - (a2_prev + input_2)),
            s_sponge_absorb * (a3_next - a3_prev)
        ]
    });

}

// implementation of additional methods for the SpongeChip
impl<F: PrimeField> SpongeChip<F> {
    // constructor
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        SpongeChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates and constraints 
    pub fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 4]
    ) -> <Self as Chip<F>>::Config {
        let s_sponge_absorb = meta.selector();

        // create the sponge I/O gates
        create_sponge_absorb_gate(meta, advice, s_sponge_absorb);

        SpongeChipConfig {
            advice,
            s_sponge_absorb
        }
    }
}

// trait for the sub-functions of the sponge construction
pub trait SpongeInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // initialize the sponge permutation's internal state to all 0
    fn initialize(
        &self, 
        layouter: &mut impl Layouter<F>
    ) -> Result<[AssignedCell<F, F>; 4], Error>;

    // absorb - Sponge I/O
    fn absorb(
        &self, 
        layouter: &mut impl Layouter<F>,
        state: [AssignedCell<F, F>; 4],
        inputs: [AssignedCell<F, F>; 3] // rate is 3 in neptune parameters
    ) -> Result<[AssignedCell<F, F>; 4], Error>;

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        state: [AssignedCell<F, F>; 4]
    ) -> Result<AssignedCell<F, F>, Error>; // capacity elements are retained in the sponge

    // pad - append constant 1 then constant zeros to a multiple of r; split into blocks of r
    fn pad(
        &self,
        layouter: &mut impl Layouter<F>,
        elements: Vec<AssignedCell<F, F>>,
        r: usize
    ) -> Result<Vec<[AssignedCell<F, F>; 3]>, Error>;
}


// implement the SpongeInstructions trait for the SpongeChip
impl<F: PrimeField> SpongeInstructions<F> for SpongeChip<F> {
    type Num = Number<F>;

    // initialize the internal state to all 0s per Poseidon paper
    fn initialize(
        &self, 
        layouter: &mut impl Layouter<F>
    ) -> Result<[AssignedCell<F, F>; 4], Error> {
        let config = self.config();
        layouter.assign_region(
            || "initial_state", |mut region| {
                let first = if tamper::get() == T::SpongeInit {
                    let c = region.assign_advice(|| "input0_tampered", config.advice[0], 0, || Value::known(F::ONE))?;
                    region.constrain_constant(c.cell(), F::ZERO)?;
                    c
                } else {
                    region.assign_advice_from_constant(|| "input0", config.advice[0], 0, F::ZERO)?
                };
                Ok([
                    first,
                    region.assign_advice_from_constant(|| "input1", config.advice[1], 0, F::ZERO)?,
                    region.assign_advice_from_constant(|| "input2", config.advice[2], 0, F::ZERO)?,
                    region.assign_advice_from_constant(|| "input3", config.advice[3], 0, F::ZERO)?
                ])
            }
        )
    }

    // absorb - Sponge I/O
    fn absorb(
        &self, 
        layouter: &mut impl Layouter<F>,
        state: [AssignedCell<F, F>; 4],
        inputs: [AssignedCell<F, F>; 3] 
    ) -> Result<[AssignedCell<F, F>; 4], Error> {
        let config = self.config();
        layouter.assign_region(
            || "sponge_absorb_region", |mut region| {
                let mut row_offset: usize = 0;
                // copy the current state into this region
                let prev_state = [
                    state[0].copy_advice(|| "a0", &mut region, config.advice[0], row_offset)?,
                    state[1].copy_advice(|| "a1", &mut region, config.advice[1], row_offset)?,
                    state[2].copy_advice(|| "a2", &mut region, config.advice[2], row_offset)?,
                    state[3].copy_advice(|| "a3", &mut region, config.advice[3], row_offset)?
                ];

                // write input elements to the r advice columns
                row_offset += 1;
                let input_elements = [
                    inputs[0].copy_advice(|| "input0", &mut region, config.advice[0], row_offset)?,
                    inputs[1].copy_advice(|| "input1", &mut region, config.advice[1], row_offset)?,
                    inputs[2].copy_advice(|| "input2", &mut region, config.advice[2], row_offset)?
                ];

                config.s_sponge_absorb.enable(&mut region, row_offset)?;
                row_offset += 1;

                // write the next state to the advice columns after input elements are added to previous state
                let next_state = [
                    region.assign_advice(
                        || "a0_next",
                        config.advice[0],
                        row_offset,
                        || prev_state[0].value().copied() + input_elements[0].value().copied()
                    )?,
                    region.assign_advice(
                        || "a1_next",
                        config.advice[1],
                        row_offset,
                        || prev_state[1].value().copied() + input_elements[1].value().copied()
                    )?,
                    region.assign_advice(
                        || "a2_next",
                        config.advice[2],
                        row_offset,
                        || prev_state[2].value().copied() + input_elements[2].value().copied()
                    )?,
                    region.assign_advice(
                        || "a3_next",
                        config.advice[3],
                        row_offset,
                        || prev_state[3].value().copied()
                    )?
                ];

                //println!("[*] Sponge chip absorb rows used: {}", row_offset);

                Ok(next_state)
            }
        )
    }

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        state: [AssignedCell<F, F>; 4]
    ) -> Result<AssignedCell<F, F>, Error> {
        Ok(state[1].clone()) // match poseidon-hash module and squeeze the 2nd state element
    }

    fn pad(
        &self,
        layouter: &mut impl Layouter<F>,
        elements: Vec<AssignedCell<F, F>>,
        r: usize
    ) -> Result<Vec<[AssignedCell<F, F>; 3]>, Error> {
        let config = self.config();
        let mut all = elements;
        let zeros = if (all.len() + 1) % r == 0 { 0 } else { r - (all.len() + 1) % r };
        let pad_cells: Vec<AssignedCell<F, F>> = layouter.assign_region(
            || "sponge_padding",
            |mut region| {
                let mut out = vec![];
                if tamper::get() == T::SpongePad {
                    let c = region.assign_advice(|| "pad_one_tampered", config.advice[0], 0, || Value::known(F::from(2u64)))?;
                    region.constrain_constant(c.cell(), F::ONE)?;
                    out.push(c);
                } else {
                    out.push(region.assign_advice_from_constant(|| "pad_one", config.advice[0], 0, F::ONE)?);
                }
                for i in 0..zeros {
                    out.push(region.assign_advice_from_constant(|| "pad_zero", config.advice[0], i + 1, F::ZERO)?);
                }
                Ok(out)
            },
        )?;
        all.extend(pad_cells);
        Ok(all.chunks(r).map(|c| [c[0].clone(), c[1].clone(), c[2].clone()]).collect())
    }
}
