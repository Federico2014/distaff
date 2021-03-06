use std::mem;
use crate::math::{ FiniteField, polynom, quartic };
use crate::crypto::{ MerkleTree, BatchMerkleProof };
use crate::stark::{ ProofOptions };

use super::{ FriProof, FriLayer, utils };

// VERIFIER
// ================================================================================================

pub fn verify<T: FiniteField>(
    proof       : &FriProof<T>,
    evaluations : &[T],
    positions   : &[usize],
    max_degree  : usize,
    options     : &ProofOptions) -> Result<bool, String>
{

    let domain_size = usize::pow(2, proof.layers[0].depth as u32) * 4;
    let domain_root = T::get_root_of_unity(domain_size);

    // powers of the given root of unity 1, p, p^2, p^3 such that p^4 = 1
    let quartic_roots = [
        T::from_usize(1),
        T::exp(domain_root, T::from_usize(domain_size / 4)),
        T::exp(domain_root, T::from_usize(domain_size / 2)),
        T::exp(domain_root, T::from_usize(domain_size * 3 / 4)),
    ];

    // 1 ----- verify the recursive components of the FRI proof -----------------------------------
    let mut domain_root = domain_root;
    let mut domain_size = domain_size;
    let mut max_degree_plus_1 = max_degree + 1;
    let mut positions = positions.to_vec();
    let mut evaluations = evaluations.to_vec();

    for (depth, layer) in proof.layers.iter().enumerate() {

        let mut augmented_positions = utils::get_augmented_positions(&positions, domain_size);
        let column_values = get_column_values(&layer.values, &positions, &augmented_positions, domain_size);
        if evaluations != column_values {
            return Err(format!("evaluations did not match column value at depth {}", depth));
        }

        // verify Merkle proof for the layer
        let merkle_proof = build_layer_merkle_proof(&layer, options);
        if !MerkleTree::verify_batch(&layer.root, &augmented_positions, &merkle_proof, options.hash_function()) {
            return Err(format!("verification of Merkle proof failed at layer {}", depth));
        }

        // build a set of x for each row polynomial
        let mut xs = Vec::with_capacity(augmented_positions.len());
        for &i in augmented_positions.iter() {
            let xe = T::exp(domain_root, T::from_usize(i));
            xs.push([
                T::mul(quartic_roots[0], xe),
                T::mul(quartic_roots[1], xe),
                T::mul(quartic_roots[2], xe),
                T::mul(quartic_roots[3], xe)
            ]);
        }

        // interpolate x and y values into row polynomials
        let row_polys = quartic::interpolate_batch(&xs, &layer.values);

        // calculate the pseudo-random x coordinate
        let special_x = T::prng(layer.root);

        // check that when the polynomials are evaluated at x, the result is equal to the corresponding column value
        evaluations = quartic::evaluate_batch(&row_polys, special_x);

        // update variables for the next iteration of the loop
        domain_root = T::exp(domain_root, T::from_usize(4));
        max_degree_plus_1 = max_degree_plus_1 / 4;
        domain_size = domain_size / 4;
        mem::swap(&mut positions, &mut augmented_positions);
    }

    // 2 ----- verify the remainder of the FRI proof ----------------------------------------------
    
    for (&position, evaluation) in positions.iter().zip(evaluations) {
        if proof.rem_values[position] != evaluation {
            return Err(String::from("remainder values are inconsistent with values of the last column"));
        }
    }

    // make sure the remainder values satisfy the degree
    return verify_remainder(&proof.rem_values, max_degree_plus_1, domain_root, options.extension_factor());
}

fn verify_remainder<T>(remainder: &[T], max_degree_plus_1: usize, domain_root: T, extension_factor: usize) -> Result<bool, String>
    where T: FiniteField
{
    if max_degree_plus_1 > remainder.len() {
        return Err(String::from("remainder degree is greater than number of remainder values"));
    }

    // exclude points which should be skipped during evaluation
    let mut positions = Vec::new();
    for i in 0..remainder.len() {
        if i % extension_factor != 0 {
            positions.push(i);
        }
    }

    // pick a subset of points from the remainder and interpolate them into a polynomial
    let domain = T::get_power_series(domain_root, remainder.len());
    let mut xs = Vec::with_capacity(max_degree_plus_1);
    let mut ys = Vec::with_capacity(max_degree_plus_1);
    for i in 0..max_degree_plus_1 {
        let p = positions[i];
        xs.push(domain[p]);
        ys.push(remainder[p]);
    }
    let poly = polynom::interpolate(&xs, &ys);

    // check that polynomial evaluates correctly for all other points in the remainder
    for i in max_degree_plus_1..positions.len() {
        let p = positions[i];
        if polynom::eval(&poly, domain[p]) != remainder[p] {
            return Err(format!("remainder is not a valid degree {} polynomial", max_degree_plus_1 - 1));
        }
    }

    return Ok(true);
}

// HELPER FUNCTIONS
// ================================================================================================
fn get_column_values<T>(values: &Vec<[T; 4]>, positions: &[usize], augmented_positions: &[usize], column_length: usize) -> Vec<T>
    where T: FiniteField
{
    let row_length = column_length / 4;

    let mut result = Vec::new();
    for position in positions {
        let idx = augmented_positions.iter().position(|&v| v == position % row_length).unwrap();
        let value = values[idx][position / row_length];
        result.push(value);
    }

    return result;
}

fn build_layer_merkle_proof<T>(layer: &FriLayer<T>, options: &ProofOptions) -> BatchMerkleProof
    where T: FiniteField
{
    return BatchMerkleProof {
        values  : utils::hash_values(&layer.values, options.hash_function()),
        nodes   : layer.nodes.clone(),
        depth   : layer.depth
    };
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {
    
    use crate::{ F64, FiniteField, polynom };

    #[test]
    fn verify_remainder() {
        let degree_plus_1: usize = 32;
        let root = F64::get_root_of_unity(degree_plus_1 * 2);
        let extension_factor = 16;

        let mut remainder = F64::rand_vector(degree_plus_1);
        remainder.resize(degree_plus_1 * 2, 0);
        polynom::eval_fft(&mut remainder, true);

        // check against exact degree
        let result = super::verify_remainder(&remainder, degree_plus_1, root, extension_factor);
        assert_eq!(Ok(true), result);

        // check against higher degree
        let result = super::verify_remainder(&remainder, degree_plus_1 + 1, root, extension_factor);
        assert_eq!(Ok(true), result);

        // check against lower degree
        let degree_plus_1 = degree_plus_1 - 1;
        let result = super::verify_remainder(&remainder, degree_plus_1, root, extension_factor);
        let err_msg = format!("remainder is not a valid degree {} polynomial", degree_plus_1 - 1);
        assert_eq!(Err(err_msg), result);
    }

}