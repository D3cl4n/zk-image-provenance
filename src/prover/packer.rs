use std::marker::PhantomData;
use ff::PrimeField;
use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, Value},
    plonk::{Advice, TableColumn, Column, ConstraintSystem, Error, Instance, Selector, Expression},
    poly::Rotation,
};


// structure for the packer chip config
#[derive(Clone, Debug)]
pub struct PackerChipConfig {

}