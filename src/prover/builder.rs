use crate::prover::Prover;

struct ProverBuilder {}

impl ProverBuilder {
    pub fn new() -> Self {
        ProverBuilder {}
    }

    pub fn build(&self) -> Prover {
        Prover{}
    }
}