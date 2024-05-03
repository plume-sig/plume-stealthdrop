mod utils;

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
        poseidon_hasher,
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

#[cfg(test)]
mod test {
    use halo2_base::{
        gates::RangeInstructions,
        halo2_proofs::{
            arithmetic::Field,
            halo2curves::{ bn256::Fr, secp256k1::{ Fq, Secp256k1, Secp256k1Affine } },
        },
        poseidon::hasher::{ spec::OptimizedPoseidonSpec, PoseidonHasher },
        utils::{ testing::base_test, BigPrimeField },
    };
    use halo2_ecc::{ ecc::EccChip, fields::FieldChip, secp256k1::{ FpChip, FqChip } };
    use plume_halo2::utils::gen_test_nullifier;
    use pse_poseidon::Poseidon;
    use rand::{ random, rngs::OsRng };

    use crate::{ prove_stealth_drop, utils::MerkleTree, StealthDropInput };

    fn generate_merkle_leaves<F: BigPrimeField>(
        hasher: &mut Poseidon<F, 3, 2>,
        n: usize
    ) -> (Vec<Fq>, Vec<Secp256k1Affine>, Vec<F>) {
        let mut secret_keys = Vec::<Fq>::new();
        let mut public_keys = Vec::<Secp256k1Affine>::new();
        let mut leaves = Vec::<F>::new();

        for _ in 0..n {
            let sk = Fq::random(OsRng);
            let pk = Secp256k1Affine::from(Secp256k1::generator() * sk);

            let pk_x = pk.x
                .to_bytes()
                .to_vec()
                .chunks(11)
                .into_iter()
                .map(|chunk| F::from_bytes_le(chunk))
                .collect::<Vec<_>>();
            let pk_y = pk.y
                .to_bytes()
                .to_vec()
                .chunks(11)
                .into_iter()
                .map(|chunk| F::from_bytes_le(chunk))
                .collect::<Vec<_>>();

            hasher.update(pk_x.as_slice());
            hasher.update(pk_y.as_slice());

            secret_keys.push(sk);
            public_keys.push(pk);
            leaves.push(hasher.squeeze_and_reset());
        }

        (secret_keys, public_keys, leaves)
    }

    #[test]
    fn test_stealth_drop_circuit() {
        let mut native_hasher = Poseidon::<Fr, 3, 2>::new(8, 57);

        let tree_size = 8;
        let (secret_keys, public_keys, leaves) = generate_merkle_leaves(
            &mut native_hasher,
            tree_size
        );
        let merkle_tree = MerkleTree::new(&mut native_hasher, leaves).unwrap();
        let merkle_root = merkle_tree.get_root();

        let message = b"zk-airdrop";

        let random_index = random::<usize>() % tree_size;
        let sk = secret_keys[random_index];
        let pk = public_keys[random_index];
        let (merkle_proof, merkle_proof_path) = merkle_tree.get_proof(random_index);

        let (nullifier, s, c) = gen_test_nullifier(&sk, message);

        base_test()
            .k(15)
            .lookup_bits(14)
            .expect_satisfied(true)
            .run(|ctx, range| {
                let fp_chip = FpChip::<Fr>::new(range, 88, 3);
                let fq_chip = FqChip::<Fr>::new(range, 88, 3);
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
                    .map(|v| ctx.load_witness(Fr::from(*v as u64)))
                    .collect::<Vec<_>>();
                let public_key = ecc_chip.load_private_unchecked(ctx, (pk.x, pk.y));

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
            })
    }
}
