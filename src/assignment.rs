use dimacs::{Lit, Sign};

pub struct Assignment {
    buffer: Vec<Option<bool>>,
    assigned_literal_count: usize,
}

impl Assignment {
    pub fn new(num_variables: usize) -> Assignment {
        Assignment {
            buffer: vec![None; num_variables],
            assigned_literal_count: 0,
        }
    }

    /// Indicates whether a literal is true under the current assignment. If
    /// the literal is unassigned, this will return false. Using this therefore
    /// cannot distinguish between the cases when the literal is false or
    /// unassigned.
    pub fn is_true(&self, literal: Lit) -> bool {
        self.buffer[self.index(literal)]
            .map(|value| value == (literal.sign() == Sign::Pos))
            .unwrap_or(false)
    }

    /// Indicates whether a literal is false under the current assignment. If
    /// the literal is unassigned, this will return false. Using this therefore
    /// cannot distinguish between the cases when the literal is true or
    /// unassigned.
    pub fn is_false(&self, literal: Lit) -> bool {
        self.buffer[self.index(literal)]
            .map(|value| value != (literal.sign() == Sign::Pos))
            .unwrap_or(false)
    }

    /// Indicates whether a literal is unassigned under the current assignment.
    pub fn is_unassigned(&self, literal: Lit) -> bool {
        self.buffer[self.index(literal)] == None
    }

    /// Set the value of the given literal to true under the current assignment.
    pub fn set_true(&mut self, literal: Lit) {
        let idx = self.index(literal);
        self.buffer[idx] = Some(literal.sign() == Sign::Pos);

        self.assigned_literal_count += 1;
    }

    pub fn unassign(&mut self, literal: Lit) {
        let idx = self.index(literal);
        self.buffer[idx] = None;

        self.assigned_literal_count -= 1;
    }

    /// Returns an iterator of the literals that are 'true' in the current
    /// assignment.
    pub fn iter(&self) -> impl Iterator<Item = Lit> + '_ {
        self.buffer
            .iter()
            .enumerate()
            .filter_map(|(variable_idx, &value)| {
                let var = (variable_idx + 1) as i64;

                value.map(|v| {
                    if v {
                        Lit::from_i64(var)
                    } else {
                        Lit::from_i64(-var)
                    }
                })
            })
    }

    pub fn size(&self) -> usize {
        self.assigned_literal_count
    }

    fn index(&self, literal: Lit) -> usize {
        literal.var().to_u64() as usize - 1
    }
}
