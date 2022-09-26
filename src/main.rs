mod assignment;
mod cnf;
mod solver;

use cnf::CnfFormula;
use dimacs::{Lit, Sign};
use solver::{SolveResult, Solver};
use std::path::Path;

fn lit_to_int(lit: Lit) -> i32 {
    let num = lit.var().to_u64() as i32;

    if lit.sign() == Sign::Pos {
        num
    } else {
        -num
    }
}

fn initialize<P: AsRef<Path>>(path: P) -> Solver {
    let contents = std::fs::read_to_string(path).expect("Failed to read input file");

    let parsed = dimacs::parse_dimacs(&contents).expect("Failed to parse file contents.");

    let formula =
        CnfFormula::try_from(parsed).expect("Incorrect file format. Got SAT when expecting CNF");
    Solver::create(formula)
}

fn print_solve_result(result: SolveResult) {
    match result {
        SolveResult::Sat(assignment) => {
            println!("SAT");

            for literal in assignment.iter() {
                print!("{} ", lit_to_int(literal));
            }

            println!("0");
        }

        SolveResult::Unsat => {
            println!("UNSAT");
        }
    }
}

fn main() {
    let file = std::env::args().nth(1).expect("Please provide a file.");

    let solver = initialize(file);

    println!("Starting solver...");
    let result = solver.solve();
    print_solve_result(result);
}
