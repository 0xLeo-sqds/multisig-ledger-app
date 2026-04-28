/**
 * Typed error hierarchy for the Squads Ledger SDK.
 * Maps APDU status words to user-friendly error classes.
 */

export class LedgerSquadsError extends Error {
  constructor(
    message: string,
    public readonly statusWord?: number
  ) {
    super(message);
    this.name = "LedgerSquadsError";
  }
}

/** User rejected the operation on the device screen. */
export class UserRejectedError extends LedgerSquadsError {
  constructor() {
    super("User rejected on device", 0x6985);
    this.name = "UserRejectedError";
  }
}

/** Blind signing is disabled in app settings but the transaction requires it. */
export class BlindSigningDisabledError extends LedgerSquadsError {
  constructor() {
    super(
      "Blind signing is disabled. Enable it in the Squads app settings on your Ledger.",
      0x6808
    );
    this.name = "BlindSigningDisabledError";
  }
}

/** The Squads app is not open on the device, or a different app is active. */
export class WrongAppError extends LedgerSquadsError {
  constructor() {
    super(
      "Squads app is not open on the device. Please open it and try again.",
      0x6e00
    );
    this.name = "WrongAppError";
  }
}

/** The device is locked (PIN required). */
export class DeviceLockedError extends LedgerSquadsError {
  constructor() {
    super("Device is locked. Please unlock it and try again.", 0x5515);
    this.name = "DeviceLockedError";
  }
}

/** Invalid data sent to the device. */
export class InvalidDataError extends LedgerSquadsError {
  constructor() {
    super("Invalid data sent to device", 0x6a80);
    this.name = "InvalidDataError";
  }
}

/** Invalid or versioned Solana message. */
export class InvalidMessageError extends LedgerSquadsError {
  constructor() {
    super(
      "Invalid or versioned Solana message. The Squads app only supports legacy messages.",
      0xb00b
    );
    this.name = "InvalidMessageError";
  }
}

/**
 * Map an APDU status word to a typed error.
 * Returns undefined for success (0x9000).
 */
export function statusWordToError(sw: number): LedgerSquadsError | undefined {
  switch (sw) {
    case 0x9000:
      return undefined;
    case 0x6985:
      return new UserRejectedError();
    case 0x6808:
      return new BlindSigningDisabledError();
    case 0x6e00:
      return new WrongAppError();
    case 0x5515:
    case 0x6982:
      return new DeviceLockedError();
    case 0x6a80:
      return new InvalidDataError();
    case 0xb00b:
      return new InvalidMessageError();
    default:
      if (sw >= 0x6f00 && sw <= 0x6fff) {
        return new LedgerSquadsError("Internal device error", sw);
      }
      return new LedgerSquadsError(
        `Unknown error: 0x${sw.toString(16).padStart(4, "0")}`,
        sw
      );
  }
}
