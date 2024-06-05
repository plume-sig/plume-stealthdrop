import { fetchMerkleData, getKzgParams } from "../utils";
import init, {
    initThreadPool,
    initPanicHook,
    Halo2Wasm,
    Circuit,
} from "plume-wasm";

self.onmessage = async (e) => {
    const { action, data } = e.data;
    if (action === "generateProof") {
        const { provingKey, verifyingKey, nullifier } = data;
        try {
            await init();
            console.log("Wasm initialized");

            initPanicHook();
            console.log("Panic hook initialized");

            await initThreadPool(navigator.hardwareConcurrency);
            console.log("Thread pool initialized");

            const halo2wasm = new Halo2Wasm();
            console.log("Halo2Wasm instance created");

            halo2wasm.config({
                k: 15,
                numAdvice: 69,
                numLookupAdvice: 8,
                numInstance: 1,
                numLookupBits: 14,
                numVirtualInstance: 1,
            });
            console.log("Halo2Wasm configured");

            const circuit = new Circuit(halo2wasm);
            console.log("Circuit instance created");

            circuit.plumeVerify({
                nullifier: nullifier.plume,
                s: nullifier.s,
                c: nullifier.c,
                message: nullifier.message,
                publicKey: nullifier.publicKey,
            });
            console.log("PlumeVerify completed");

            const { merkleProof, proofHelper, root } = await fetchMerkleData(
                "https://storage.googleapis.com/plume-keys/merkle_tree_8.json",
                0
            );

            circuit.merkleVerify({
                root,
                publicKey: nullifier.publicKey,
                proof: merkleProof,
                proofHelper,
            });
            console.log("MerkleVerify completed");

            halo2wasm.useInstances();
            console.log("Using instances");

            let instanceValues = halo2wasm.getInstanceValues(0);
            console.log("Instance values:", instanceValues);

            let params = await getKzgParams(15);
            if (params instanceof Uint8Array) {
                halo2wasm.loadParams(params);
                console.log("KZG params loaded");
            } else {
                console.error("Invalid KZG params format");
            }

            halo2wasm.loadVk(verifyingKey);
            console.log("Verification key loaded");

            halo2wasm.loadPk(provingKey);
            console.log("Proving key loaded");

            const proofStart = performance.now();
            let proof = halo2wasm.prove();
            const proofEnd = performance.now();
            console.log(
                "Proof generated:",
                proof,
                "in",
                (proofEnd - proofStart) / 1000,
                "seconds"
            );

            const verifyStart = performance.now();
            halo2wasm.verify(proof);
            const verifyEnd = performance.now();
            console.log(
                "Proof verified in",
                (verifyEnd - verifyStart) / 1000,
                "seconds"
            );

            self.postMessage({
                status: "success",
                message: "Proof generated and verified successfully",
                proof,
            });
        } catch (error) {
            console.error("Error running main function:", error);
            self.postMessage({
                status: "error",
                message: "Error running main function",
            });
        }
    }
};
