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
pub struct SpongeChipConfig {
    advice: [Column<Advice>; 4],
    s_sponge_absorb: Selector
}

// structure for the sponge construction chip
struct SpongeChip<F: PrimeField> {
    config: SpongeChipConfig, 
    _marker: PhantomData<F>
}

// structure to store numbers in cells
struct Number<F: PrimeField>(AssignedCell<F, F>);

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
        let input_0 = meta.query_advice(advice[0], Rotation::cur());;
        let input_1 = meta.query_advice(advice[1], Rotation::cur());
        let input_2 = meta.query_advice(advice[2], Rotation::cur()); 
        let input_3 = meta.query_advice(advice[3], Rotation::cur());
        let a0_next = meta.query_advice(advice[0], Rotation::next());
        let a1_next = meta.query_advice(advice[1], Rotation::next());
        let a2_next = meta.query_advice(advice[2], Rotation::next()); 
        let a3_next = meta.query_advice(advice[3], Rotation::next());

        vec![
            s_sponge_absorb.clone() * (a0_next - a0_prev + input_0),
            s_sponge_absorb.clone() * (a1_next - a1_prev + input_1),
            s_sponge_absorb.clone() * (a2_next - a2_prev + input_2),
            s_sponge_absorb * (a3_next - a3_prev)
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
    ) -> <Self as Chip<F>>::Config {
        for column in &advice { // TODO: do this in the circuit synthesis not in the chip configuration
            meta.enable_equality(*column);
        }

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
trait SpongeInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // absorb - Sponge I/O
    fn absorb(
        &self, 
        layouter: impl Layouter<F>,
        state: [AssignedCell<F, F>; 4],
        inputs: [Value<F>; 3] // rate is 3 in neptune parameters
    ) -> Result<[AssignedCell<F, F>; 4], Error>;

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        layouter: impl Layouter<F>,
        state: [AssignedCell<F, F>; 4],
        c: usize // capacity is 1 in neptune parameters making t = r + c = 3 + 1 = 4
    ) -> Result<[Value<F>; 3], Error>; // capacity elements are retained in the sponge
}


// implement the SpongeInstructions trait for the SpongeChip
impl<F: PrimeField> SpongeInstructions<F> for SpongeChip<F> {
    type Num = Number<F>;
    // absorb - Sponge I/O
    // create a separate region for computing state = state + input and constraining it, return values only not cells to permute()
    fn absorb(
        &self, 
        mut layouter: impl Layouter<F>,
        state: [AssignedCell<F, F>; 4],
        inputs: [Value<F>; 3] 
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
                    region.assign_advice(|| "input0", config.advice[0], row_offset, || inputs[0])?,
                    region.assign_advice(|| "input1", config.advice[1], row_offset, || inputs[1])?,
                    region.assign_advice(|| "input2", config.advice[2], row_offset, || inputs[2])?
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

                Ok(next_state)
            }
        )
    }

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        layouter: impl Layouter<F>,
        state: [AssignedCell<F, F>; 4],
        c: usize 
    ) -> Result<[Value<F>; 3], Error> {
        Ok([state[0].value().copied(), state[1].value().copied(), state[2].value().copied()])
    }
}