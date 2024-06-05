import { handleAction } from "../utils";

export const step5 = async (
    completedActions: boolean[],
    setCompletedActions: (actions: boolean[]) => void,
    setActiveCard: (card: number | null) => void
) => {
    try {
        handleAction(5, completedActions, setCompletedActions, setActiveCard);
    } catch (error) {
        console.error("Error claiming airdrop:", error);
    }
};
