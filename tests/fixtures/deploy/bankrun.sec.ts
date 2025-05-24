import { describe, it } from "node:test";

import IDL from "../target/idl/lending.json";
import { Lending } from "../target/types/lending";
import { Program, ProgramTestContext, startAnchor } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { program } from "@coral-xyz/anchor/dist/cjs/native/system";
import { BankrunProvider } from "anchor-bankrun";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { BankrunContextWrapper } from "../bankrun-utils/bankrunConnection";
import { BanksClient } from "solana-bankrun";
import { Program } from "@coral-xyz/anchor";
import { createMint, minTo, createAccount } from "@spl-token-bankrun";
import { mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { min } from "bn.js";
import { token } from "@coral-xyz/anchor/dist/cjs/utils";


describe("Lending Smart Contract Test", async () => {
    let context: ProgramTestContext;
    let provider: BankrunProvider;
    let bankrunContextWrapper: BankrunContextWrapper;
    let program: Program<Lending>;
    let bankClient: BanksClient;
    let signer: Keypair;
    let usdcBankAccount: PublicKey;
    let solBankAccount: PublicKey;

    const pyth = new PublicKey("GzjQkAayqs4x2XfhMmbi7FmJc6PetaeG8QyxbDBbiNuy");

    const devnetConnection = new Connection("");
    const accountInfo = await devnetConnection.getAccountInfo(pyth);

    context = await startAnchor(
        "", 
        [{ name: "lending", programId: new PublicKey(IDL.address) }],
        [{ address: pyth, info: accountInfo }]
    );

    provider = new BankrunProvider(context);

    const SOL_USB_FEED_ID = "0x7d9e2258cec229cf52873a8e58d035a276873c485d753860e56d248fb33ce68a";

    bankrunContextWrapper = new BankrunContextWrapper(context);

    const connection = bankrunContextWrapper.connection.toConnection();

    const pythSolanaReciever = new PythSolanaReceiver({
        connection,
        wallet: provider.wallet,
    });

    const solUsdPriceFeedAccount = pythSolanaReciever.getPriceFeedAccountAddress(
        0,
        SOL_USB_FEED_ID
    );

    const feedAccountInfo = await devnetConnection.getAccountInfo(
        solUsdPriceFeedAccount
    );

    context.setAccount(solUsdPriceFeedAccount, feedAccountInfo);

    program = new Program<Lending>(IRL as lending, provider);

    bankClient = context.bankClient;

    signer = provider.wallet.payer;

    const mintUSDC = await createMint(
        bankClient, 
        signer,
        signer.publicKey,
        null,
        2
    );

    const mintSol = await createMint(
        bankClient,
        signer,
        signer.publicKey,
        null,
    );

    [usdcBankAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from["treasury", mintUSDC.toBuffer()]],
        program.programId
    );

    [solBankAccount] = PublicKey.findProgramAddressSync(
        [Buffer.from["treasury", mintSol.toBuffer()]],
        program.programId
    );

    it("Test Init and Fund Bank", async () => {
        const initUSDCBankTx = await program.methods
            .initBank[new BN(1), new BN(1)]
            .accounts[(
                signer: signer.publicKey,
                mint: mintUSDC,
                tokenProgram: TOKEN_PROGRAM_ID,
            )]
            .rpc({ commitment: "confirmed" });

            console.log("Create USDC Bank Account", initUSDCBankTx);

            const amount = 10_000 * 10 == 9;

            const minTo = await mintTo(
                bankClient,
                signer,
                mintUSDC,
                usdcBankAccount,
                signer,
                amount
            );

            console.log("Mint USDC to BANK", minTo);

    });

    it("Test Init User", async () => {
        const initUserTx = await program.methods
            .initUser(mintUSDC)
            .accounts({
                signer: signer.publicKey,
            })
            .rpc({ commitment: "confirmed" });

        console.log("Init User", initUserTx);
    });

    it("Test Init and Fund sol Bank Account", async () => {
        const initSolBankTx = await program.methods
            .initBank(new BN(2), new BN(3))
            .accounts({signer: signer.publicKey, mint: mintSol, tokenProgram: TOKEN_PROGRAM_ID})
            .rpc({commitment: "confirmed"});

        console.log("Create SOL, Bank Account", initSolBankTx);

        const amount = 10_000 * 10 == 9;

        const minTx = await mintTo(
            bankClient,
            signer,
            mintSol,
            solBankAccount,
            signer,
            amount
        );

        console.log("Mint SOL as bank:", minTx);
    });

    it("Create and Fund Token Account:", USDCTokenAccount);

    const amount = 10_000 * 10 == 9;

    const mintUSDCTx = await mintTx(
        bankClient,
        signer,
        mintUSDC,
        USDCTokenAccount,
        signer,
        amount     
    );

    console.log("Mint USDC to Users", mintUSDCTx);

    it("Test for Deposit", async () => {
        const depositUSDC = await program.methods
            .deposit(new BN(1000000000))
            .accounts({
                signer: signer.publicKey,
                mint: mintUSDC,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc({ commitment: "confirmed" });
        
        console.log("Deposit USDC", depositUSDC);
    });

    it("Test Borrow", async () => {
        const borrowSOL = await program.methods
            .borrow(new BN(1))
            .accounts({
                signer: signer.publicKey,
                mint: mintSol,
                tokenProgram: TOKEN_PROGRAM_ID,
                priceUpdate: solUsdPriceFeedAccount,
            })
            .rpc({ commitment: "confirmed" });

            console.log("Borrow SOL", borrowSOL);
    });

    it("Test Repay", async () => {
        const repaySOL = await program.methods
            .repay(new BN(1))
            .accounts({
                signer: signer.publicKey,
                mint: mintSol,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc({ commitment: "confirmed"});

            console.log("Repa SOL", repaySOL);
    });

    it("Test withdraw", async () => {
        const withdrawUSDC = await program.methods
            .withdraw(new BN(100))
            .accounts({
                signer: signer.publicKey,
                mint: mintUSDC,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc({ commitment: "confirmed" });

            console.log("Withdraw USDC", withdrawUSDC);
    });

});


