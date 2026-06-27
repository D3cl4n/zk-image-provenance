use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Instance, Column, ConstraintSystem, Error, Selector, Expression},
    poly::Rotation,
};
use crate::prover::number::Number;


// structure for the packing chip config
#[derive(Clone, Debug)]
pub struct PackingChipConfig {
    pub advice: [Column<Advice>; 2], // one advice column for the accumulator (encodes position) one column for the byte being packed
    instance: Column<Instance>,
    s_pack: Selector
}


// structure for the packing chip
#[derive(Clone, Debug)]
pub struct PackingChip<F: PrimeField> {
    config: PackingChipConfig,
    _marker: PhantomData<F>
}


// implement the chip trait for PackingChip
impl<F: PrimeField> Chip<F> for PackingChip<F> {
    type Config = PackingChipConfig;
    type Loaded = ();

    // getter for the chip config 
    fn config(&self) -> &Self::Config {
        &self.config
    }

    // getter for the Loaded field
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}


// helper function to create the packing gate (accumulator_{i+1} = accumulator_{i}*256+byte)
fn create_packing_accumulation_gate<F: PrimeField> (
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 2],
    s_pack: Selector
) {
    meta.create_gate("packing_accumulation_gate", |meta| {
        let s_pack = meta.query_selector(s_pack);
        let accumulator_curr = meta.query_advice(advice[0], Rotation::cur());
        let accumulator_next = meta.query_advice(advice[0], Rotation::next());
        let byte = meta.query_advice(advice[1], Rotation::cur());

        vec![
            s_pack * (accumulator_next - (accumulator_curr * Expression::Constant(F::from(256 as u64)) + byte))
        ]
    });
}


// implementation of additional methods for the PackingChip
impl<F: PrimeField> PackingChip<F> {
    // constructor
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        PackingChip {config, _marker: PhantomData}
    }

    // configure the chip including all gates, constraints and selectors
    pub fn configure(
        meta: &mut ConstraintSystem<F>, 
        advice: [Column<Advice>; 2],
        instance: Column<Instance>
    ) -> <Self as Chip<F>>::Config {
        let s_pack = meta.selector();
        create_packing_accumulation_gate(meta, advice, s_pack);

        PackingChipConfig {
            advice,
            instance,
            s_pack
        }
    }
}


// trait for sub-functions of this chip
pub trait PackingChipInstructions<F: PrimeField>: Chip<F> {
    type Num;

    // expose output as public
    fn expose_as_public(
        &self, 
        layouter: &mut impl Layouter<F>, 
        num: Self::Num, 
        row: usize
    ) -> Result<(), Error>;

    // function signature for pack
    fn pack(
        &self, 
        layouter: &mut impl Layouter<F>,
        bytes: &Vec<Self::Num>
    ) -> Result<Vec<Number<F>>, Error>;
}


// implement the PackingChipInstructions trait for PackingChip
impl<F: PrimeField> PackingChipInstructions<F> for PackingChip<F> {
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

    // pack function definition
    fn pack(
        &self,
        layouter: &mut impl Layouter<F>,
        bytes: &Vec<Self::Num>
    ) -> Result<Vec<Number<F>>, Error> {
        let config = self.config();
        let bytes_per_element: usize = 31;

        let packed_elements = layouter.assign_region(|| "packing_region", |mut region| {
            let mut row_offset = 0;
            let base_256: F = F::from(256u64);
            let mut local_packed: Vec<Number<F>> = vec![];

            // iterate over each 31 byte chunk
            for chunk in bytes.chunks(bytes_per_element) {
                let mut accumulator_val: Value<F> = Value::known(F::ZERO);

                // iterate over each byte per chunk and pack into 1 field element
                for (i, byte) in chunk.iter().enumerate() {
                    let is_last = i == chunk.len() - 1;

                    if !is_last {
                        config.s_pack.enable(&mut region, row_offset)?;
                    }

                    region.assign_advice(|| format!("acc_curr_{}", row_offset), config.advice[0], row_offset, || accumulator_val)?;
                    let byte_cell: AssignedCell<F, F> = region.assign_advice(
                        || format!("byte_{}", row_offset),
                        config.advice[1], 
                        row_offset,
                        || byte.0.value().copied()
                    )?;

                    accumulator_val = accumulator_val
                        .zip(byte_cell.value().copied())
                        .map(|(a, b)| a * base_256 + b);

                    row_offset += 1;
                }

                let packed_cell: AssignedCell<F, F> = region.assign_advice(
                    || format!("packed_result_{}", row_offset),
                    config.advice[0],
                    row_offset,
                    || accumulator_val
                )?;

                row_offset += 1;
                local_packed.push(Number(packed_cell));
            }

            println!("[*] Packing chip rows used: {}", row_offset);

            Ok(local_packed)
        })?;

        Ok(packed_elements)
    }
} // end of implementation