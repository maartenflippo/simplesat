use std::iter::FusedIterator;

use dimacs::{Lit, Sign};
use fixedbitset::FixedBitSet;

pub struct Assignment {
    buffer: FixedBitSet,
    assigned_literal_count: usize,
}

impl Assignment {
    pub fn new(num_variables: usize) -> Assignment {
        Assignment {
            buffer: FixedBitSet::with_capacity(num_variables * 2),
            assigned_literal_count: 0,
        }
    }

    /// Indicates whether a literal is true under the current assignment. If
    /// the literal is unassigned, this will return false. Using this therefore
    /// cannot distinguish between the cases when the literal is false or
    /// unassigned.
    pub fn is_true(&self, literal: Lit) -> bool {
        let idx = self.index(literal);
        self.buffer[idx] && self.buffer[idx + 1] == (literal.sign() == Sign::Pos)
    }

    /// Indicates whether a literal is false under the current assignment. If
    /// the literal is unassigned, this will return false. Using this therefore
    /// cannot distinguish between the cases when the literal is true or
    /// unassigned.
    pub fn is_false(&self, literal: Lit) -> bool {
        let idx = self.index(literal);
        self.buffer[idx] && self.buffer[idx + 1] != (literal.sign() == Sign::Pos)
    }

    /// Indicates whether a literal is unassigned under the current assignment.
    pub fn is_unassigned(&self, literal: Lit) -> bool {
        !self.buffer[self.index(literal)]
    }

    /// Set the value of the given literal to true under the current assignment.
    pub fn set_true(&mut self, literal: Lit) {
        let idx = self.index(literal);
        self.buffer.set(idx, true);
        self.buffer.set(idx + 1, literal.sign() == Sign::Pos);

        self.assigned_literal_count += 1;
    }

    pub fn unassign(&mut self, literal: Lit) {
        let idx = self.index(literal);
        self.buffer.set(idx, false);

        self.assigned_literal_count -= 1;
    }

    /// Returns an iterator of the literals that are 'true' in the current
    /// assignment.
    pub fn iter(&self) -> impl Iterator<Item = Lit> + '_ {
        AssignmentIter {
            assignment: self,
            idx: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.assigned_literal_count
    }

    fn index(&self, literal: Lit) -> usize {
        (literal.var().to_u64() as usize - 1) * 2
    }
}

struct AssignmentIter<'a> {
    assignment: &'a Assignment,
    idx: usize,
}

impl<'a> AssignmentIter<'a> {
    fn next_var(&mut self) {
        self.idx += 2;
    }

    fn is_finished(&self) -> bool {
        self.idx >= self.assignment.buffer.len()
    }
}

impl<'a> Iterator for AssignmentIter<'a> {
    type Item = Lit;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.is_finished() && !self.assignment.buffer[self.idx] {
            self.next_var();
        }

        if self.is_finished() {
            return None;
        }

        let var = (self.idx / 2) as i64 + 1;
        let is_positive = self.assignment.buffer[self.idx + 1];
        self.next_var();

        if is_positive {
            Some(Lit::from_i64(var))
        } else {
            Some(Lit::from_i64(-var))
        }
    }
}

impl<'a> FusedIterator for AssignmentIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_assignment_makes_all_variables_unset() {
        let assignment = Assignment::new(3);

        for var in 1..=3 {
            let pos = lit(var);
            let neg = lit(-var);

            assert!(assignment.is_unassigned(pos));
            assert!(assignment.is_unassigned(neg));
            assert!(!assignment.is_true(pos));
            assert!(!assignment.is_true(neg));
            assert!(!assignment.is_false(pos));
            assert!(!assignment.is_false(neg));
        }
    }

    #[test]
    fn assigning_a_literal_can_be_observed() {
        let mut assignment = Assignment::new(3);

        assignment.set_true(lit(2));
        assert!(assignment.is_true(lit(2)));
        assert!(!assignment.is_false(lit(2)));
        assert!(!assignment.is_true(lit(-2)));
        assert!(assignment.is_false(lit(-2)));
        assert!(!assignment.is_unassigned(lit(2)));
        assert_eq!(1, assignment.size());
    }

    #[test]
    fn unassigning_a_literal_is_observed() {
        let mut assignment = Assignment::new(3);

        let pos = lit(2);
        let neg = lit(-2);

        assignment.set_true(pos);
        assignment.unassign(pos);

        assert!(assignment.is_unassigned(pos));
        assert!(assignment.is_unassigned(neg));
        assert!(!assignment.is_true(pos));
        assert!(!assignment.is_true(neg));
        assert!(!assignment.is_false(pos));
        assert!(!assignment.is_false(neg));
    }

    #[test]
    fn iterator_gives_all_literals() {
        let mut assignment = Assignment::new(3);
        assignment.set_true(lit(1));
        assignment.set_true(lit(-2));
        assignment.set_true(lit(3));

        let lits = assignment.iter().collect::<Vec<_>>();
        assert_eq!(vec![lit(1), lit(-2), lit(3)], lits);
    }

    #[test]
    fn iterator_excludes_unassigned_literals() {
        let mut assignment = Assignment::new(3);
        assignment.set_true(lit(1));
        assignment.set_true(lit(-3));

        let lits = assignment.iter().collect::<Vec<_>>();
        assert_eq!(vec![lit(1), lit(-3)], lits);
    }

    fn lit(l: i64) -> Lit {
        Lit::from_i64(l)
    }
}
