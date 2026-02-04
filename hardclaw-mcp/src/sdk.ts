import * as nacl from "tweetnacl";
import * as bip39 from "bip39";
import { derivePath } from "ed25519-hd-key";
import { createHash } from "crypto";

export interface Job {
  id: string;
  jobType: "deterministic" | "subjective";
  requester: string;
  description: string;
  input: string;
  bounty: number;
  burnFee: number;
  timeout: number;
  expectedHash?: string;
  timestamp: number;
}

export interface Solution {
  id: string;
  jobId: string;
  solver: string;
  output: string;
  timestamp: number;
  signature: string;
}

export interface VerificationResult {
  valid: boolean;
  reason?: string;
}

export class HardClawSDK {
  private keypair: nacl.SignKeyPair | null = null;
  private mnemonic: string | null = null;

  createWallet(): { address: string; mnemonic: string } {
    this.mnemonic = bip39.generateMnemonic(256); // 24 words
    const seed = bip39.mnemonicToSeedSync(this.mnemonic);
    const derivedSeed = derivePath("m/44'/501'/0'/0'", seed.toString("hex")).key;
    
    this.keypair = nacl.sign.keyPair.fromSeed(derivedSeed);
    const address = this.publicKeyToAddress(this.keypair.publicKey);

    return { address, mnemonic: this.mnemonic };
  }

  loadWallet(mnemonic: string): string {
    if (!bip39.validateMnemonic(mnemonic)) {
      throw new Error("Invalid mnemonic phrase");
    }

    this.mnemonic = mnemonic;
    const seed = bip39.mnemonicToSeedSync(mnemonic);
    const derivedSeed = derivePath("m/44'/501'/0'/0'", seed.toString("hex")).key;
    
    this.keypair = nacl.sign.keyPair.fromSeed(derivedSeed);
    return this.publicKeyToAddress(this.keypair.publicKey);
  }

  getAddress(): string {
    if (!this.keypair) {
      throw new Error("No wallet loaded. Use createWallet() or loadWallet() first.");
    }
    return this.publicKeyToAddress(this.keypair.publicKey);
  }

  createJob(params: {
    description: string;
    input: string;
    bounty: number;
    jobType: "deterministic" | "subjective";
    expectedHash?: string;
    timeout: number;
  }): Job {
    if (!this.keypair) {
      throw new Error("No wallet loaded");
    }

    const inputData = this.encodeInput(params.input);
    const expectedHash =
      params.expectedHash || this.hash(Buffer.from(inputData, "hex"));

    const job: Job = {
      id: this.hash(
        Buffer.concat([
          this.keypair.publicKey,
          Buffer.from(inputData, "hex"),
          Buffer.from(params.description),
          Buffer.from(Date.now().toString()),
        ])
      ),
      jobType: params.jobType,
      requester: this.publicKeyToAddress(this.keypair.publicKey),
      description: params.description,
      input: inputData,
      bounty: params.bounty,
      burnFee: 1,
      timeout: params.timeout,
      expectedHash: params.jobType === "deterministic" ? expectedHash : undefined,
      timestamp: Date.now(),
    };

    return job;
  }

  createSolution(jobId: string, output: string): Solution {
    if (!this.keypair) {
      throw new Error("No wallet loaded");
    }

    const outputData = this.encodeInput(output);
    const solutionData = Buffer.concat([
      Buffer.from(jobId, "hex"),
      this.keypair.publicKey,
      Buffer.from(outputData, "hex"),
      Buffer.from(Date.now().toString()),
    ]);

    const signature = nacl.sign.detached(solutionData, this.keypair.secretKey);

    const solution: Solution = {
      id: this.hash(solutionData),
      jobId,
      solver: this.publicKeyToAddress(this.keypair.publicKey),
      output: outputData,
      timestamp: Date.now(),
      signature: Buffer.from(signature).toString("hex"),
    };

    return solution;
  }

  verifySolution(jobId: string, solutionId: string): VerificationResult {
    // Simulated verification - in production this would:
    // 1. Fetch the job from network
    // 2. Fetch the solution from network
    // 3. Verify signature
    // 4. Check hash match (deterministic) or run Schelling voting (subjective)
    
    return {
      valid: true,
      reason: "Verification simulated - connect to hardclaw-node for real verification",
    };
  }

  private publicKeyToAddress(publicKey: Uint8Array): string {
    const hash = this.hash(Buffer.from(publicKey));
    return `hc1${hash.slice(0, 40)}`;
  }

  private hash(data: Buffer): string {
    return createHash("blake2s256").update(data).digest("hex");
  }

  private encodeInput(input: string): string {
    // If already hex, return as-is
    if (/^[0-9a-f]+$/i.test(input)) {
      return input.toLowerCase();
    }
    // Otherwise encode as hex
    return Buffer.from(input).toString("hex");
  }
}
