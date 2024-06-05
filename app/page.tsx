"use client";

import { useState, useEffect } from "react";
import { openDB } from "idb";
import Card from "./components/Card";
import Header from "./components/Header";
import Confetti from "react-confetti";
import { step1 } from "./steps/step1";
import { step2 } from "./steps/step2";
import { step3 } from "./steps/step3";
import { step4 } from "./steps/step4";
import { step5 } from "./steps/step5";

export default function Home() {
    const [activeCard, setActiveCard] = useState<number | null>(1);
    const [completedActions, setCompletedActions] = useState<boolean[]>([
        false,
        false,
        false,
        false,
        false,
    ]);
    const [publicWallet, setPublicWallet] = useState<string>("");
    const [anonWallet, setAnonWallet] = useState<string>("");
    const [nullifier, setNullifier] = useState<string>("");
    const [proof, setProof] = useState<string>("");
    const [showConfetti, setShowConfetti] = useState<boolean>(false);
    const [isDownloadComplete, setIsDownloadComplete] =
        useState<boolean>(false);

    useEffect(() => {
        if (completedActions.every((action) => action)) {
            setShowConfetti(true);
            const timer = setTimeout(() => {
                setShowConfetti(false);
            }, 5000);

            return () => {
                clearTimeout(timer);
            };
        }
    }, [completedActions]);

    useEffect(() => {
        const downloadAndStoreFiles = async () => {
            console.log("Checking and downloading files if necessary");
            const db = await openDB("my-database", 1, {
                upgrade(db) {
                    db.createObjectStore("files");
                },
            });

            const filesToDownload = [
                {
                    url: "https://storage.googleapis.com/plume-keys/plume_merkle_verify_vk_15_8.bin",
                    name: "vk",
                },
                {
                    url: "https://storage.googleapis.com/plume-keys/plume_merkle_verify_pk_15_8.bin",
                    name: "pk",
                },
            ];

            for (const file of filesToDownload) {
                const existingFile = await db.get("files", file.name);
                if (!existingFile) {
                    console.log(`Downloading ${file.name}`);
                    const response = await fetch(file.url);
                    const arrayBuffer = await response.arrayBuffer();
                    const uint8Array = new Uint8Array(arrayBuffer);
                    await db.put("files", uint8Array, file.name);
                    console.log(`Downloaded and stored ${file.name}`);
                } else {
                    console.log(
                        `${file.name} already exists, skipping download`
                    );
                }
            }
            console.log("File check and download complete");

            setIsDownloadComplete(true);
        };

        downloadAndStoreFiles();
    }, []);
    const toggleCard = (cardIndex: number) => {
        if (cardIndex === 1 || completedActions[cardIndex - 2]) {
            setActiveCard(activeCard === cardIndex ? null : cardIndex);
        }
    };

    const fetchKeysFromIndexedDB = async () => {
        try {
            const db = await openDB("my-database", 1);
            const pk = await db.get("files", "pk");
            const vk = await db.get("files", "vk");

            if (!pk || !vk) {
                throw new Error(
                    "Proving key or verifying key not found in IndexedDB"
                );
            }

            return { pk, vk };
        } catch (error) {
            console.error("Error fetching keys from IndexedDB:", error);
            throw error;
        }
    };

    return (
        <main className="flex-grow container mx-auto px-64 py-8">
            <Header />
            <div className="space-y-4">
                <Card
                    key={1}
                    item={1}
                    heading="Connect Public Wallet"
                    description="Connect your public wallet to which is eligible for the airdrop."
                    buttonLabel="Connect"
                    activeCard={activeCard}
                    completedActions={completedActions}
                    toggleCard={toggleCard}
                    handleAction={() =>
                        step1(
                            setPublicWallet,
                            completedActions,
                            setCompletedActions,
                            setActiveCard
                        )
                    }
                />
                <Card
                    key={2}
                    item={2}
                    heading="Generate a Unique Nullifier"
                    description="Generate a unique nullifier for your claiming wallet, this will be used to claim the airdrop and must be kept secret."
                    buttonLabel="Generate"
                    activeCard={activeCard}
                    completedActions={completedActions}
                    toggleCard={toggleCard}
                    handleAction={() =>
                        step2(
                            publicWallet,
                            setNullifier,
                            completedActions,
                            setCompletedActions,
                            setActiveCard
                        )
                    }
                />

                <Card
                    key={3}
                    item={3}
                    heading="Generate Claim Proof"
                    description="Generate a ZK proof to claim the airdrop."
                    buttonLabel="Generate"
                    activeCard={activeCard}
                    completedActions={completedActions}
                    toggleCard={toggleCard}
                    handleAction={() =>
                        fetchKeysFromIndexedDB().then((keys) => {
                            step3(
                                completedActions,
                                setCompletedActions,
                                setActiveCard,
                                setProof,
                                {
                                    nullifier,
                                    pk: keys.pk,
                                    vk: keys.vk,
                                }
                            );
                        })
                    }
                    disabled={!isDownloadComplete}
                />

                <Card
                    key={4}
                    item={4}
                    heading="Connect Anon Wallet"
                    description="Connect your anon wallet to which the airdrop will be claimed."
                    buttonLabel="Claim"
                    activeCard={activeCard}
                    completedActions={completedActions}
                    toggleCard={toggleCard}
                    handleAction={() =>
                        step4(
                            publicWallet,
                            setAnonWallet,
                            completedActions,
                            setCompletedActions,
                            setActiveCard
                        )
                    }
                />
                <Card
                    key={5}
                    item={5}
                    heading="Claim Airdrop"
                    description="Claim the airdrop using your anon wallet."
                    buttonLabel="Claim"
                    activeCard={activeCard}
                    completedActions={completedActions}
                    toggleCard={toggleCard}
                    handleAction={() =>
                        step5(
                            completedActions,
                            setCompletedActions,
                            setActiveCard
                        )
                    }
                />
            </div>
            {completedActions.every((action) => action) && (
                <div className="fixed inset-0 flex items-center justify-center z-50">
                    <div className="bg-white p-8 rounded-lg shadow-lg text-center">
                        <h2 className="text-gray-800 text-2xl font-bold mb-4">
                            Congratulations!
                        </h2>
                        <p className="text-gray-600 mb-4">
                            You have successfully claimed the airdrop.
                        </p>
                        <a
                            href="https://your-link-here"
                            target="blank"
                            rel="noopener noreferrer"
                            className="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded"
                        >
                            View Airdrop
                        </a>
                    </div>
                </div>
            )}
            {showConfetti && <Confetti recycle={false} />}
        </main>
    );
}
