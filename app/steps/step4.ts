import { handleAction } from "../utils";

export const step4 = async (
    publicWallet: string,
    setAnonWallet: (wallet: string) => void,
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

            let currentWallet = (
                await window.ethereum.request({
                    method: "eth_accounts",
                    params: [],
                })
            )[0];
            console.log("currentWallet", currentWallet);

            if (currentWallet === publicWallet) {
                alert(
                    "Please connect a different wallet than the public wallet."
                );
                return;
            }

            setAnonWallet(currentWallet);
            handleAction(
                4,
                completedActions,
                setCompletedActions,
                setActiveCard
            );
        } else {
            console.log("Metamask is not installed");
        }
    } catch (error) {
        console.error("Error connecting anon wallet:", error);
    }
};
