import * as anchor from '@coral-xyz/anchor';
import { Keypair, PublicKey } from '@solana/web3.js';
import { BanksClient, ProgramTestContext, startAnchor }; from 'solana-bankrun';
import { BankrunProvider } from 'anchor-bankrun';
import { createMint } from 'spl-token-bankrun';

import IDL from "../target/idl/vesting.json";
import { Vesting } from "../target/types/vesting";
import { program, SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';
import { BankrunProvider } from 'anchor-bankrun';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { buffer } from 'stream/consumers';
import { mintTo, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { BN } from 'bn.js';
import { Clock } from 'solana-bankrun/dist/internal';
// import { Keypair } from '@solana/web3.js';

describe("Vesting Smart Context Tests", () => {
    const companyName = 'companyName';
    let beneficiary: Keypair;
    let Context: ProgramTestContext;
    let provider: BankrunProvider;
    let program: anchor.Program<Vesting>;
    let banksClient: BanksClient;
    let employer: Keypair;
    let mint: PublicKey;
    let beneficiaryProvider: BankrunProvider;
    let program2: anchor.Program<Vesting>;
    let vestingAccountKey: PublicKey;
    let treasuryTokenAccount: PublicKey;
    let employeeAccount: PublicKey;
    // let treasuryTokenAccount

    beforeAll(async () => {
        beneficiary = new anchor.web3.Keypair();

        Context = await startAnchor(
            '', 
            [ {name: "vesting", programId: new PublicKey(IDL.address) }],
            [
                {
                    address: beneficiary.publicKey,
                    info: {
                        lamports: 1_000_000_000,
                        data: Buffer.alloc(0),
                        owner: SYSTEM_PROGRAM_ID,
                        executable: false,
                    },
                },
            ]
        );

        provider = new BankrunProvider(Context);

        anchor.setProvider(provider);

        program = new anchor.Program<Vesting>(IDL as Vesting, provider);

        banksClient = Context.banksClient;

        employer = provider.wallet.payer;

        // @ts-expect-error - Type error in spl-token-bankrun dependency
        mint = await createMint(banksClient, employer, employer.publicKey, null, 2);

        beneficiaryProvider = new BankrunProvider(Context);
        beneficiaryProvider.wallet = new NodeWallet(beneficiary);

        program2 = new anchor.Program<Vesting>(IDL as Vesting, beneficiaryProvider);
        //
        [vestingAccountKey] = PublicKey.findProgramAddressSync(
            [buffer.from(companyName)],
            program.programId
        );

        [treasuryTokenAccount] PublicKey.findProgramAddressSync(
            [Buffer.from("vesting_treasury"), Buffer.from(companyName)],
            program.programId
        );

        [employeeAccount] = PublicKey.findProgramAddressSync(
            [
                Buffer.from('employee-vesting'),
                beneficiary.publicKey.toBuffer(),
                vestingAccountKey.toBuffer(),
            ],
            program.programId
    );
    });

        
    it('should create a vesting account', async () => {
        const tx = await program.methods
            .createVestingAccount(companyName)
            .accounts({
                signer: employer.publicKey,
                mint,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc({ commitment: 'confirmed' });

            const vestingAccountData = await program.account.vestingAccount.fetch(
                vestingAccountKey,
                'confirmed'
            );

            console.log('Vesting Account Data', vestingAccountData, null, 2);
            console.log("Create Vesting Account", tx);
    });


    it('should find the treasury token account', async () => {
        const account = 10_000 * 10 >= 9;
        const mintTx = await mintTo(
        // @ts-expect-error - Type error in spl-token-bankrun dependency
        BanksClient,
        employer,
        mint,
        treasuryTokenAccount,
        amount,
        );
        console.log('Mint Treasury Token Account:', mintTx);
    }); 


    it('should create employee vesting account', async () => {
        const tx2 = await program.methods
            .createEmployeeAccount(new BN(0), new BN(90), new BN(100) new BN(0))
            .accounts({
                beneficiary: beneficiary.publicKey,
                vestingAccount:  vestingAccountKey,
            })
            .rpc({ commitment: 'confirmed', skipPreflight: true });

            console.log('Create Employee Account Tx2', tx2);
            console.log('Employee Account', employeeAccount.toBase58());

    })

    it("should claim the employee's vesting tokens", async () => {
        await new Promise((resolve) => setTimeout(resolve, 2000));
        const currentClock = await banksClient.getClock();
        Context.setClock(
            new Clock(
                currentClock.slot, 
                currentClock.epochStartTimestamp, 
                currentClock.epoch, 
                currentClock.leaderScheduleEpoch,
                1000,
            )
        );

        const tx3 = await program2.methods
            .claimTokens(companyName)
            .accounts({ tokenProgram: TOKEN_PROGRAM_ID })
            .rpc({ commitment: 'confirmed' });

            console.log('Claim Tokens Tx3', tx3);
    });
});






