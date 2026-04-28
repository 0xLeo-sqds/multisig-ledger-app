"""
Test GET_APP_CONFIGURATION and GET_VERSION commands.
"""

# CLA and INS constants matching src/main.rs
CLA = 0xE0
INS_GET_VERSION = 0x03
INS_GET_APP_CONFIGURATION = 0x04


def test_get_version(backend):
    """GET_VERSION returns 3-byte version (major, minor, patch)."""
    response = backend.exchange(cla=CLA, ins=INS_GET_VERSION, p1=0x00, p2=0x00, data=b"")
    assert len(response.data) == 3
    major, minor, patch = response.data[0], response.data[1], response.data[2]
    assert major == 0  # v0.1.0
    assert minor == 1
    assert patch == 0


def test_get_app_configuration(backend):
    """GET_APP_CONFIGURATION returns blind_signing flag + reserved + version."""
    response = backend.exchange(cla=CLA, ins=INS_GET_APP_CONFIGURATION, p1=0x00, p2=0x00, data=b"")
    assert len(response.data) == 5
    blind_signing = response.data[0]
    assert blind_signing == 0  # disabled by default
