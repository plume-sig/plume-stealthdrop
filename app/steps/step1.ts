import { handleAction } from "../utils";

export const step1 = async (
    setPublicWallet: (wallet: string) => void,
    completedActions: boolean[],
    setCompletedActions: (actions: boolean[]) => void,
    setActiveCard: (card: number | null) => void
) => {
    try {
        if (typeof window.ethereum !== "undefined") {
            await window.ethereum.request({
                method: "eth_requestAccounts",
                params: [],
            });

            let publicWallet = (
                await window.ethereum.request({
                    method: "eth_accounts",
                    params: [],
                })
            )[0];
            console.log("publicWallet", publicWallet);
            setPublicWallet(publicWallet);

            handleAction(
                1,
                completedActions,
                setCompletedActions,
                setActiveCard
            );
        } else {
            console.log("Metamask is not installed");
        }
    } catch (error) {
        console.error("Error connecting wallet:", error);
    }
};
