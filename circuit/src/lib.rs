use halo2_base::{
    gates::{ GateChip, GateInstructions },
    poseidon::hasher::PoseidonHasher,
    utils::BigPrimeField,
    AssignedValue,
    Context,
};
use halo2_ecc::{ bigint::ProperCrtUint, ecc::EcPoint, fields::FieldChip, secp256k1::Secp256k1Chip };
use plume_halo2::plume::{ verify_plume, PlumeInput };

pub struct StealthDropInput<F: BigPrimeField> {
    // Public
    pub merkle_root: AssignedValue<F>,
    pub nullifier: EcPoint<F, ProperCrtUint<F>>,
    pub s: ProperCrtUint<F>,

    // Private
    pub merkle_proof: Vec<AssignedValue<F>>,
    pub merkle_proof_path: Vec<AssignedValue<F>>,
    pub c: ProperCrtUint<F>,
    pub message: Vec<AssignedValue<F>>,
    pub public_key: EcPoint<F, ProperCrtUint<F>>,
}

pub fn dual_mux<F: BigPrimeField>(
    ctx: &mut Context<F>,
    gate: &GateChip<F>,
    a: &AssignedValue<F>,
    b: &AssignedValue<F>,
    switch: &AssignedValue<F>
) -> [AssignedValue<F>; 2] {
    gate.assert_bit(ctx, *switch);

    let a_sub_b = gate.sub(ctx, *a, *b);
    let b_sub_a = gate.sub(ctx, *b, *a);

    let left = gate.mul_add(ctx, a_sub_b, *switch, *b); // left = (a-b)*s + b;
    let right = gate.mul_add(ctx, b_sub_a, *switch, *a); // right = (b-a)*s + a;

    [left, right]
}

pub fn verify_membership_proof<F: BigPrimeField, const T: usize, const RATE: usize>(
    ctx: &mut Context<F>,
    gate: &GateChip<F>,
    hasher: &PoseidonHasher<F, T, RATE>,
    root: &AssignedValue<F>,
    leaf: &AssignedValue<F>,
    proof: &[AssignedValue<F>],
    helper: &[AssignedValue<F>]
) {
    let mut computed_hash = ctx.load_witness(*leaf.value());

    for (proof_element, helper) in proof.iter().zip(helper.iter()) {
        let inp = dual_mux(ctx, gate, &computed_hash, proof_element, helper);
        computed_hash = hasher.hash_fix_len_array(ctx, gate, &inp);
    }
    ctx.constrain_equal(&computed_hash, root);
}

pub fn prove_stealth_drop<F: BigPrimeField>(
    ctx: &mut Context<F>,
    secp256k1_chip: &Secp256k1Chip<'_, F>,
    poseidon_hasher: &PoseidonHasher<F, 3, 2>,
    fixed_window_bits: usize,
    var_window_bits: usize,
    input: StealthDropInput<F>
) {
    let base_chip = secp256k1_chip.field_chip();
    let gate = base_chip.gate();

    let leaf_preimage = [input.public_key.x().limbs(), input.public_key.y().limbs()].concat();
    let leaf = poseidon_hasher.hash_fix_len_array(ctx, gate, &leaf_preimage[..]);

    verify_membership_proof(
        ctx,
        gate,
        &poseidon_hasher,
        &input.merkle_root,
        &leaf,
        &input.merkle_proof,
        &input.merkle_proof_path
    );

    let plume_input = PlumeInput {
        nullifier: input.nullifier,
        s: input.s,
        c: input.c,
        pk: input.public_key,
        m: input.message,
    };

    verify_plume(
        ctx,
        secp256k1_chip,
        poseidon_hasher,
        fixed_window_bits,
        var_window_bits,
        plume_input
    )
}
