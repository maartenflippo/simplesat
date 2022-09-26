use std::collections::VecDeque;

use dimacs::{Clause, Lit};

use crate::{assignment::Assignment, cnf::CnfFormula};

pub struct Solver {
    /// The assignment to the variables.
    assignment: Assignment,

    /// A history of the assignments.
    trail: VecDeque<(Lit, usize)>,
    order: Vec<Option<usize>>,

    /// The CNF formula that requires solving. Learned clauses are added to this
    /// formula as the solver identifies them.
    formula: Vec<Clause>,
    variable_count: usize,

    /// Storage for the decision levels at which literals were assigned.
    variable_decision_level: Vec<Option<usize>>,

    /// Storage for the antecedents of literals.
    variable_antecedent: Vec<Option<usize>>,
}

pub enum SolveResult {
    Sat(Assignment),
    Unsat,
}

impl Solver {
    pub fn create(formula: CnfFormula) -> Solver {
        let num_vars = formula.num_variables();
        let clauses = formula.clauses();

        Solver {
            assignment: Assignment::new(num_vars),
            trail: VecDeque::new(),
            order: vec![None; num_vars],
            variable_antecedent: vec![None; num_vars],
            variable_decision_level: vec![None; num_vars],
            formula: Vec::from(clauses),
            variable_count: num_vars,
        }
    }

    /// Run the solver to find a satisfying assignment or prove unsat.
    pub fn solve(mut self) -> SolveResult {
        let mut decision_level = 0;

        // Find top-level conflicts. If they exist, the formula is unsatisfiable.
        let unit_propagate_result = self.unit_propagate(decision_level);
        if unit_propagate_result.is_some() {
            return SolveResult::Unsat;
        }

        while !self.all_variables_assigned() {
            let picked_variable = self.pick_branching_variable();
            decision_level += 1;

            self.assign_literal(picked_variable, decision_level, None);
            println!(
                "Decision: literal {:?} set at dl {}",
                picked_variable, decision_level
            );

            // Continuously propagate and learn, until propagation no longer derives a conflict.
            loop {
                let unit_propagate_result = self.unit_propagate(decision_level);

                if let Some(conflicting_clause) = unit_propagate_result {
                    println!("\tConflict found.");

                    // If the conflict was at the top level, the formula is unsatisfiable
                    if decision_level == 0 {
                        return SolveResult::Unsat;
                    }

                    decision_level =
                        self.conflict_analysis_and_backtrack(conflicting_clause, decision_level);
                    println!("\tBacktracking to level {}", decision_level);
                } else {
                    // No conflict was derived, continue with search.
                    break;
                }
            }
        }

        // If we reached here, all variables were successfully assigned, and the
        // formula is satisfiable
        SolveResult::Sat(self.assignment)
    }

    fn all_variables_assigned(&self) -> bool {
        self.variable_count == self.assignment.size()
    }

    /// Run boolean constraint propagation on the formula.
    ///
    /// If propagation causes a clause to be conflicting, this method returns
    /// Some(clause_idx) where clause_idx is the index of the conflicting clause
    /// in the formula.
    ///
    /// If propagation finishes without identifying a conflict, None is
    /// returned.
    fn unit_propagate(&mut self, decision_level: usize) -> Option<usize> {
        let mut unit_clause_found = true;
        while unit_clause_found {
            unit_clause_found = false;

            // Iterate over all clauses if no unit clause has been found so far
            'clause: for clause_idx in 0..self.formula.len() {
                let clause = &self.formula[clause_idx];

                let mut unassigned_literal = None;

                for &literal in clause.lits() {
                    if self.assignment.is_true(literal) {
                        // The clause is satisfied.
                        continue 'clause;
                    }

                    if self.assignment.is_unassigned(literal) && unassigned_literal.is_none() {
                        // First unassigned literal we encountered for this clause.
                        unassigned_literal = Some(literal);
                    } else if self.assignment.is_unassigned(literal) {
                        // More than 1 unassigned literal, so we cannot propagate.
                        continue 'clause;
                    }
                }

                if let Some(literal) = unassigned_literal {
                    unit_clause_found = true;

                    self.assign_literal(literal, decision_level, Some(clause_idx));
                }
            }
        }

        let possible_conflict = self.formula.iter().enumerate().find(|(_, clause)| {
            clause
                .lits()
                .iter()
                .all(|&literal| self.assignment.is_false(literal))
        });

