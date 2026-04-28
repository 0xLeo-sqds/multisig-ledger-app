import Transport from "@ledgerhq/hw-transport";
import {
  CLA,
  INS_GET_APP_CONFIGURATION,
  INS_GET_PUBKEY,
  INS_GET_VERSION,
  INS_SIGN_TRANSACTION,
  P1_CONFIRM,
  P1_NON_CONFIRM,
} from "./constants";
import { sendChunked } from "./utils/apdu";
import { serializePath } from "./utils/path";
import {
  WrongAppError,
  LedgerSquadsError,
  statusWordToError,
} from "./errors";

export interface AppConfig {
  /** Whether blind signing is enabled in app settings. */
  blindSigningEnabled: boolean;
  /** App version string (e.g., "0.1.0"). */
  version: string;
}

export interface SignResult {
  /** 64-byte Ed25519 signature. */
  signature: Buffer;
}

/**
 * SDK for communicating with the Squads Ledger app.
 *
 * Accepts any Ledger Transport (WebHID, Node HID, Speculos, BLE).
 * Does NOT use the DMK — uses the stable legacy transport API.
 *
 * @example
 * ```ts
 * import TransportWebHID from "@ledgerhq/hw-transport-webhid";
 * import { LedgerSquads } from "@aspect-finance/ledger-squads";
 *
 * const transport = await TransportWebHID.create();
 * const squads = new LedgerSquads(transport);
 * const config = await squads.getAppConfiguration();
 * const { signature } = await squads.signTransaction("44'/501'/0'", txBuffer);
 * ```
 */
export class LedgerSquads {
  constructor(private transport: Transport) {}

  /**
   * Get the app configuration (version + blind signing status).
   * Also serves as app detection — throws WrongAppError if the wrong app is open.
   */
  async getAppConfiguration(): Promise<AppConfig> {
    try {
      const response = await this.transport.send(
        CLA,
        INS_GET_APP_CONFIGURATION,
        0x00,
        0x00,
        Buffer.alloc(0),
        [0x9000]
      );

      // Response: [blind_signing(1)] [reserved(1)] [major(1)] [minor(1)] [patch(1)]
      const blindSigningEnabled = response[0] === 1;
      const version = `${response[2]}.${response[3]}.${response[4]}`;
      return { blindSigningEnabled, version };
    } catch (e: any) {
      if (e.statusCode === 0x6e00 || e.statusCode === 0x6d00) {
        throw new WrongAppError();
      }
      throw e;
    }
  }

  /**
   * Verify the correct app is open and meets minimum version requirements.
   *
   * @param minVersion - Minimum required version (e.g., "0.1.0")
   */
  async ensureCorrectApp(minVersion = "0.1.0"): Promise<AppConfig> {
    const config = await this.getAppConfiguration();
    const [reqMajor, reqMinor, reqPatch] = minVersion.split(".").map(Number);
    const [curMajor, curMinor, curPatch] = config.version.split(".").map(Number);

    if (
      curMajor < reqMajor ||
      (curMajor === reqMajor && curMinor < reqMinor) ||
      (curMajor === reqMajor && curMinor === reqMinor && curPatch < reqPatch)
    ) {
      throw new LedgerSquadsError(
        `App version ${config.version} is too old. Minimum required: ${minVersion}`
      );
    }
    return config;
  }

  /**
   * Get the Ed25519 public key for the given BIP32 derivation path.
   *
   * @param path - BIP32 path string (e.g., "44'/501'/0'") or array of u32 components
   * @param display - If true, show the address on the device screen for verification
   * @returns 32-byte public key as Buffer
   */
  async getAddress(
    path: string | number[],
    display = false
  ): Promise<Buffer> {
    const pathBuf = serializePath(path);
    const p1 = display ? P1_CONFIRM : P1_NON_CONFIRM;

    const response = await this.transport.send(
      CLA,
      INS_GET_PUBKEY,
      p1,
      0x00,
      pathBuf,
      [0x9000]
    );

    // Response: [pubkey_len(1)] [pubkey(32)]
    const pubkeyLen = response[0];
    return response.subarray(1, 1 + pubkeyLen);
  }

  /**
   * Sign a serialized Solana transaction message.
   *
   * The transaction is sent in chunks. The device parses Squads instructions
   * and displays clear signing information before requesting user confirmation.
   *
   * @param path - BIP32 derivation path (e.g., "44'/501'/0'")
   * @param txBuffer - Serialized Solana legacy transaction message
   * @returns 64-byte Ed25519 signature
   */
  async signTransaction(
    path: string | number[],
    txBuffer: Buffer
  ): Promise<SignResult> {
    const pathBuf = serializePath(path);

    // Build payload: path + transaction message
    const payload = Buffer.concat([pathBuf, txBuffer]);

    const response = await sendChunked(
      this.transport,
      INS_SIGN_TRANSACTION,
      0x00,
      payload
    );

    // Response: 64-byte signature
    return {
      signature: response.subarray(0, 64),
    };
  }
}
