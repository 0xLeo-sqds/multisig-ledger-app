//! Known Solana program registry for human-readable labels.
//!
//! Even when we can't decode an instruction's data, we can label the program
//! so users see "Jupiter Swap" instead of a raw base58 address.
//!
//! Programs are grouped by category for the display layer.

/// Program category — helps the user understand what kind of operation this is.
#[derive(Clone, Copy)]
pub enum ProgramCategory {
    /// Core Solana infrastructure
    System,
    /// Token operations (transfer, mint, burn)
    Token,
    /// Account creation/management
    Account,
    /// Compute/resource management
    Compute,
    /// DEX / swap
    Swap,
    /// Lending / borrowing
    Lending,
    /// Staking / liquid staking
    Staking,
    /// Program deployment / upgrade
    ProgramMgmt,
    /// Governance
    Governance,
    /// Other / memo / utility
    Utility,
    /// Completely unknown
    Unknown,
}

impl ProgramCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Token => "Token",
            Self::Account => "Account",
            Self::Compute => "Compute",
            Self::Swap => "Swap",
            Self::Lending => "Lending",
            Self::Staking => "Staking",
            Self::ProgramMgmt => "Program Mgmt",
            Self::Governance => "Governance",
            Self::Utility => "Utility",
            Self::Unknown => "Program Call",
        }
    }
}

/// Known program entry.
pub struct KnownProgram {
    pub id: [u8; 32],
    pub name: &'static str,
    pub category: ProgramCategory,
}

/// Look up a program by its 32-byte ID.
/// Returns the human-readable name and category, or None if unknown.
pub fn lookup_program(id: &[u8; 32]) -> Option<(&'static str, ProgramCategory)> {
    for prog in KNOWN_PROGRAMS.iter() {
        if prog.id == *id {
            return Some((prog.name, prog.category));
        }
    }
    None
}

/// Get just the name for a program ID, or a short base58 prefix for unknown.
pub fn program_label(id: &[u8; 32]) -> &'static str {
    lookup_program(id).map(|(name, _)| name).unwrap_or("Unknown")
}

/// Get the category for display grouping.
pub fn program_category(id: &[u8; 32]) -> ProgramCategory {
    lookup_program(id)
        .map(|(_, cat)| cat)
        .unwrap_or(ProgramCategory::Unknown)
}

// --- Registry ---
// Sorted by frequency of use in Squads transactions.

