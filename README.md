# xixi-zk: Proof of Concept for Mota Game Zero Knowledge Proof

A prototype demonstrating zero-knowledge proof verification for the mota game [希希传说](https://h5mota.com/games/xxchuanshuo0/) using RISC0 zkVM.

## Core Capabilities

This prototype demonstrates how zero-knowledge proofs can be applied to mota game verification:

1. **Game Simulation & Validation**
   - Simulates the mota game mechanics with rust
   - Validates solution JSON object against game rules

2. **Proof Generation**
   - Creates cryptographic proofs of valid solutions
   - Uses RISC0 zkVM to generate STARK proofs

3. **Proof Verification**
   - Allows independent verification without solution disclosure
   - Confirms solution validity and final score

## Technology Stack

- RISC0 zkVM v2.1+ (proof generation)
- Rust 1.88 (implementation language)

## Project Structure

```
xixi-zk/
├── CAVEAT.md - Important notes
├── example/ - Sample game solutions
├── json-generator/ - Scripts that generates stages (example: xixi)
├── motadata-visualizer/ - Static website to visualize xixi game config json
├── route-converter/ - Scripts that generates route file on xixi
└── xixi-zk/ - Core ZKP implementation in rust, standard risc0 project
    ├── core/ - Shared game data structure
    ├── host/ - Prover/verifier host code  
    └── methods/ - zkVM guest programs
```

## Getting Started

### Prerequisites

- Linux
- [docker](https://docs.docker.com/engine/install/)
- [Rust](https://www.rust-lang.org/tools/install) external to docker; docker-in-docker might work as well
- [RISC0](https://dev.risczero.com/api/zkvm/quickstart) external to docker

rust install & risc0 install can be probably summarized in:

```bash
mkdir rust-env-tmp && cd rust-env-tmp
wget "https://static.rust-lang.org/rustup/archive/1.28.2/x86_64-unknown-linux-gnu/rustup-init"
chmod +x rustup-init
./rustup-init

curl -L https://risczero.com/install -o rzup-install
bash rzup-install
rzup install
```

### Building rust parts

```bash
mkdir -p build
cd xixi-zk
cargo build --release
# test
target/release/host
# copy
cp target/release/host ../build/
```

### Generate game stage file as rkyv

```bash
(cd json-generator && python3 convert_motadata.py bigdata/motadata.py --json-output ../build/xixi.json)
# build/xixi.compact.json is what to be used
build/host convert build/xixi.compact.json build/xixi.rkyv
```

Note: our convert_motadata.py set boss hp to 1. Therefore the known best record would have 180666 in score.

### Generate your own route

You can use given `example/route1.txt` for testing, or use your own route.

If you have a `h5route` or `h5save`, you can turn to `route-converter` to get one of your route file `route.txt`.

NOTE: `route.txt` generated from your route contains full info about your mota route, keep it secret to other people. Later we will generate its ZKP (`proof.bin`) that can be shared to forum.

### Run solution but not prove

```bash
echo 'user3266' > build/user
RUST_LOG=info RISC0_DEV_MODE=1 RISC0_INFO=1 build/host prove build/xixi.rkyv build/user example/route1.txt build/t1.bin
```

You will see things like
```shell
WARNING: proving in dev mode. This will not generate valid, secure proofs.
2025-07-19T13:53:05.708838Z  INFO risc0_zkvm::host::server::exec::executor: execution time: 24.630149ms
2025-07-19T13:53:05.708863Z  INFO risc0_zkvm::host::server::session: number of segments: 1
2025-07-19T13:53:05.708866Z  INFO risc0_zkvm::host::server::session: 262144 total cycles
2025-07-19T13:53:05.708869Z  INFO risc0_zkvm::host::server::session: 182445 user cycles (69.60%)
2025-07-19T13:53:05.708873Z  INFO risc0_zkvm::host::server::session: 48879 paging cycles (18.65%)
2025-07-19T13:53:05.708876Z  INFO risc0_zkvm::host::server::session: 30820 reserved cycles (11.76%)
2025-07-19T13:53:05.708879Z  INFO risc0_zkvm::host::server::session: ecalls
2025-07-19T13:53:05.708889Z  INFO risc0_zkvm::host::server::session: 	71 Sha2 calls, 42790 cycles, (16.32%)
2025-07-19T13:53:05.708893Z  INFO risc0_zkvm::host::server::session: 	226 Read calls, 3174 cycles, (1.21%)
2025-07-19T13:53:05.708896Z  INFO risc0_zkvm::host::server::session: 	1 Terminate calls, 2 cycles, (0.00%)
2025-07-19T13:53:05.708899Z  INFO risc0_zkvm::host::server::session: 	0 Write calls, 0 cycles, (0.00%)
2025-07-19T13:53:05.708902Z  INFO risc0_zkvm::host::server::session: 	0 User calls, 0 cycles, (0.00%)
2025-07-19T13:53:05.708905Z  INFO risc0_zkvm::host::server::session: 	0 Poseidon2 calls, 0 cycles, (0.00%)
2025-07-19T13:53:05.708908Z  INFO risc0_zkvm::host::server::session: 	0 BigInt calls, 0 cycles, (0.00%)
2025-07-19T13:53:05.708911Z  INFO risc0_zkvm::host::server::session: syscalls
2025-07-19T13:53:05.708914Z  INFO risc0_zkvm::host::server::session: 	67 Write calls
2025-07-19T13:53:05.708917Z  INFO risc0_zkvm::host::server::session: 	45 Read calls
2025-07-19T13:53:05.708919Z  INFO risc0_zkvm::host::server::session: 	0 VerifyIntegrity2 calls
2025-07-19T13:53:05.708922Z  INFO risc0_zkvm::host::server::session: 	0 VerifyIntegrity calls
2025-07-19T13:53:05.708924Z  INFO risc0_zkvm::host::server::session: 	0 ProveKeccak calls
2025-07-19T13:53:05.708927Z  INFO risc0_zkvm::host::server::session: 	0 Keccak calls
WARNING: Proving in dev mode does not generate a valid receipt. Receipts generated from this process are invalid and should never be used in production.
Guest output:
{
  "config_hash": "5cc8681fb14dd7a453b3cc0d673310cdc22f3039c5e87f1b70998448d75aaa30",
  "scores": [
    46595
  ],
  "user_cred_hash": "8934c726e11a90e6ef0a06548262ba1f16b7b1322ac9112256ffcc87ec41bc64"
}
Proof written to: build/s1.bin
```

#### Explanations (as for my understanding)

`Guest` - The program to be proven to be run. In our case it is simulator & verifier of mota route.

`262144 total cycles` - the proof consist of these cpu cycles. A segment proof always contains number of cpu cycles that is power of 2.

`182445 user cycles`, `48879 paging cycles` - our program uses 182445 cpu cycles, whereas internal page mapping uses 48879 cycles.

`71 Sha2 calls, 42790 cycles, (16.32%)` - our program uses 42790 cycles to generate hash of game stage data (rkyv)

`WARNING: Proving in dev mode does not generate a valid receipt.` - `RISC0_DEV_MODE=1` skips proof generation and only runs input solution file.

`config_hash` - in this project, game stage data are named `GameConfig` and is input as `rkyv` so that it can be efficiently accessed without copying cost. The data are passed as a big buffer and its hash are part of output of guest.

`scores` - basically mota scores when you complete the game without dying. An array, but currently with only one element (HP)

`user_cred_hash` - hash of `"user3266\n"`.

### Prove and verify

#### Prove

```bash
time RUST_LOG=info RISC0_INFO=1 build/host prove build/xixi.rkyv build/user example/route1.txt build/s1.bin
```

A output could be (skipping identical logs):

```shell
Guest output:
{
  "config_hash": "5cc8681fb14dd7a453b3cc0d673310cdc22f3039c5e87f1b70998448d75aaa30",
  "scores": [
    46595
  ],
  "user_cred_hash": "8934c726e11a90e6ef0a06548262ba1f16b7b1322ac9112256ffcc87ec41bc64"
}
Proof written to: build/s1.bin

real	6m12.088s
user	23m4.970s
sys	0m4.680s
```

Proving is 

#### Verify

```bash
build/host verify build/s1.bin
```

The output could be:

```json
{"game":"5cc8681fb14dd7a453b3cc0d673310cdc22f3039c5e87f1b70998448d75aaa30","scores":[46595],"status":"verified","usercred":"user3266\n"}
```

Above shows that `build/s1.bin` is a valid proof that user claiming to be `user3266` had played the game "5cc8681fb14dd7a453b3cc0d673310cdc22f3039c5e87f1b70998448d75aaa30" (xixi) and scored 46595.

The other file is not a valid proof:

```bash
build/host verify build/t1.bin 
```

```shell
Error: verification indicates proof is invalid
```

# Reproducibility test

(Update : moved `core` folder inside `guest` so that file difference in building environment is minimize)

```
$ r0vm --id --elf xixi_verifier.bin
5be9bd21c4e75c2b4ccef0e7b6308f5ad5352940b3ccd83aaf22a2a229f67b61
```

# Technical Discussion

## Zero-Knowledge Proof Fundamentals

Zero-knowledge proofs allow one party (the prover) to convince another party (the verifier) that a statement is true without revealing any information beyond the validity of the statement itself.

**Key Properties:**
- Completeness: If the statement is true, an honest verifier will be convinced
- Soundness: If the statement is false, no cheating prover can convince the verifier
- Zero-knowledge: The verifier learns nothing beyond the validity of the statement

## RISC Zero Execution Proof Mechanism

RISC Zero's zkVM provides a verifiable computation environment:

1. **Deterministic Execution**
   - All executions are recorded
   - Uses RISC-V ISA for standardized behavior
   - Each guest is a ELF executable

2. **Proof Components**
   - **Journal**: Contains public outputs
   - **Receipt**: Cryptographic proof of execution
   - **Image ID**: Cryptographic identifier of the zkVM program binary
   - **Seal**: STARK proof of correct execution

3. **I/O Handling**
   - stdin: Private inputs to the program
   - stdout: Private outputs (not used in our project)
   - stderr: Debug output (not part of proof)
   - journal: Public outputs (included in proof)

4. **Verification Process**
   - Verifier checks:
     * Image ID matches expected program
     * Seal validates execution trace
     * Journal contains authentic outputs

## Game-Specific Implementation

This prototype implements zero-knowledge proofs for mota game solutions using RISC Zero zkVM:

1. **Game Setup**
   - Game configuration is serialized using rkyv format
   - SHA-256 hash of configuration is included in proof
   - This ensures the proof corresponds to a specific game setup

2. **Proof Generation**
   - Game simulation runs in isolated RISC-V zkVM
   - All operations are recorded in execution trace
   - STARK proof is generated from this trace
   - Proof includes:
     * Game configuration hash
     * User identifier hash
     * Final score

3. **Verification**
   - Verifier checks cryptographic integrity
   - Confirms untampered execution

## Zero-Knowledge Proof Implementation

**What the Proof Shows**
- The game was played with the specified configuration
- A specific user completed the game
- A valid solution exists (without revealing it)
- The final score is accurate

**Private Information (Not Revealed)**
- The actual game solution (sequence of moves)
- Intermediate game states during play

**Technical Components**
1. Game Data:
   - Serialized using rkyv format for efficiency
   - SHA-256 hash identifies the game version

2. Proof Generation:
   - Game simulation runs in RISC Zero zkVM
   - Records cryptographic evidence of computation
   - Output proof contains:
     * Game configuration hash
     * User identifier hash
     * Final score

3. Verification:
   - Completes in under 1 second
   - Confirms proof validity
   - Reveals only: "User X completed game Y with score Z"

## Security Considerations

The system's security relies on:

1. **Cryptographic Foundations**
   - SHA-256 for data integrity
   - RISC Zero's STARK proof system
   - FRI protocol for polynomial commitments

2. **Implementation Factors**
   - Correct game simulation implementation
   - RISC Zero implementation security

3. **Security Status**
   - RISC Zero is actively maintained with security advisories
   - The platform is still evolving with occasional updates
   - Full stability will depend on RISC Zero reaching maturity

**Performance Metrics (Measured on i7-12700K)**

| Metric | Value | Notes |
|--------|-------|-------|
| Proof Generation | ~23 minutes | For 262,144 cycles, single CPU core |
| Verification | <1 second | Minimal computational overhead |
| Proof Size | 217.45 KB | Exact size of a STARK proof |

Recent improvements in RISC Zero (2.1.0+ in July 2025) have optimized the process through:
- Reduced proof generation time and memory requirements
- Improved API interfaces

## Common Questions

**"How do I know the game data is correct?"**
The game configuration is extracted directly from the original game files. The SHA-256 hash serves as a fingerprint - any change to the game data would produce a different hash, making tampering detectable.

**"How does the system prevent proof forgery?"**

Security is enforced through multiple layers:

1. **Cryptographic Guarantees**
   - SHA-256 for data integrity
   - STARK proofs for execution validity
   - FRI protocol soundness

2. **System Design**
   - Deterministic builds ensure reproducibility
   - Isolated execution in zkVM
   - Game simulation with comprehensive validation

**"What if someone modifies the game simulation executable?"**
RISC Zero embeds each proof with an image ID derived from the program binary. Any modification to the code would produce a different image ID. RISC Zero uses deterministic builds to ensure the same source code always produces identical binaries, allowing verification that the proof was generated by the expected program.

**"Is the JSON representation exactly identical to the original game?"**
Demonstration of this aspect is still under development. However you can be convinced partially by finding that (1) your route file is correctly recognized by our zero-knowledge proof system, and (2) each route uses (nearly) all info of game config data.

Nonetheless backdoors are still possible logically (Example: add an edge so a late enemy can be beaten much earlier). Therefore another evidence is at `motadata-visualizer/` which everyone can audit whether there are difference between original game and the abstracted graph form.

In the future maybe a reproducible graphic form generator script may be provided, then this question may not exist any more.

**"How can I convert a h5route into such a proof?"**
See `route-converter`. H5mota user should find it easy to generate a `route.txt` using the given js plugin.


## License

This project is licensed under Apache 2.0, the same license used by RISC Zero.
