#!/usr/bin/env node

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  Tool,
} from "@modelcontextprotocol/sdk/types.js";
import { HardClawSDK } from "./sdk.js";

const sdk = new HardClawSDK();

const server = new Server(
  {
    name: "hardclaw-mcp",
    version: "0.1.0",
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

const tools: Tool[] = [
  {
    name: "create_wallet",
    description: "Create a new HardClaw wallet with BIP39 mnemonic",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "load_wallet",
    description: "Load wallet from mnemonic phrase",
    inputSchema: {
      type: "object",
      properties: {
        mnemonic: {
          type: "string",
          description: "BIP39 mnemonic phrase (12 or 24 words)",
        },
      },
      required: ["mnemonic"],
    },
  },
  {
    name: "get_address",
    description: "Get the current wallet address",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "submit_job",
    description: "Submit a job to the HardClaw network with a bounty",
    inputSchema: {
      type: "object",
      properties: {
        description: {
          type: "string",
          description: "Description of the job",
        },
        input: {
          type: "string",
          description: "Input data (hex-encoded or plain text)",
        },
        bounty: {
          type: "number",
          description: "Bounty amount in HCLAW tokens",
        },
        jobType: {
          type: "string",
          enum: ["deterministic", "subjective"],
          description: "Type of job verification",
        },
        expectedHash: {
          type: "string",
          description: "Expected output hash for deterministic jobs (optional)",
        },
        timeout: {
          type: "number",
          description: "Job timeout in seconds (default: 3600)",
        },
      },
      required: ["description", "input", "bounty", "jobType"],
    },
  },
  {
    name: "submit_solution",
    description: "Submit a solution for a job",
    inputSchema: {
      type: "object",
      properties: {
        jobId: {
          type: "string",
          description: "Job ID (hex hash)",
        },
        output: {
          type: "string",
          description: "Solution output data (hex-encoded or plain text)",
        },
      },
      required: ["jobId", "output"],
    },
  },
  {
    name: "verify_solution",
    description: "Verify a solution against a job (verifier operation)",
    inputSchema: {
      type: "object",
      properties: {
        jobId: {
          type: "string",
          description: "Job ID (hex hash)",
        },
        solutionId: {
          type: "string",
          description: "Solution ID (hex hash)",
        },
      },
      required: ["jobId", "solutionId"],
    },
  },
  {
    name: "get_balance",
    description: "Get wallet balance (requires network connection)",
    inputSchema: {
      type: "object",
      properties: {},
      required: [],
    },
  },
  {
    name: "list_jobs",
    description: "List available jobs from the network",
    inputSchema: {
      type: "object",
      properties: {
        limit: {
          type: "number",
          description: "Maximum number of jobs to return (default: 10)",
        },
      },
      required: [],
    },
  },
];

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools,
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "create_wallet": {
        const result = sdk.createWallet();
        return {
          content: [
            {
              type: "text",
              text: `Created new wallet\nAddress: ${result.address}\nMnemonic: ${result.mnemonic}\n\n⚠️  Save this mnemonic securely!`,
            },
          ],
        };
      }

      case "load_wallet": {
        const { mnemonic } = args as { mnemonic: string };
        const address = sdk.loadWallet(mnemonic);
        return {
          content: [
            {
              type: "text",
              text: `Loaded wallet\nAddress: ${address}`,
            },
          ],
        };
      }

      case "get_address": {
        const address = sdk.getAddress();
        return {
          content: [
            {
              type: "text",
              text: address,
            },
          ],
        };
      }

      case "submit_job": {
        const { description, input, bounty, jobType, expectedHash, timeout } =
          args as {
            description: string;
            input: string;
            bounty: number;
            jobType: "deterministic" | "subjective";
            expectedHash?: string;
            timeout?: number;
          };

        const job = sdk.createJob({
          description,
          input,
          bounty,
          jobType,
          expectedHash,
          timeout: timeout || 3600,
        });

        return {
          content: [
            {
              type: "text",
              text: `Job created:\n  ID: ${job.id}\n  Type: ${job.jobType}\n  Bounty: ${bounty} HCLAW\n  Burn: 1 HCLAW\n  Timeout: ${job.timeout}s\n\n📡 Broadcast this to the network using hardclaw-node`,
            },
          ],
        };
      }

      case "submit_solution": {
        const { jobId, output } = args as { jobId: string; output: string };
        const solution = sdk.createSolution(jobId, output);

        return {
          content: [
            {
              type: "text",
              text: `Solution created:\n  Job ID: ${jobId}\n  Solution ID: ${solution.id}\n  Solver: ${solution.solver}\n\n📡 Broadcast this to the network using hardclaw-node`,
            },
          ],
        };
      }

      case "verify_solution": {
        const { jobId, solutionId } = args as {
          jobId: string;
          solutionId: string;
        };
        const result = sdk.verifySolution(jobId, solutionId);

        return {
          content: [
            {
              type: "text",
              text: `Verification result:\n  Valid: ${result.valid}\n  Reason: ${result.reason || "N/A"}\n\n${result.valid ? "✅ Solution passes verification" : "❌ Solution failed verification"}`,
            },
          ],
        };
      }

      case "get_balance": {
        const address = sdk.getAddress();
        return {
          content: [
            {
              type: "text",
              text: `Balance for ${address}: 0.0 HCLAW\n\n⚠️  Balance lookup requires connection to hardclaw-node`,
            },
          ],
        };
      }

      case "list_jobs": {
        const { limit = 10 } = args as { limit?: number };
        return {
          content: [
            {
              type: "text",
              text: `Fetching ${limit} jobs from network...\n\n⚠️  Job listing requires connection to hardclaw-node`,
            },
          ],
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return {
      content: [
        {
          type: "text",
          text: `Error: ${message}`,
        },
      ],
      isError: true,
    };
  }
});

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("HardClaw MCP server running on stdio");
}

main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
