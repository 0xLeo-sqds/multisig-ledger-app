import Transport from "@ledgerhq/hw-transport";
import { CLA, P2_EXTEND, P2_MORE, MAX_CHUNK_SIZE } from "../constants";
import { statusWordToError } from "../errors";

/**
 * Send a chunked APDU command, splitting the payload across multiple frames.
 *
 * Chunking protocol (compatible with app-solana):
 * - First chunk: P2 = P2_MORE (if more chunks follow) or 0x00 (if single chunk)
 * - Continuation chunks: P2 = P2_EXTEND | P2_MORE (if more follow) or P2_EXTEND (if last)
 *
 * @returns The response data from the final chunk (excluding the 2-byte status word).
 */
export async function sendChunked(
  transport: Transport,
  ins: number,
  p1: number,
  payload: Buffer
): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for (let offset = 0; offset < payload.length; offset += MAX_CHUNK_SIZE) {
    chunks.push(payload.subarray(offset, offset + MAX_CHUNK_SIZE));
  }

  if (chunks.length === 0) {
    chunks.push(Buffer.alloc(0));
  }

  let response: Buffer = Buffer.alloc(0);

  for (let i = 0; i < chunks.length; i++) {
    const isFirst = i === 0;
    const isLast = i === chunks.length - 1;

    let p2 = 0x00;
    if (!isFirst) p2 |= P2_EXTEND;
    if (!isLast) p2 |= P2_MORE;

    response = await transport.send(CLA, ins, p1, p2, chunks[i], [0x9000]);
  }

  return response;
}
