"""
Pytest configuration for Squads Ledger app standalone tests.
Uses Ragger framework for Speculos-based functional testing.
"""
import pytest


def pytest_addoption(parser):
    parser.addoption("--device", default="nanosp", help="Device model: nanosp, nanox, stax, flex")
