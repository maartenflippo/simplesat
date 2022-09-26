use dimacs::{Clause, Instance};

pub struct CnfFormula {
    num_vars: usize,
    clauses: Box<[Clause]>,
}

#[derive(Debug)]
pub struct UnknownFormulaFormat;

impl TryFrom<Instance> for CnfFormula {
    type Error = UnknownFormulaFormat;

    fn try_from(instance: Instance) -> Result<Self, Self::Error> {
        match instance {
            Instance::Cnf { num_vars, clauses } => Ok(CnfFormula {
                num_vars: num_vars as usize,
                clauses,
            }),
            Instance::Sat { .. } => Err(UnknownFormulaFormat),
        }
    }
}

impl CnfFormula {
    pub fn num_variables(&self) -> usize {
        self.num_vars
    }

    pub fn clauses(&self) -> &[Clause] {
        &self.clauses
    }
}
