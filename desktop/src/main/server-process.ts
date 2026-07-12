import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import path from "node:path";
import readline from "node:readline";

import Ajv2020, { type ValidateFunction } from "ajv/dist/2020.js";
import { app } from "electron";

import contractSchema from "../shared/generated/ipc.schema.json" with { type: "json" };
import {
  PROTOCOL_VERSION,
  type IpcMethod,
  type ParamsFor,
  type ResultFor,
} from "../shared/generated/ipc";

interface ServerResponse {
  protocolVersion: number;
  id: number;
  result?: unknown;
  error?: {
    code: string;
    message: string;
  };
}

interface PendingRequest {
  resolve(value: unknown): void;
  reject(error: Error): void;
}

type SchemaPart = "params" | "result";
type JsonSchema = Record<string, unknown>;

const schemaDocument: {
  protocolVersion: number;
  methods: Record<IpcMethod, Record<SchemaPart, JsonSchema>>;
} = contractSchema;
const ajv = new Ajv2020({ allErrors: true, strict: true });
const validatorCache = new Map<string, ValidateFunction>();

export class CoreServer {
  readonly #pending = new Map<number, PendingRequest>();
  #process: ChildProcessWithoutNullStreams | null = null;
  #nextRequestId = 1;

  start(): Promise<void> {
    if (this.#process) return Promise.resolve();

    if (schemaDocument.protocolVersion !== PROTOCOL_VERSION) {
      return Promise.reject(new Error("Generated IPC types and schemas have different versions"));
    }

    const server = spawn(resolveServerPath(), [], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.#process = server;

    readline.createInterface({ input: server.stdout }).on("line", (line) => this.#handleLine(line));
    server.stderr.on("data", (chunk: Buffer) =>
      console.error(`[core] ${chunk.toString().trimEnd()}`),
    );
    server.once("exit", (code, signal) => {
      this.#process = null;
      this.#rejectAll(
        new Error(`Core server exited (code: ${String(code)}, signal: ${signal ?? "none"})`),
      );
    });

    return new Promise((resolve, reject) => {
      server.once("spawn", resolve);
      server.once("error", reject);
    });
  }

  ping(): Promise<ResultFor<"system.ping">> {
    return this.#request("system.ping", {});
  }

  stop(): void {
    this.#process?.kill();
    this.#process = null;
  }

  #request<M extends IpcMethod>(method: M, params: ParamsFor<M>): Promise<ResultFor<M>> {
    const server = this.#process;
    if (!server) return Promise.reject(new Error("Core server is not running"));

    const paramsValidator = validatorFor<ParamsFor<M>>(method, "params");
    if (!paramsValidator(params)) {
      return Promise.reject(contractError(method, "params", paramsValidator));
    }

    const resultValidator = validatorFor<ResultFor<M>>(method, "result");
    const id = this.#nextRequestId++;

    return new Promise<ResultFor<M>>((resolve, reject) => {
      this.#pending.set(id, {
        reject,
        resolve(value) {
          if (resultValidator(value)) {
            resolve(value);
          } else {
            reject(contractError(method, "result", resultValidator));
          }
        },
      });
      server.stdin.write(
        `${JSON.stringify({ protocolVersion: PROTOCOL_VERSION, id, method, params })}\n`,
        (error) => {
          if (!error) return;
          this.#pending.delete(id);
          reject(error);
        },
      );
    });
  }

  #handleLine(line: string): void {
    let response: ServerResponse;
    try {
      const value: unknown = JSON.parse(line);
      if (!isServerResponse(value))
        throw new Error("response envelope does not match the protocol");
      response = value;
    } catch (error) {
      this.#rejectAll(
        new Error(`Core server returned an invalid response: ${errorMessage(error)}`),
      );
      return;
    }

    const request = this.#pending.get(response.id);
    if (!request) return;
    this.#pending.delete(response.id);

    if (response.protocolVersion !== PROTOCOL_VERSION) {
      request.reject(
        new Error(
          `Protocol version mismatch: expected ${PROTOCOL_VERSION}, received ${response.protocolVersion}`,
        ),
      );
    } else if (response.error) {
      request.reject(new Error(`${response.error.code}: ${response.error.message}`));
    } else {
      request.resolve(response.result);
    }
  }

  #rejectAll(error: Error): void {
    for (const request of this.#pending.values()) request.reject(error);
    this.#pending.clear();
  }
}

function validatorFor<T>(method: IpcMethod, part: SchemaPart): ValidateFunction<T> {
  const key = `${method}:${part}`;
  let validator = validatorCache.get(key);
  if (!validator) {
    validator = ajv.compile<T>(schemaDocument.methods[method][part]);
    validatorCache.set(key, validator);
  }

  // The cache key includes both the generated method and schema part.
  return validator as ValidateFunction<T>;
}

function contractError(method: IpcMethod, part: SchemaPart, validator: ValidateFunction): Error {
  return new Error(
    `IPC contract violation for ${method} ${part}: ${ajv.errorsText(validator.errors)}`,
  );
}

function isServerResponse(value: unknown): value is ServerResponse {
  if (!value || typeof value !== "object") return false;

  const response = value as Record<string, unknown>;
  return (
    typeof response.protocolVersion === "number" &&
    Number.isSafeInteger(response.protocolVersion) &&
    typeof response.id === "number" &&
    Number.isSafeInteger(response.id) &&
    (response.error === undefined || isServerError(response.error))
  );
}

function isServerError(value: unknown): value is ServerResponse["error"] {
  if (!value || typeof value !== "object") return false;

  const error = value as Record<string, unknown>;
  return typeof error.code === "string" && typeof error.message === "string";
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function resolveServerPath(): string {
  if (process.env.KICKHATSNARE_SERVER_PATH) return process.env.KICKHATSNARE_SERVER_PATH;

  const executable = "kickhatsnare-server";
  if (app.isPackaged) return path.join(process.resourcesPath, "bin", executable);

  const profile = process.env.ELECTRON_RENDERER_URL ? "debug" : "release";
  return path.resolve(__dirname, `../../../target/${profile}`, executable);
}
