use serde::{ Serialize, Deserialize };
use crate::math::{ FiniteField };
use crate::crypto::{ BatchMerkleProof };
use crate::stark::{ Accumulator, fri::FriProof, TraceState, ProofOptions };
use crate::utils::{ uninit_vector, as_bytes };

// TYPES AND INTERFACES
// ================================================================================================

// TODO: custom serialization should reduce size by 5% - 10%
#[derive(Clone, Serialize, Deserialize)]
pub struct StarkProof<T: FiniteField + Accumulator> {
    trace_root          : [u8; 32],
    domain_depth        : u8,
    trace_nodes         : Vec<Vec<[u8; 32]>>,
    trace_evaluations   : Vec<Vec<T>>,
    constraint_root     : [u8; 32],
    constraint_proof    : BatchMerkleProof,
    deep_values         : DeepValues<T>,
    degree_proof        : FriProof<T>,
    pow_nonce           : u64,
    options             : ProofOptions
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeepValues<T: FiniteField + Accumulator> {
    pub trace_at_z1     : Vec<T>,
    pub trace_at_z2     : Vec<T>,
}

// STARK PROOF IMPLEMENTATION
// ================================================================================================
impl <T> StarkProof<T>
    where T: FiniteField + Accumulator
{
    pub fn new(
        trace_root          : &[u8; 32],
        trace_proof         : BatchMerkleProof,
        trace_evaluations   : Vec<Vec<T>>,
        constraint_root     : &[u8; 32],
        constraint_proof    : BatchMerkleProof,
        deep_values         : DeepValues<T>,
        degree_proof        : FriProof<T>,
        pow_nonce           : u64,
        options             : &ProofOptions ) -> StarkProof<T>
    {
        return StarkProof {
            trace_root          : *trace_root,
            domain_depth        : trace_proof.depth,
            trace_nodes         : trace_proof.nodes,
            trace_evaluations   : trace_evaluations,
            constraint_root     : *constraint_root,
            constraint_proof    : constraint_proof,
            deep_values         : deep_values,
            degree_proof        : degree_proof,
            pow_nonce           : pow_nonce,
            options             : options.clone()
        };
    }

    pub fn trace_root(&self) -> &[u8; 32] {
        return &self.trace_root;
    }

    pub fn options(&self) -> &ProofOptions {
        return &self.options;
    }

    pub fn domain_size(&self) -> usize {
        return usize::pow(2, self.domain_depth as u32);
    }

    pub fn trace_proof(&self) -> BatchMerkleProof {

        let hash = self.options.hash_function();
        let mut hashed_states = uninit_vector::<[u8; 32]>(self.trace_evaluations.len());
        for i in 0..self.trace_evaluations.len() {
            hash(as_bytes(&self.trace_evaluations[i]), &mut hashed_states[i]);
        }

        return BatchMerkleProof {
            nodes   : self.trace_nodes.clone(),
            values  : hashed_states,
            depth   : self.domain_depth,
         };
    }

    pub fn constraint_root(&self) -> &[u8; 32] {
        return &self.constraint_root;
    }

    pub fn constraint_proof(&self) -> BatchMerkleProof {
        return self.constraint_proof.clone();
    }

    pub fn degree_proof(&self) -> &FriProof<T> {
        return &self.degree_proof;
    }

    pub fn trace_evaluations(&self) -> &[Vec<T>] {
        return &self.trace_evaluations;
    }

    pub fn trace_length(&self) -> usize {
        return self.domain_size() / self.options.extension_factor();
    }

    pub fn stack_depth(&self) -> usize {
        return TraceState::<T>::compute_stack_depth(self.trace_evaluations[0].len());
    }

    pub fn pow_nonce(&self) -> u64 {
        return self.pow_nonce;
    }

    // DEEP VALUES
    // -------------------------------------------------------------------------------------------
    pub fn get_state_at_z1(&self) -> TraceState<T> {
        return TraceState::from_raw_state(self.deep_values.trace_at_z1.clone());
    }

    pub fn get_state_at_z2(&self) -> TraceState<T> {
        return TraceState::from_raw_state(self.deep_values.trace_at_z2.clone());
    }
}