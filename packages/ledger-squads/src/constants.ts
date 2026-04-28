/**
 * APDU constants for the Squads Ledger app.
 * Compatible with the Solana Ledger app protocol.
 */

/** Application class byte — matches Solana convention. */
export const CLA = 0xe0;

/** Instruction codes. */
export const INS_GET_VERSION = 0x03;
export const INS_GET_APP_CONFIGURATION = 0x04;
export const INS_GET_PUBKEY = 0x05;
export const INS_SIGN_TRANSACTION = 0x06;

/** P1 values. */
export const P1_NON_CONFIRM = 0x00;
export const P1_CONFIRM = 0x01;

/** P2 bit flags for chunking. */
export const P2_EXTEND = 0x01;
export const P2_MORE = 0x02;

/** Maximum APDU payload per chunk. */
export const MAX_CHUNK_SIZE = 255;

/** BIP32 hardened flag. */
export const HARDENED = 0x80000000;
