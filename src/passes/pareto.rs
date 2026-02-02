

// Input: vectors of the same length 
    // (coming from rank vectors of monomorphizations of a given generic functions)
// Output – variants: 
    // Pareto improvement found
    // Monomorphization set saturated
        // Or wait, how do we detect a "cycle" that makes the same, converging monos 
    // Neither happened yet

// Insight: the bad things is when the current vector Pareto-improves on any old one
    // Pareto-nonimprovement is okay (incommesurable or decrease) 

// Algorithm
// Input for an iteration: previous vectors, current vector  
// Iteration pseudocode:
    // See if the current vector dominates any previous ones, panic if yes
    //  



fn dominates(a: &[usize], b: &[usize]) -> bool {
    if a.len() != b.len() {
        panic!("Attempted to compare vectors of different length");
    }
    let mut strict_improvement = false;
    
    for i in 0..a.len() {
        if b[i] > a[i] {
            return false;
        } else if b[i] < a[i] {
            strict_improvement = true;
        }
    }
    return strict_improvement;
}