        if let Some((conflict_idx, _)) = possible_conflict {
            Some(conflict_idx)
        } else {
            None
        }
    }

    /// Take a dimacs literal and return the index for the variable this literal
    /// is for. Note: The variable is 0-indexed, whereas in DIMACS a the
    /// variable 0 does not exist.
    fn literal_to_variable_index(&self, literal: Lit) -> usize {
        literal.var().to_u64() as usize - 1
    }

    /// Assign a literal the value 'true', and record at which decision level
    /// the literal was assigned, as well as its antecedent.
    fn assign_literal(&mut self, literal: Lit, decision_level: usize, antecedent: Option<usize>) {
        let variable = self.literal_to_variable_index(literal); // get the index

        self.assignment.set_true(literal);
        self.order[variable] = Some(self.trail.len());
        self.trail.push_back((literal, decision_level));

        self.variable_decision_level[variable] = Some(decision_level); // set decision level
        self.variable_antecedent[variable] = antecedent; // set antecedent
    }

    /// Unassign the given literal, as well as update the bookkeeping in place
    /// for each variable.
    fn unassign_literal(&mut self, literal: Lit) {
        let literal_index = self.literal_to_variable_index(literal);
        self.assignment.unassign(literal);
        self.order[literal_index] = None;
        self.variable_decision_level[literal_index] = None; // unassign decision level
        self.variable_antecedent[literal_index] = None; // unassign antecedent
    }

    /// Analyze the conflict, which occurs in the clause with index
    /// 'conflicting_clause'.
    fn conflict_analysis_and_backtrack(
        &mut self,
        conflicting_clause: usize,
        conflict_decision_level: usize,
    ) -> usize {
        // the new clause to learn, initialized with the antecedent of the conflict
        let mut learnt_clause = Vec::from(self.formula[conflicting_clause].lits());

        loop {
            let assigned_at_conflict_level = learnt_clause
                .iter()
                .filter(|&&lit| self.decision_level(lit).unwrap() == conflict_decision_level)
                .count();

            if assigned_at_conflict_level == 1 {
                break;
            }

            let resolving_literal = learnt_clause
                .iter()
                .filter(|&&lit| self.decision_level(lit).unwrap() == conflict_decision_level)
                .max_by_key(|&&lit| self.assignment_order(lit));

            match resolving_literal {
                Some(&lit) => self.resolve(&mut learnt_clause, lit),
                None => {}
            }
        }

        self.formula.push(Clause::from_vec(learnt_clause));

        let backtracked_decision_level = self
            .formula
            .last()
            .unwrap()
            .lits()
            .iter()
            .map(|&lit| self.decision_level(lit).unwrap())
            .filter(|&level| level != conflict_decision_level)
            .max()
            .unwrap_or(0);

        self.backtrack_to_level(backtracked_decision_level);
        backtracked_decision_level
    }

    /// Undo variable assignments above the given decision level.
    fn backtrack_to_level(&mut self, target_decision_level: usize) {
        loop {
            let (literal, decision_level) = match self.trail.back() {
                Some(&entry) => entry,
                None => return,
            };

            if decision_level == target_decision_level {
                return;
            }

            self.trail.pop_back();
            self.unassign_literal(literal);
        }
    }

    /// Pick the next literal to assign in the search. This will return a
    /// literal whose value is not yet assigned.
    fn pick_branching_variable(&self) -> Lit {
        // This is very naive and inefficient, but should work for very small
        // instances. Just pick the first unassigned literal in a clause which
        // is not yet satisfied.

        for clause in self.formula.iter() {
            if self.is_sat(clause) {
                continue;
            }

            for &literal in clause.lits() {
                if self.assignment.is_unassigned(literal) {
                    return literal;
                }
            }
        }

        panic!("Could not find branching variable.")
    }

    fn resolve(&mut self, input_clause: &mut Vec<Lit>, literal: Lit) {
        let literal_index = self.literal_to_variable_index(literal);
        let antecedent = self.variable_antecedent[literal_index].unwrap();

        // Add the antecedent to the input clause.
        input_clause.extend(self.formula[antecedent].lits());

        // Find all indices in the (now extended) input clause for which the
        // literal has the same variable as the resolving literal. These need to
        // be removed from the input_clause.
        let indices_to_remove = input_clause
            .iter()
            .enumerate()
            .filter(|(_, lit)| literal.var() == lit.var())
            .map(|(idx, _)| idx)
            .collect::<Vec<_>>();

        for index in indices_to_remove.iter().rev() {
            input_clause.remove(*index);
        }

        // Remove duplicates from the result. The `lit_to_int` function is there
        // because we cannot implement PartialOrd on the dimacs::Lit struct ourselves.
        input_clause.sort_by(|&a, &b| lit_to_int(a).cmp(&lit_to_int(b)));
        input_clause.dedup();
    }

    /// Indicate whether a clause is satisfied under the current assignment.
    fn is_sat(&self, clause: &Clause) -> bool {
        clause
            .lits()
            .iter()
            .any(|&lit| self.assignment.is_true(lit))
    }

    /// Get the decision level at which the given literal was assigned, or None
    /// if the literal is unassigned.
    fn decision_level(&self, literal: Lit) -> Option<usize> {
        let idx = self.literal_to_variable_index(literal);
        self.variable_decision_level[idx]
    }

    /// If this returns Some(i), then the given literal is the (i+1)th literal
    /// to be assigned. If this returns None, the given literal is not yet
    /// assigned.
    fn assignment_order(&self, literal: Lit) -> Option<usize> {
        let idx = self.literal_to_variable_index(literal);
        self.order[idx]
    }
}

fn lit_to_int(lit: Lit) -> i32 {
    use dimacs::Sign;

    let num = lit.var().to_u64() as i32;

    if lit.sign() == Sign::Pos {
        num
    } else {
        -num
    }
}
