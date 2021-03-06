use crate::math::{ FiniteField };

mod hash128;

// TYPES AND INTERFACES
// ================================================================================================
pub trait Hasher: FiniteField {

    const CYCLE_LENGTH  : usize;
    const NUM_ROUNDS    : usize;
    const STATE_WIDTH   : usize;
    const STATE_RATE    : usize;
    const DIGEST_SIZE   : usize;

    fn digest(values: &[Self]) -> Vec<Self> {
        assert!(values.len() <= Self::STATE_RATE,
            "expected no more than {}, but received {}", Self::STATE_RATE, values.len());

        let mut state = vec![Self::ZERO; Self::STATE_WIDTH];
        state[..values.len()].copy_from_slice(values);
        state.reverse();

        for i in 0..Self::NUM_ROUNDS {
            Self::apply_round(&mut state, i);
        }

        state.reverse();
        return state[..Self::DIGEST_SIZE].to_vec();
    }

    fn apply_round(state: &mut [Self], step: usize) {
        
        let ark_idx = step % Self::CYCLE_LENGTH;

        // apply Rescue round
        Self::add_constants(state, ark_idx, 0);
        Self::apply_sbox(state);
        Self::apply_mds(state);

        Self::add_constants(state, ark_idx, Self::STATE_WIDTH);
        Self::apply_inv_sbox(state);
        Self::apply_mds(state);
    }

    fn add_constants(state: &mut[Self], idx: usize, offset: usize);

    fn apply_sbox(state: &mut [Self]);
    fn apply_inv_sbox(state: &mut[Self]);

    fn apply_mds(state: &mut[Self]);
    fn apply_inv_mds(state: &mut[Self]);

    fn get_extended_constants(extension_factor: usize) -> (Vec<Vec<Self>>, Vec<Vec<Self>>);
}