import { handleAction } from "../utils";

export const step3 = async (
    completedActions: boolean[],
    setCompletedActions: (actions: boolean[]) => void,
    setActiveCard: (card: number | null) => void,
    setProof: (proof: string) => void,
    step3Data: any
) => {
    try {
        const worker = new Worker(
            new URL("../workers/step3Worker.ts", import.meta.url)
        );

        worker.postMessage({
            action: "generateProof",
            data: {
                provingKey: step3Data.pk,
                verifyingKey: step3Data.vk,
                nullifier: step3Data.nullifier,
            },
        });

        worker.onmessage = (event) => {
            const { status, message, proof } = event.data;
            if (status === "success") {
                setProof(proof);
                setCompletedActions([...completedActions, true]);
                setActiveCard(null);
                handleAction(
                    3,
                    completedActions,
                    setCompletedActions,
                    setActiveCard
                );
            } else {
                console.error(
                    "Error generating claim proof in worker:",
                    message
                );
            }
            worker.terminate();
        };

        worker.onerror = (error) => {
            console.error("Worker error:", error);
            worker.terminate();
        };
    } catch (error) {
        console.error("Error generating claim proof:", error);
    }
};
