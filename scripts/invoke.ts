import * as anchor from "@anchor-lang/core";
import { Program, BN } from "@anchor-lang/core";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
	TOKEN_PROGRAM_ID,
	ASSOCIATED_TOKEN_PROGRAM_ID,
	getAssociatedTokenAddressSync,
	getAccount,
	getMint,
} from "@solana/spl-token";
import fs from "fs";
import path from "path";

const UNIT = 1_000_000; // 6 decimals
const MAX_SUPPLY = 20 * UNIT;

const explorer = (sig: string) =>
	`https://explorer.solana.com/tx/${sig}?cluster=devnet`;

async function main() {
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);

	const idl = JSON.parse(
		fs.readFileSync(path.resolve("target/idl/assignment_1.json"), "utf8")
	);
	const program = new Program(idl, provider);
	const wallet = provider.wallet.publicKey;
	console.log("Program:", program.programId.toBase58());
	console.log("Wallet :", wallet.toBase58());

	const [config] = PublicKey.findProgramAddressSync(
		[Buffer.from("config")],
		program.programId
	);

	// ---------- 1. initialize (only once per deployment) ----------
	let mint: PublicKey;
	const existing = await provider.connection.getAccountInfo(config);
	if (!existing) {
		const mintKp = Keypair.generate();
		const sig = await program.methods
			.initialize(new BN(MAX_SUPPLY))
			.accountsPartial({
				payer: wallet,
				config,
				mint: mintKp.publicKey,
				tokenProgram: TOKEN_PROGRAM_ID,
				systemProgram: SystemProgram.programId,
			})
			.signers([mintKp])
			.rpc();
		mint = mintKp.publicKey;
		console.log("\n[initialize] mint:", mint.toBase58());
		console.log(explorer(sig));
	} else {
		const cfg: any = await (program.account as any).tokenConfig.fetch(config);
		mint = cfg.mint;
		console.log("\n[initialize] already done, mint:", mint.toBase58());
	}

	// ---------- 2. mint_tokens ----------
	const myAta = getAssociatedTokenAddressSync(mint, wallet);
	const mintAccounts = {
		user: wallet,
		config,
		mint,
		userTokenAccount: myAta,
		tokenProgram: TOKEN_PROGRAM_ID,
		associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
		systemProgram: SystemProgram.programId,
	};

	try {
		const sig = await program.methods
			.mintTokens(new BN(5 * UNIT))
			.accountsPartial(mintAccounts)
			.rpc();
		console.log("\n[mint_tokens] minted 5 tokens");
		console.log(explorer(sig));
	} catch (e: any) {
		console.log("\n[mint_tokens] 5 tokens rejected (cap already reached):", e.message);
	}

	// ---------- 3. mint over the cap (should fail) ----------
	try {
		await program.methods
			.mintTokens(new BN(100 * UNIT))
			.accountsPartial(mintAccounts)
			.rpc();
		console.log("UNEXPECTED: over-cap mint succeeded");
	} catch (e: any) {
		console.log("\n[mint_tokens] 100 tokens rejected, as expected");
	}

	// ---------- 4. transfer_token ----------
	const recipient = Keypair.generate(); // fresh wallet, has no token account yet
	const recipientAta = getAssociatedTokenAddressSync(mint, recipient.publicKey);
	const sig = await program.methods
		.transferToken(new BN(2 * UNIT))
		.accountsPartial({
			sender: wallet,
			recipient: recipient.publicKey,
			mint,
			senderTokenAccount: myAta,
			recipientTokenAccount: recipientAta,
			tokenProgram: TOKEN_PROGRAM_ID,
			associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			systemProgram: SystemProgram.programId,
		})
		.rpc();
	console.log("\n[transfer_token] sent 2 tokens to", recipient.publicKey.toBase58());
	console.log(explorer(sig));

	// ---------- results ----------
	const conn = provider.connection;
	const mine = await getAccount(conn, myAta);
	const theirs = await getAccount(conn, recipientAta);
	const mintInfo = await getMint(conn, mint);
	console.log("\n--- Balances (tokens) ---");
	console.log("Your wallet :", Number(mine.amount) / UNIT);
	console.log("Recipient   :", Number(theirs.amount) / UNIT);
	console.log("Total supply:", Number(mintInfo.supply) / UNIT, "/ 20");
}

main().catch((e) => {
	console.error(e);
	process.exit(1);
});