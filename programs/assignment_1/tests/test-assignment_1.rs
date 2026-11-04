use anchor_lang::{prelude::Pubkey, system_program, InstructionData, ToAccountMetas};
use anchor_spl::{associated_token, token};
use litesvm::LiteSVM;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;

const UNIT: u64 = 1_000_000; // 6 decimals
const MAX_SUPPLY: u64 = 20 * UNIT;

// ---------- helpers ----------

struct Env {
    svm: LiteSVM,
    mint: Pubkey,
    config: Pubkey,
}

fn ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[wallet.as_ref(), token::ID.as_ref(), mint.as_ref()],
        &associated_token::ID,
    )
    .0
}

fn send(
    svm: &mut LiteSVM,
    ix: Instruction,
    payer: &Keypair,
    extra: &[&Keypair],
) -> Result<(), String> {
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend(extra);
    let tx = Transaction::new(
        &signers,
        Message::new(&[ix], Some(&payer.pubkey())),
        svm.latest_blockhash(),
    );
    let res = svm
        .send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e));
    svm.expire_blockhash(); // avoids "already processed" on repeated identical txs
    res
}

fn new_user(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    svm.airdrop(&kp.pubkey(), 5_000_000_000).unwrap();
    kp
}

// token account layout: mint(32) owner(32) amount(u64) ...
fn balance(svm: &LiteSVM, wallet: &Pubkey, mint: &Pubkey) -> u64 {
    match svm.get_account(&ata(wallet, mint)) {
        Some(acc) => u64::from_le_bytes(acc.data[64..72].try_into().unwrap()),
        None => 0,
    }
}

// mint layout: mint_authority COption<Pubkey>(36) supply(u64) ...
fn supply(svm: &LiteSVM, mint: &Pubkey) -> u64 {
    let acc = svm.get_account(mint).unwrap();
    u64::from_le_bytes(acc.data[36..44].try_into().unwrap())
}

fn init_ix(payer: &Pubkey, config: Pubkey, mint: Pubkey, max_supply: u64) -> Instruction {
    let accounts = assignment_1::accounts::Initialize {
        payer: *payer,
        config,
        mint,
        token_program: token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(
        assignment_1::ID,
        &assignment_1::instruction::Initialize { max_supply }.data(),
        accounts,
    )
}

fn mint_ix(user: &Pubkey, config: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let accounts = assignment_1::accounts::MintToken {
        user: *user,
        config,
        mint,
        user_token_account: ata(user, &mint),
        token_program: token::ID,
        associated_token_program: associated_token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(
        assignment_1::ID,
        &assignment_1::instruction::MintTokens { amount }.data(),
        accounts,
    )
}

fn transfer_ix(sender: &Pubkey, recipient: &Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let accounts = assignment_1::accounts::TransferToken {
        sender: *sender,
        recipient: *recipient,
        mint,
        sender_token_account: ata(sender, &mint),
        recipient_token_account: ata(recipient, &mint),
        token_program: token::ID,
        associated_token_program: associated_token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    Instruction::new_with_bytes(
        assignment_1::ID,
        &assignment_1::instruction::TransferToken { amount }.data(),
        accounts,
    )
}

fn setup() -> Env {
    let mut svm = LiteSVM::new(); // SPL Token + ATA programs are preloaded
    svm.add_program_from_file(assignment_1::ID, "../../target/deploy/assignment_1.so")
        .unwrap();

    let payer = new_user(&mut svm);
    let mint_kp = Keypair::new();
    let (config, _) = Pubkey::find_program_address(&[b"config"], &assignment_1::ID);

    let ix = init_ix(&payer.pubkey(), config, mint_kp.pubkey(), MAX_SUPPLY);
    send(&mut svm, ix, &payer, &[&mint_kp]).expect("initialize failed");

    Env {
        svm,
        mint: mint_kp.pubkey(),
        config,
    }
}

// ---------- tests ----------

#[test]
fn initialize_creates_mint() {
    let env = setup();
    assert!(env.svm.get_account(&env.config).is_some());
    assert_eq!(supply(&env.svm, &env.mint), 0);
}

#[test]
fn anyone_can_mint_within_cap() {
    let mut env = setup();
    let alice = new_user(&mut env.svm);

    let ix = mint_ix(&alice.pubkey(), env.config, env.mint, 10 * UNIT);
    send(&mut env.svm, ix, &alice, &[]).expect("mint failed");

    assert_eq!(balance(&env.svm, &alice.pubkey(), &env.mint), 10 * UNIT);
    assert_eq!(supply(&env.svm, &env.mint), 10 * UNIT);
}

#[test]
fn mint_over_cap_fails() {
    let mut env = setup();
    let alice = new_user(&mut env.svm);
    let bob = new_user(&mut env.svm);

    let ix = mint_ix(&alice.pubkey(), env.config, env.mint, 15 * UNIT);
    send(&mut env.svm, ix, &alice, &[]).unwrap();

    // 15 + 10 > 20, and the cap is global across users
    let ix = mint_ix(&bob.pubkey(), env.config, env.mint, 10 * UNIT);
    assert!(send(&mut env.svm, ix, &bob, &[]).is_err());
    assert_eq!(supply(&env.svm, &env.mint), 15 * UNIT);

    // exactly reaching the cap is fine
    let ix = mint_ix(&bob.pubkey(), env.config, env.mint, 5 * UNIT);
    send(&mut env.svm, ix, &bob, &[]).expect("minting up to the cap should work");
    assert_eq!(supply(&env.svm, &env.mint), MAX_SUPPLY);
}

#[test]
fn transfer_moves_tokens_and_creates_recipient_ata() {
    let mut env = setup();
    let alice = new_user(&mut env.svm);
    let bob = Keypair::new(); // never held the token, no ATA yet

    let ix = mint_ix(&alice.pubkey(), env.config, env.mint, 10 * UNIT);
    send(&mut env.svm, ix, &alice, &[]).unwrap();

    let ix = transfer_ix(&alice.pubkey(), &bob.pubkey(), env.mint, 4 * UNIT);
    send(&mut env.svm, ix, &alice, &[]).expect("transfer failed");

    assert_eq!(balance(&env.svm, &alice.pubkey(), &env.mint), 6 * UNIT);
    assert_eq!(balance(&env.svm, &bob.pubkey(), &env.mint), 4 * UNIT);
    assert_eq!(supply(&env.svm, &env.mint), 10 * UNIT); // supply unchanged
}

#[test]
fn transfer_more_than_balance_fails() {
    let mut env = setup();
    let alice = new_user(&mut env.svm);
    let bob = Keypair::new();

    let ix = mint_ix(&alice.pubkey(), env.config, env.mint, 3 * UNIT);
    send(&mut env.svm, ix, &alice, &[]).unwrap();

    let ix = transfer_ix(&alice.pubkey(), &bob.pubkey(), env.mint, 5 * UNIT);
    assert!(send(&mut env.svm, ix, &alice, &[]).is_err());
    assert_eq!(balance(&env.svm, &alice.pubkey(), &env.mint), 3 * UNIT);
}
