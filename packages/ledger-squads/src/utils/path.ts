import { HARDENED } from "../constants";

/**
 * Serialize a BIP32 derivation path to bytes for the APDU payload.
 *
 * Format: [num_components(1)] [component_u32_be(4)]...
 *
 * @param path - String path like "44'/501'/0'" or array of u32 components
 */
export function serializePath(path: string | number[]): Buffer {
  const components =
    typeof path === "string" ? parsePath(path) : path;

  if (components.length < 3 || components.length > 4) {
    throw new Error(
      `Invalid BIP32 path: expected 3 or 4 components, got ${components.length}`
    );
  }

  const buf = Buffer.alloc(1 + components.length * 4);
  buf[0] = components.length;

  for (let i = 0; i < components.length; i++) {
    buf.writeUInt32BE(components[i], 1 + i * 4);
  }

  return buf;
}

function parsePath(path: string): number[] {
  const parts = path
    .replace(/^m\//, "")
    .split("/")
    .map((part) => {
      const hardened = part.endsWith("'") || part.endsWith("h");
      const index = parseInt(hardened ? part.slice(0, -1) : part, 10);
      if (isNaN(index)) {
        throw new Error(`Invalid path component: ${part}`);
      }
      return hardened ? index | HARDENED : index;
    });
  return parts;
}
