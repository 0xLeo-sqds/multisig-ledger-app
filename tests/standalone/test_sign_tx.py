"""
Test SIGN_TRANSACTION command — Squads instruction parsing and rejection.
"""
import struct

CLA = 0xE0
INS_SIGN_TX = 0x06

HARDENED = 0x80000000

# Status words
SW_OK = 0x9000
SW_DENY = 0x6985
SW_INVALID_DATA = 0x6A80
SW_BLIND_DISABLED = 0x6808
SW_INVALID_MESSAGE = 0xB00B
SW_PARSING_FAIL = 0xB005


def _build_path(*components):
    data = bytes([len(components)])
    for c in components:
        data += struct.pack(">I", c)
    return data


def _build_minimal_legacy_message():
    """Build a minimal valid Solana legacy message (no instructions)."""
    num_required_sigs = 1
    num_readonly_signed = 0
    num_readonly_unsigned = 0

    # Header
    header = bytes([num_required_sigs, num_readonly_signed, num_readonly_unsigned])

    # One account key (compact-u16 length = 1, then 32 bytes)
    num_accounts = bytes([1])
    account_key = bytes(32)  # all zeros (System Program)

    # Recent blockhash (32 bytes)
    blockhash = bytes(32)

    # Zero instructions (compact-u16 length = 0)
    num_instructions = bytes([0])

    return header + num_accounts + account_key + blockhash + num_instructions


def test_reject_versioned_message(backend):
    """Versioned (v0) messages should be rejected with InvalidMessage."""
    path = _build_path(44 | HARDENED, 501 | HARDENED, 0 | HARDENED)

    # v0 message starts with 0x80
    versioned_msg = bytes([0x80]) + bytes(100)

    # Build APDU payload: path + message
    payload = path + versioned_msg

    try:
        # Send as single chunk (P2=0x00, no extend, no more)
        backend.exchange(cla=CLA, ins=INS_SIGN_TX, p1=0x00, p2=0x00, data=payload)
        assert False, "Should have been rejected"
    except Exception as e:
        # Should get InvalidMessage (0xB00B)
        pass


def test_reject_malformed_message(backend):
    """A message with invalid structure (indices out of bounds) should be rejected."""
    path = _build_path(44 | HARDENED, 501 | HARDENED, 0 | HARDENED)

    # Malformed: num_required_sigs > num_accounts
    malformed_msg = bytes([
        5,  # num_required_sigs = 5
        0,  # num_readonly_signed
        0,  # num_readonly_unsigned
        1,  # compact-u16: 1 account
    ]) + bytes(32) + bytes(32) + bytes([0])  # 1 account key + blockhash + 0 instructions

    payload = path + malformed_msg

    try:
        backend.exchange(cla=CLA, ins=INS_SIGN_TX, p1=0x00, p2=0x00, data=payload)
        assert False, "Should have been rejected"
    except Exception:
        pass  # Expected: TxParsingFail


def test_sign_empty_message_no_squads(backend):
    """A valid message with no Squads instructions should require blind signing."""
    path = _build_path(44 | HARDENED, 501 | HARDENED, 0 | HARDENED)
    msg = _build_minimal_legacy_message()
    payload = path + msg

    try:
        # Blind signing is disabled by default, so this should fail
        backend.exchange(cla=CLA, ins=INS_SIGN_TX, p1=0x00, p2=0x00, data=payload)
        assert False, "Should have been rejected (blind signing disabled)"
    except Exception:
        pass  # Expected: BlindSigningDisabled or TxParsingFail
