use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Fixed, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};

//TODO: refactor to load inputs into advice columns (part of private witness) and use region.constrain_equal
// so that the two chips can return and pass AssignedCells not Value<F>

// structure for sponge construction chip configuration
#[derive(Clone, Debug)]
pub struct SpongeChipConfig<F: PrimeField> {
    advice: [Column<Advice>; 4],
    fixed: [Column<Fixed>; 4],
    s_sponge_absorb: Selector,
    s_sponge_squeeze: Selector
}

// structure for the sponge construction chip
struct SpongeChip<F: PrimeField> {
    config: SpongeChipConfig<F>, 
    _marker: PhantomData<F>
}

// structure to store numbers in cells
struct Number<F: PrimeField>(AssignedCell<F, F>);

// implement the Chip trait for SpongeChip
impl<F: PrimeField> Chip<F> for SpongeChip<F> {
    type Config = SpongeChipConfig<F>;
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
    fixed: [Column<Fixed>; 3], // store the input to be absorbed in the r fixed columns
    s_sponge_absorb: Selector
) {
    meta.create_gate("PS_sponge_absorb_gate", |meta| {
        let s_sponge_absorb = meta.query_selector(s_sponge_absorb);
        let a0 = meta.query_advice(advice[0], Rotation::cur());
        let a1 = meta.query_advice(advice[1], Rotation::cur());
        let a2 = meta.query_advice(advice[2], Rotation::cur()); 
        let a3 = meta.query_advice(advice[3], Rotation::cur());
        let a0_next = meta.query_advice(advice[0], Rotation::next());
        let a1_next = meta.query_advice(advice[1], Rotation::next());
        let a2_next = meta.query_advice(advice[2], Rotation::next()); 
        let a3_next = meta.query_advice(advice[3], Rotation::next());
        let input_0 = meta.query_fixed(fixed[0]);
        let input_1 = meta.query_fixed(fixed[1]);
        let input_2 = meta.query_fixed(fixed[2]);

        vec![
            s_sponge_absorb.clone() * (a0_next - a0 + input_0),
            s_sponge_absorb.clone() * (a1_next - a1 + input_1),
            s_sponge_absorb.clone() * (a2_next - a2 + input_2),
            s_sponge_absorb * (a3_next - a3)
        ]
    });

}

// implementation of additional methods for the SpongeChip
impl<F: PrimeField> SpongeChip<F> {
    // constructor
    fn construct(config: <Self as Chip<F>>::Config) -> Self {
        SpongeChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates and constraints TODO: add lookup argument here too
    fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 4],
        fixed: [Column<Fixed>; 4]
    ) -> <Self as Chip<F>>::Config {
        for column in &advice {
            meta.enable_equality(*column);
        }

        for column in &fixed {
            meta.enable_constant(*column);
        }

        let s_sponge_absorb = meta.selector();
        let s_sponge_squeeze = meta.selector();

        // create the sponge I/O gates
        create_sponge_absorb_gate(meta, advice, [fixed[0], fixed[1], fixed[2]], s_sponge_absorb);

        SpongeChipConfig {
            advice, 
            fixed,
            s_sponge_absorb, 
            s_sponge_squeeze
        }
    }
}

// trait for the sub-functions of the sponge construction
trait SpongeInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // absorb - Sponge I/O
    fn absorb(
        &self, 
        layouter: impl Layouter<F>,
        state: [Value<F>; 4],
        inputs: [Value<F>; 3] // rate is 3 in neptune parameters
    ) -> Result<[Value<F>; 4], Error>;

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        layouter: impl Layouter<F>,
        state: [Value<F>; 4],
        c: usize // capacity is 1 in neptune parameters making t = r + c = 3 + 1 = 4
    ) -> Result<[Value<F>; 3], Error>; // capacity elements are retained in the sponge
}


// implement the SpongeInstructions trait for the SpongeChip
impl<F: PrimeField> SpongeInstructions<F> for SpongeChip<F> {
    // TODO: validate this  
    // absorb - Sponge I/O
    // create a separate region for computing state = state + input and constraining it, return values only not cells to permute()
    fn absorb(
        &self, 
        mut layouter: impl Layouter<F>,
        state: [Value<F>; 4],
        inputs: [Value<F>; 3] 
    ) -> Result<[Value<F>; 4], Error> {
        let config = self.config();
        layouter.assign_region(
            || "sponge_absorb_region", |mut region| {
                let mut row_offset: usize = 0;
                config.s_sponge_absorb.enable(&mut region, row_offset)?;

                let internal_state = [
                    region.assign_advice(|| "a0", config.advice[0], row_offset, || state[0])?,
                    region.assign_advice(|| "a1", config.advice[1], row_offset, || state[1])?,
                    region.assign_advice(|| "a2", config.advice[2], row_offset, || state[2])?,
                    region.assign_advice(|| "a3", config.advice[3], row_offset, || state[3])?
                ];

                let input_elements = [
                    region.assign_fixed(|| "input0", config.fixed[0], row_offset, || inputs[0])?,
                    region.assign_fixed(|| "input1", config.fixed[1], row_offset, || inputs[1])?,
                    region.assign_fixed(|| "input2", config.fixed[2], row_offset, || inputs[2])?
                ];

                let after_absorb = [
                    internal_state[0].value().copied() + input_elements[0].value().copied(),
                    internal_state[1].value().copied() + input_elements[1].value().copied(),
                    internal_state[2].value().copied() + input_elements[2].value().copied()
                ];

                row_offset += 1;
                region.assign_advice(|| "a0_next", config.advice[0], row_offset, || after_absorb[0])?;
                region.assign_advice(|| "a1_next", config.advice[1], row_offset, || after_absorb[1])?;
                region.assign_advice(|| "a2_next", config.advice[2], row_offset, || after_absorb[2])?;
                region.assign_advice(|| "a3_next", config.advice[3], row_offset, || internal_state[3].value().copied())?;

                Ok([after_absorb[0], after_absorb[1], after_absorb[2], internal_state[3].value().copied()])
            }
        )
    }

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        layouter: impl Layouter<F>,
        state: [Value<F>; 4],
        c: usize 
    ) -> Result<[Value<F>; 3], Error> {
        Ok([state[0].clone(), state[1].clone(), state[2].clone()])
    }
}