const KNOWN_PROGRAMS: &[KnownProgram] = &[
    // Core Solana
    KnownProgram {
        id: super::SYSTEM_PROGRAM,
        name: "System Program",
        category: ProgramCategory::System,
    },
    KnownProgram {
        id: super::SPL_TOKEN_PROGRAM,
        name: "Token Program",
        category: ProgramCategory::Token,
    },
    KnownProgram {
        id: super::TOKEN_2022_PROGRAM,
        name: "Token-2022",
        category: ProgramCategory::Token,
    },
    KnownProgram {
        id: super::ATA_PROGRAM,
        name: "Token Account",
        category: ProgramCategory::Account,
    },
    KnownProgram {
        id: super::COMPUTE_BUDGET_PROGRAM,
        name: "Compute Budget",
        category: ProgramCategory::Compute,
    },
    KnownProgram {
        id: super::MEMO_PROGRAM,
        name: "Memo",
        category: ProgramCategory::Utility,
    },
    KnownProgram {
        id: super::BPF_LOADER_UPGRADEABLE,
        name: "BPF Loader",
        category: ProgramCategory::ProgramMgmt,
    },
    // Squads
    KnownProgram {
        id: super::super::squads::SQUADS_V4_PROGRAM_ID,
        name: "Squads v4",
        category: ProgramCategory::Governance,
    },
    // DEX / Swaps
    KnownProgram {
        id: super::JUPITER_V6_PROGRAM,
        name: "Jupiter",
        category: ProgramCategory::Swap,
    },
    KnownProgram {
        id: [
            0x04, 0x3d, 0x60, 0xff, 0xf0, 0x6b, 0x80, 0x62, 0xbb, 0x42, 0x1c, 0x3f, 0x30, 0xb5,
            0x9b, 0x5e, 0x97, 0x6d, 0x02, 0xf0, 0xf6, 0x26, 0x55, 0xca, 0x2e, 0x81, 0x67, 0x2f,
            0x43, 0x60, 0x7a, 0xeb,
        ],
        name: "Raydium AMM",
        category: ProgramCategory::Swap,
    },
    KnownProgram {
        id: [
            0xa5, 0xd5, 0xca, 0x9e, 0x04, 0xcf, 0x5d, 0xb5, 0x90, 0xb7, 0x14, 0xba, 0x2f, 0xe3,
            0x2c, 0xb1, 0x59, 0x13, 0x3f, 0xc1, 0xc1, 0x92, 0xb7, 0x22, 0x57, 0xfd, 0x07, 0xd3,
            0x9c, 0xb0, 0x40, 0x1e,
        ],
        name: "Raydium CLMM",
        category: ProgramCategory::Swap,
    },
    KnownProgram {
        id: [
            0x0e, 0x03, 0x68, 0x5f, 0x8e, 0x90, 0x90, 0x53, 0xe4, 0x58, 0x12, 0x1c, 0x66, 0xf5,
            0xa7, 0x6a, 0xed, 0xc7, 0x70, 0x6a, 0xa1, 0x1c, 0x82, 0xf8, 0xaa, 0x95, 0x2a, 0x8f,
            0x2b, 0x78, 0x79, 0xa9,
        ],
        name: "Orca Whirlpool",
        category: ProgramCategory::Swap,
    },
    KnownProgram {
        id: [
            0x05, 0xd0, 0xea, 0x4f, 0x33, 0x73, 0x70, 0x13, 0xa5, 0x63, 0xe0, 0x93, 0x48, 0xed,
            0xb6, 0xf4, 0x59, 0x3d, 0x91, 0xfc, 0x76, 0x41, 0xf9, 0x24, 0x7c, 0x24, 0x41, 0xa8,
            0x42, 0xa1, 0xbb, 0xeb,
        ],
        name: "Phoenix",
        category: ProgramCategory::Swap,
    },
    // Lending
    KnownProgram {
        id: [
            0x04, 0xb2, 0xac, 0xb1, 0x12, 0x58, 0xcc, 0xe3, 0x68, 0x28, 0xe7, 0xb8, 0x35, 0x23,
            0x40, 0x7f, 0xef, 0x6c, 0x30, 0x7e, 0x43, 0x6d, 0x42, 0xf3, 0xf4, 0x75, 0xce, 0x5a,
            0xc5, 0xde, 0x5f, 0x93,
        ],
        name: "Kamino",
        category: ProgramCategory::Lending,
    },
    KnownProgram {
        id: [
            0x05, 0x30, 0x7a, 0xd6, 0x45, 0x4b, 0xbc, 0x5e, 0x1e, 0x4e, 0x92, 0x05, 0x92, 0x53,
            0xa1, 0x8b, 0xb8, 0xc8, 0x86, 0x8c, 0x58, 0xa6, 0x31, 0x2e, 0xc8, 0x6a, 0x39, 0xe6,
            0x22, 0x4e, 0x37, 0x3b,
        ],
        name: "Marginfi",
        category: ProgramCategory::Lending,
    },
    KnownProgram {
        id: [
            0x09, 0x54, 0xdb, 0xbe, 0x9e, 0xc9, 0x60, 0xc9, 0x8a, 0x7a, 0x29, 0x3f, 0xe2, 0x13,
            0x36, 0x96, 0x6f, 0xe1, 0x80, 0xd1, 0x51, 0xae, 0x4b, 0x81, 0x79, 0x56, 0x1f, 0x89,
            0x85, 0x4a, 0x53, 0xf6,
        ],
        name: "Drift",
        category: ProgramCategory::Lending,
    },
    // Staking
    KnownProgram {
        id: super::MARINADE_PROGRAM,
        name: "Marinade",
        category: ProgramCategory::Staking,
    },
    KnownProgram {
        id: super::JITO_STAKE_POOL,
        name: "Jito",
        category: ProgramCategory::Staking,
    },
    KnownProgram {
        id: [
            0x06, 0x80, 0x65, 0x76, 0x96, 0xcb, 0x8d, 0xe6, 0xe5, 0x31, 0xdc, 0x18, 0xfd, 0xd9,
            0x67, 0x79, 0x5f, 0x1a, 0x1e, 0x99, 0x0b, 0x30, 0xe6, 0x15, 0x98, 0x5c, 0x83, 0xcf,
            0x01, 0x73, 0x22, 0x33,
        ],
        name: "Sanctum",
        category: ProgramCategory::Staking,
    },
    // Governance
    KnownProgram {
        id: [
            0xea, 0xe4, 0x35, 0xbd, 0xee, 0x75, 0xb7, 0x34, 0xcd, 0x59, 0x3e, 0xcf, 0x9a, 0x30,
            0x4b, 0x80, 0x24, 0xba, 0x28, 0x98, 0x67, 0xb7, 0x69, 0xb1, 0xf9, 0x3c, 0xa7, 0xbb,
            0xb8, 0x8e, 0x46, 0xfe,
        ],
        name: "Realms",
        category: ProgramCategory::Governance,
    },
];
