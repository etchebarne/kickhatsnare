import { chmod, copyFile, mkdir } from "node:fs/promises";
import path from "node:path";

const executable = "kickhatsnare-server";
const source = path.resolve(import.meta.dirname, "../../target/release", executable);
const outputDirectory = path.resolve(import.meta.dirname, "../build/bin");
const destination = path.join(outputDirectory, executable);

await mkdir(outputDirectory, { recursive: true });
await copyFile(source, destination);
await chmod(destination, 0o755);

console.log(`Staged Rust server at ${destination}`);
