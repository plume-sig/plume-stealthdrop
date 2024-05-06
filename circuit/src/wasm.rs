use halo2_base::{
    halo2_proofs::{ arithmetic::CurveAffine, halo2curves::bn256::Fr },
    utils::ScalarField,
};
use halo2_ecc::fields::FieldChip;
use halo2_wasm::{
    halo2_base::{
        gates::{ circuit::builder::BaseCircuitBuilder, RangeChip, RangeInstructions },
        poseidon::hasher::{ spec::OptimizedPoseidonSpec, PoseidonHasher },
    },
    halo2_ecc::secp256k1::{ FpChip, FqChip },
    halo2lib::ecc::{ EccChip, Secp256k1Affine, Secp256k1Fp as Fp, Secp256k1Fq as Fq },
    Halo2Wasm,
};
use num_bigint::BigUint;
use num_traits::Num;
use std::{ cell::RefCell, rc::Rc };
use wasm_bindgen::prelude::*;
use tsify::Tsify;
use serde::{ Serialize, Deserialize };

use crate::{ prove_stealth_drop, StealthDropInput };

#[wasm_bindgen]
pub struct StealthDropWasmCircuit {
    range: RangeChip<Fr>,
    builder: Rc<RefCell<BaseCircuitBuilder<Fr>>>,
}

#[derive(Tsify, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct CircuitInput {
    // Public
    pub merkle_root: String,
    pub nullifier: (String, String),
    pub s: String,

    // Private
    pub merkle_proof: Vec<String>,
    pub merkle_proof_path: Vec<String>,
    pub c: String,
    pub message: String,
    pub public_key: (String, String),
}

#[wasm_bindgen]
impl StealthDropWasmCircuit {
    #[wasm_bindgen(constructor)]
    pub fn new(circuit: &Halo2Wasm) -> Self {
        let builder = Rc::clone(&circuit.circuit);
        let lookup_bits = match builder.borrow_mut().lookup_bits() {
            Some(x) => x,
            None => panic!("Lookup bits not found"),
        };
        let lookup_manager = builder.borrow_mut().lookup_manager().clone();
        let range = RangeChip::<Fr>::new(lookup_bits, lookup_manager);
        StealthDropWasmCircuit {
            range,
            builder: Rc::clone(&circuit.circuit),
        }
    }

    pub fn run(&mut self, input: CircuitInput) {
        let merkle_root = Fr::from_bytes_le(
            BigUint::from_str_radix(&input.merkle_root, 10).unwrap().to_bytes_le().as_slice()
        );

        let nullifier_x = Fp::from_bytes_le(
            BigUint::from_str_radix(&input.nullifier.0, 10).unwrap().to_bytes_le().as_slice()
        );
        let nullifier_y = Fp::from_bytes_le(
            BigUint::from_str_radix(&input.nullifier.1, 10).unwrap().to_bytes_le().as_slice()
        );
        let nullifier = Secp256k1Affine::from_xy(nullifier_x, nullifier_y).unwrap();

        let s = Fq::from_bytes_le(
            BigUint::from_str_radix(&input.s, 10).unwrap().to_bytes_le().as_slice()
        );

        let merkle_proof = input.merkle_proof
            .iter()
            .map(|v| {
                Fr::from_bytes_le(BigUint::from_str_radix(v, 10).unwrap().to_bytes_le().as_slice())
            })
            .collect::<Vec<_>>();
        let merkle_proof_path = input.merkle_proof_path
            .iter()
            .map(|v| {
                Fr::from_bytes_le(BigUint::from_str_radix(v, 10).unwrap().to_bytes_le().as_slice())
            })
            .collect::<Vec<_>>();

        let c = Fq::from_bytes_le(
            BigUint::from_str_radix(&input.c, 10).unwrap().to_bytes_le().as_slice()
        );

        let message = input.message
            .as_bytes()
            .iter()
            .map(|x| Fr::from(*x as u64))
            .collect::<Vec<_>>();

        let public_key_x = Fp::from_bytes_le(
            BigUint::from_str_radix(&input.public_key.0, 10).unwrap().to_bytes_le().as_slice()
        );
        let public_key_y = Fp::from_bytes_le(
            BigUint::from_str_radix(&input.public_key.1, 10).unwrap().to_bytes_le().as_slice()
        );
        let public_key = Secp256k1Affine::from_xy(public_key_x, public_key_y).unwrap();

        let mut builder_borrow = self.builder.borrow_mut();
        let ctx = builder_borrow.main(0);
        let range = &self.range;

        let fp_chip = FpChip::<Fr>::new(&range, 88, 3);
        let fq_chip = FqChip::<Fr>::new(&range, 88, 3);
        let ecc_chip = EccChip::<Fr, FpChip<Fr>>::new(&fp_chip);

        let mut poseidon_hasher = PoseidonHasher::<Fr, 3, 2>::new(
            OptimizedPoseidonSpec::new::<8, 57, 0>()
        );
        poseidon_hasher.initialize_consts(ctx, range.gate());

        let merkle_root = ctx.load_witness(merkle_root);
        let nullifier = ecc_chip.load_private_unchecked(ctx, (nullifier.x, nullifier.y));
        let s = fq_chip.load_private(ctx, s);
        let merkle_proof = merkle_proof
            .iter()
            .map(|v| ctx.load_witness(*v))
            .collect::<Vec<_>>();
        let merkle_proof_path = merkle_proof_path
            .iter()
            .map(|v| ctx.load_witness(*v))
            .collect::<Vec<_>>();
        let c = fq_chip.load_private(ctx, c);
        let message = message
            .iter()
            .map(|v| ctx.load_witness(*v))
            .collect::<Vec<_>>();
        let public_key = ecc_chip.load_private_unchecked(ctx, (public_key.x, public_key.y));

        let input = StealthDropInput {
            merkle_root,
            nullifier,
            s,
            merkle_proof,
            merkle_proof_path,
            c,
            message,
            public_key,
        };

        prove_stealth_drop(ctx, &ecc_chip, &poseidon_hasher, 4, 4, input)
    }
}
