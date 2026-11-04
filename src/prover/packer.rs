use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, Instance, Column, ConstraintSystem, Error, Selector, Expression, TableColumn},
    poly::Rotation,
};
use crate::prover::number::Number;
use crate::prover::tamper::{self, T};

#[derive(Clone, Debug)]
pub struct PackingChipConfig {
    pub advice: [Column<Advice>; 2], // [accumulator, byte]
    instance: Column<Instance>,
    s_pack: Selector,
    s_range: Selector,
}

#[derive(Clone, Debug)]
pub struct PackingChip<F: PrimeField> {
    config: PackingChipConfig,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> Chip<F> for PackingChip<F> {
    type Config = PackingChipConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config { &self.config }
    fn loaded(&self) -> &Self::Loaded { &() }
}

// acc_{i+1} = 256 * acc_i + byte_i
fn create_packing_accumulation_gate<F: PrimeField>(
    meta: &mut ConstraintSystem<F>,
    advice: [Column<Advice>; 2],
    s_pack: Selector,
) {
    meta.create_gate("packing_accumulation_gate", |meta| {
        let s_pack = meta.query_selector(s_pack);
        let acc_curr = meta.query_advice(advice[0], Rotation::cur());
        let acc_next = meta.query_advice(advice[0], Rotation::next());
        let byte = meta.query_advice(advice[1], Rotation::cur());
        vec![s_pack * (acc_next - (acc_curr * Expression::Constant(F::from(256u64)) + byte))]
    });
}

impl<F: PrimeField> PackingChip<F> {
    pub fn construct(config: <Self as Chip<F>>::Config) -> Self {
        PackingChip { config, _marker: PhantomData }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
        byte_table: TableColumn,
    ) -> <Self as Chip<F>>::Config {
        let s_pack = meta.selector();
        let s_range = meta.complex_selector();
        create_packing_accumulation_gate(meta, advice, s_pack);

        // every byte loaded through load_bytes must lie in [0, 255]
        meta.lookup(|meta| {
            let s = meta.query_selector(s_range);
            let byte = meta.query_advice(advice[1], Rotation::cur());
            vec![(s * byte, byte_table)]
        });

        PackingChipConfig { advice, instance, s_pack, s_range }
    }
}

pub trait PackingChipInstructions<F: PrimeField>: Chip<F> {
    type Num;

    fn expose_as_public(
        &self,
        layouter: &mut impl Layouter<F>,
        num: Self::Num,
        row: usize,
    ) -> Result<(), Error>;

    // witness bytes and range-check them to [0,255]
    fn load_bytes(
        &self,
        layouter: &mut impl Layouter<F>,
        bytes: &[u8],
    ) -> Result<Vec<Number<F>>, Error>;

    fn pack(
        &self,
        layouter: &mut impl Layouter<F>,
        bytes: &Vec<Self::Num>,
    ) -> Result<Vec<Number<F>>, Error>;
}

impl<F: PrimeField> PackingChipInstructions<F> for PackingChip<F> {
    type Num = Number<F>;

    fn expose_as_public(
        &self,
        layouter: &mut impl Layouter<F>,
        num: Self::Num,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(num.0.cell(), self.config().instance, row)
    }

    fn load_bytes(
        &self,
        layouter: &mut impl Layouter<F>,
        bytes: &[u8],
    ) -> Result<Vec<Number<F>>, Error> {
        let config = self.config();
        layouter.assign_region(
            || "load_bytes",
            |mut region| {
                let mut out = vec![];
                for (i, b) in bytes.iter().enumerate() {
                    config.s_range.enable(&mut region, i)?;
                    let val = if tamper::get() == T::ExifRange && i == 0 { F::from(256u64) } else { F::from(*b as u64) };
                    let c = region.assign_advice(
                        || "byte",
                        config.advice[1],
                        i,
                        || Value::known(val),
                    )?;
                    out.push(Number(c));
                }
                Ok(out)
            },
        )
    }

    fn pack(
        &self,
        layouter: &mut impl Layouter<F>,
        bytes: &Vec<Self::Num>,
    ) -> Result<Vec<Number<F>>, Error> {
        let config = self.config();
        let bytes_per_element: usize = 31;
        let base_256: F = F::from(256u64);

        layouter.assign_region(
            || "packing_region",
            |mut region| {
                let mut row_offset = 0usize;
                let mut packed: Vec<Number<F>> = vec![];

                for chunk in bytes.chunks(bytes_per_element) {
                    // accumulator starts at a CONSTANT zero (pinned in the fixed column)
                    let mut acc: AssignedCell<F, F> = if tamper::get() == T::PackerInit && row_offset == 0 {
                        let c = region.assign_advice(|| "acc_init_tampered", config.advice[0], row_offset, || Value::known(F::ONE))?;
                        region.constrain_constant(c.cell(), F::ZERO)?;
                        c
                    } else {
                        region.assign_advice_from_constant(|| "acc_init", config.advice[0], row_offset, F::ZERO)?
                    };

                    for byte in chunk.iter() {
                        // gate on EVERY byte row, including the last
                        config.s_pack.enable(&mut region, row_offset)?;
                        let byte_cell = if tamper::get() == T::PackByte(bytes.len()) && row_offset == 0 {
                            let c = region.assign_advice(|| "byte_tampered", config.advice[1], row_offset, || byte.0.value().copied() + Value::known(F::ONE))?;
                            region.constrain_equal(byte.0.cell(), c.cell())?;
                            c
                        } else {
                            byte.0.copy_advice(|| "byte", &mut region, config.advice[1], row_offset)?
                        };

                        acc = region.assign_advice(
                            || "acc_next",
                            config.advice[0],
                            row_offset + 1,
                            || acc.value().copied().zip(byte_cell.value().copied()).map(|(a, b)| a * base_256 + b),
                        )?;
                        row_offset += 1;
                    }

                    packed.push(Number(acc));
                    row_offset += 1;
                }
                Ok(packed)
            },
        )
    }
}

