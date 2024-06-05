import React from "react";
import { FaCheck } from "react-icons/fa";

interface CardProps {
    item: number;
    heading: string;
    description: string;
    buttonLabel: string;
    activeCard: number | null;
    completedActions: boolean[];
    toggleCard: (cardIndex: number) => void;
    handleAction: () => void;
    disabled?: boolean;
}

const Card: React.FC<CardProps> = ({
    item,
    heading,
    description,
    buttonLabel,
    activeCard,
    completedActions,
    toggleCard,
    handleAction,
    disabled = false,
}) => {
    const isCardDisabled = completedActions[item - 1];
    const isActionCompleted = completedActions[item - 1];

    return (
        <div
            className={`bg-white rounded shadow p-4 cursor-pointer ${
                activeCard === item ? "border-blue-500 border-2" : ""
            } ${
                item !== 1 && !completedActions[item - 2]
                    ? "opacity-50 pointer-events-none"
                    : ""
            } ${isCardDisabled ? "opacity-50 pointer-events-none" : ""}`}
            onClick={() => toggleCard(item)}
        >
            <div className="flex items-center justify-between">
                <h2 className="text-lg font-bold text-gray-800">{heading}</h2>
                <div
                    className={`w-6 h-6 rounded-full flex items-center justify-center ${
                        isActionCompleted ? "bg-green-500" : "bg-gray-300"
                    }`}
                >
                    {isActionCompleted && (
                        <FaCheck className="text-white" size={12} />
                    )}
                </div>
            </div>
            {activeCard === item && (
                <>
                    <p className="text-gray-600 mt-4 mb-4">{description}</p>
                    <div className="flex justify-between items-center">
                        <button
                            className={`text-white font-bold py-2 px-4 rounded ${
                                isCardDisabled || disabled
                                    ? "bg-gray-400 cursor-not-allowed"
                                    : "bg-blue-500 hover:bg-blue-600"
                            }`}
                            onClick={handleAction}
                            disabled={isCardDisabled || disabled}
                        >
                            {buttonLabel}
                        </button>
                    </div>
                </>
            )}
        </div>
    );
};

export default Card;
