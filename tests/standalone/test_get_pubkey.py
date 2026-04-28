"""
Test GET_PUBKEY command with BIP32 path validation.
"""
import struct

CLA = 0xE0
INS_GET_PUBKEY = 0x05


def _build_path(*components):
    """Build a BIP32 path payload: [num_components, u32_be, u32_be, ...]"""
    data = bytes([len(components)])
    for c in components:
        data += struct.pack(">I", c)
    return data


HARDENED = 0x80000000


def test_get_pubkey_valid_path(backend):
    """GET_PUBKEY with valid 44'/501'/0' path returns 32-byte pubkey."""
    path = _build_path(44 | HARDENED, 501 | HARDENED, 0 | HARDENED)
    response = backend.exchange(cla=CLA, ins=INS_GET_PUBKEY, p1=0x00, p2=0x00, data=path)
    # Response: [pubkey_len, pubkey_bytes...]
    pubkey_len = response.data[0]
    assert pubkey_len == 32
    assert len(response.data) == 1 + 32


def test_get_pubkey_4_components(backend):
    """GET_PUBKEY with 44'/501'/0'/0' (4 components) also works."""
    path = _build_path(44 | HARDENED, 501 | HARDENED, 0 | HARDENED, 0 | HARDENED)
    response = backend.exchange(cla=CLA, ins=INS_GET_PUBKEY, p1=0x00, p2=0x00, data=path)
    pubkey_len = response.data[0]
    assert pubkey_len == 32


def test_get_pubkey_wrong_purpose_rejected(backend):
    """GET_PUBKEY with wrong purpose (not 44') should be rejected."""
    path = _build_path(45 | HARDENED, 501 | HARDENED, 0 | HARDENED)
    try:
        backend.exchange(cla=CLA, ins=INS_GET_PUBKEY, p1=0x00, p2=0x00, data=path)
        assert False, "Should have been rejected"
    except Exception:
        pass  # Expected: WrongP1P2 (0x6A86)


def test_get_pubkey_wrong_coin_rejected(backend):
    """GET_PUBKEY with wrong coin type (not 501') should be rejected."""
    path = _build_path(44 | HARDENED, 60 | HARDENED, 0 | HARDENED)  # Ethereum coin type
    try:
        backend.exchange(cla=CLA, ins=INS_GET_PUBKEY, p1=0x00, p2=0x00, data=path)
        assert False, "Should have been rejected"
    except Exception:
        pass  # Expected: WrongP1P2 (0x6A86)


def test_get_pubkey_unhardened_rejected(backend):
    """GET_PUBKEY with non-hardened component should be rejected."""
    path = _build_path(44 | HARDENED, 501 | HARDENED, 0)  # account not hardened
    try:
        backend.exchange(cla=CLA, ins=INS_GET_PUBKEY, p1=0x00, p2=0x00, data=path)
        assert False, "Should have been rejected"
    except Exception:
        pass  # Expected: WrongP1P2 (0x6A86)
