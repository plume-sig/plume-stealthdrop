import { handleAction } from "../utils";

export const step2 = async (
    publicWallet: string,
    setNullifier: (nullifier: string) => void,
    completedActions: boolean[],
    setCompletedActions: (actions: boolean[]) => void,
    setActiveCard: (card: number | null) => void
) => {
    try {
        if (typeof window.ethereum !== "undefined") {
            const message = "CLAIM MY MONIII";
            let nullifier = await window.ethereum.request({
                method: "eth_getPlumeSignature",
                params: [message, publicWallet],
            });
            setNullifier({ ...nullifier, message });
            console.log("nullifier", nullifier);

            handleAction(
                2,
                completedActions,
                setCompletedActions,
                setActiveCard
            );
        } else {
            console.log("Metamask is not installed");
        }
    } catch (error) {
        console.error("Error generating nullifier:", error);
    }
};
