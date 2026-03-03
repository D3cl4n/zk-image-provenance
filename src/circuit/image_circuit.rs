use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Region, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, TableColumn, Circuit, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};


// struct for the image provenance circuit config as a whole (hash + greyscale)
pub struct ImageCircuitConfig {
    greyscale: GreyscaleChipConfig,
    poseidon: PoseidonChipConfig,
    instance: Column<Instance>
}


// implementation of the circuit trait for the GreyscaleCircuit
// TODO: change this to one ImageCircuit for hashing and greyscale and add ImageCircuit struct that owns greyscale chip and poseidon chip
impl<F: PrimeField> Circuit<F> for ImageCircuit {
    type Config = GreyscaleChipConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    // configure the circuit including column creation
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = [meta.advice_column(), meta.advice_column(), meta.advice_column(), meta.advice_column()];
        let table = meta.lookup_table_column();

        GreyscaleChip::configure(meta, advice, table, instance)
    }

    // synthesize the circuit
    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = GreyscaleChip::construct(config.clone());

        // load the lookup table
        layouter.assign_table(
            || "lookup_table", |mut table| {
                for i in 0..256 {
                    table.assign_cell(
                        || "byte_val",
                        config.table,
                        i,
                        || Value::known(F::from(i as u64))
                    )?;
                }
                Ok(())
            }
        )?;

        let result = chip.greyscale(
            layouter.namespace(|| "greyscale_namespace"), 
            &self.r_elements, 
            &self.g_elements,
            &self.b_elements
        )?;

        // expose the greyscale values as public in the instance column
        for i in 0..result.len() {
            let public_value = Number(result[i].clone());
            chip.expose_as_public(layouter.namespace(|| "greyscale_value"), public_value, i)?;
        }

        Ok(())
    }
}