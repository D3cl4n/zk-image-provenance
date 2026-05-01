use ff::PrimeField;
use halo2_proofs::circuit::AssignedCell;

#[derive(Clone, Debug)]
pub struct Number<F: PrimeField>(pub AssignedCell<F, F>);