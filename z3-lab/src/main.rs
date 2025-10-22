use z3::{Config, Context, Solver, ast::BV};

fn main() {
    let solver = Solver::new();

    // 32-bit unsigned variable
    let x = BV::fresh_const("x", 32);

    // Constants for division by 10
    let m = BV::from_u64(0xCCCCCCCD, 64); // magic number, 64-bit for multiplication
    let s = 35;

    // Extend x to 64-bit for multiplication
    let x64 = x.zero_ext(32); // 32-bit -> 64-bit
    let div_magic = x64.bvmul(&m).bvlshr(&BV::from_u64(s, 64));

    // Truncate back to 32-bit
    let div_magic32 = div_magic.extract(31, 0);

    // True division
    let div_true = x.bvudiv(&BV::from_u64(10, 32));

    // Check if any x violates the equality
    solver.assert(div_magic32.eq(div_true).not());

    dbg!(&solver);

    // Solve
    match solver.check() {
        z3::SatResult::Sat => {
            println!("Counterexample found: {}", solver.get_model().unwrap());
        }
        z3::SatResult::Unsat => {
            println!("Optimization is correct for all 32-bit unsigned x");
        }
        z3::SatResult::Unknown => {
            println!("Solver returned unknown");
        }
    }
}
