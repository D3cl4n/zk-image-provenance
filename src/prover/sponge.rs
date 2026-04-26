use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
};


// structure for sponge construction chip configuration
#[derive(Clone, Debug)]
pub struct SpongeChipConfig {
    advice: [Column<Advice>; 4],
    instance: Column<Instance>,
    s_sponge_absorb: Selector
}

// structure for the sponge construction chip
pub struct SpongeChip<F: PrimeField> {
    config: SpongeChipConfig, 
    _marker: PhantomData<F>
}

// structure to store numbers in cells
pub struct Number<F: PrimeField>(pub AssignedCell<F, F>);

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

    // configure the chip including all gates and constraints TODO: add lookup argument here too
    pub fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 4],
        instance: Column<Instance>
    ) -> <Self as Chip<F>>::Config {
        let s_sponge_absorb = meta.selector();

        // create the sponge I/O gates
        create_sponge_absorb_gate(meta, advice, s_sponge_absorb);

        SpongeChipConfig {
            advice,
            instance,
            s_sponge_absorb
        }
    }
}

// trait for the sub-functions of the sponge construction
pub trait SpongeInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // expose the squeezed elements as public in the instance column
    fn expose_as_public(
        &self, 
        layouter: &mut impl Layouter<F>, 
        num: Self::Num, 
        row: usize
    ) -> Result<(), Error>;

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
        inputs: [Value<F>; 3] // rate is 3 in neptune parameters
    ) -> Result<[AssignedCell<F, F>; 4], Error>;

    // squeeze - Sponge I/O
    fn squeeze(
        &self, 
        state: [AssignedCell<F, F>; 4]
    ) -> Result<AssignedCell<F, F>, Error>; // capacity elements are retained in the sponge

    // pad - Sponge funtionality before input is passed to permute()
    // split each vector into chunks of size r, pad last chunk to r elements using 0s if needed
    // pack 31 bytes from the preimage into one field element
    fn pad(
        &self,
        r_channel: &Vec<u8>,
        g_channel: &Vec<u8>,
        b_channel: &Vec<u8>,
        exif: &Vec<u8>,
        r: usize
    ) -> Result<Vec<[Value<F>; 3]>, Error>; // return one vector of arrays (each 3 elements): pad(r || g || b || exif)
}


// implement the SpongeInstructions trait for the SpongeChip
impl<F: PrimeField> SpongeInstructions<F> for SpongeChip<F> {
    type Num = Number<F>;

    // expose output as public
    fn expose_as_public(
        &self, 
        layouter: &mut impl Layouter<F>, 
        num: Self::Num, 
        row: usize
    ) -> Result<(), Error> {
        let config = self.config();
        layouter.constrain_instance(num.0.cell(), config.instance, row)
    }

    // initialize the internal state to all 0s per Poseidon paper
    fn initialize(
        &self, 
        layouter: &mut impl Layouter<F>
    ) -> Result<[AssignedCell<F, F>; 4], Error> {
        let config = self.config();
        layouter.assign_region(
            || "initial_state", |mut region| {
                Ok([
                    region.assign_advice(|| "input0", config.advice[0], 0, || Value::known(F::from(0u64)))?,
                    region.assign_advice(|| "input1", config.advice[1], 0, || Value::known(F::from(0u64)))?,
                    region.assign_advice(|| "input2", config.advice[2], 0, || Value::known(F::from(0u64)))?,
                    region.assign_advice(|| "input3", config.advice[3], 0, || Value::known(F::from(0u64)))?
                ])
            }
        )
    }

    // absorb - Sponge I/O
    fn absorb(
        &self, 
        layouter: &mut impl Layouter<F>,
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
                    region.assign_advice(|| "input2", config.advice[2], row_offset, || inputs[2])?,
                    region.assign_advice(|| "zero", config.advice[3], row_offset, || Value::known(F::ZERO))?
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
        state: [AssignedCell<F, F>; 4]
    ) -> Result<AssignedCell<F, F>, Error> {
        Ok(state[1].clone()) // match poseidon-hash module and squeeze the 2nd state element
    }

    // pad function - returning pad(r_channel || g_channel || b_chanel || exif)
    fn pad(
        &self,
        r_channel: &Vec<u8>,
        g_channel: &Vec<u8>,
        b_channel: &Vec<u8>,
        exif: &Vec<u8>,
        r: usize
    ) -> Result<Vec<[Value<F>; 3]>, Error> {
        // concatenate input vectors 
        let input: Vec<u8> = r_channel.iter().copied()
            .chain(g_channel.iter().copied())
            .chain(b_channel.iter().copied())
            .chain(exif.iter().copied())
            .collect();

        // pack 31 bytes into each field element then pad and construct blocks
        let bytes_per_element: usize = 31;
        let mut preimage_elements: Vec<F> = input
            .chunks(bytes_per_element) // split input vector into slides of size bytes_per_element
            .map(|chunk| { // for each slice execute a closure to produce a packed field element
                let mut element: F = F::ZERO;
                let mut base: F = F::ONE;
                let base_256: F = F::from(256 as u64);

                // iterate over each byte in slice and pack into position based on powers of 256
                for &byte in chunk {
                    element += F::from(byte as u64) * base; // pack
                    base *= base_256;
                } 

                element
            })
            .collect();

        // divide the vector of packed field elements into slices of 3 and pad
        preimage_elements.push(F::ONE);
        let rem: usize = preimage_elements.len() % r;
        if rem != 0 {
            preimage_elements.resize(preimage_elements.len() + (r - rem), F::ZERO);
        }

        let blocks: Vec<[Value<F>; 3]> = preimage_elements
            .chunks(r)
            .map(|elements| {
                [
                    Value::known(elements[0]),
                    Value::known(elements[1]),
                    Value::known(elements[2])
                ]
            })
            .collect();

        // return
        Ok(blocks)
    }
}