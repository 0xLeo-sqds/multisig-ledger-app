//! Zero-copy parser for config_transaction_create instruction data.
//!
//! The instruction data layout:
//!   [discriminator(8)] [action_count(u32)] [actions...]
//!
//! Each ConfigAction is tag-dispatched:
//!   0: AddMember { member(32), permissions(1) }
//!   1: RemoveMember { member(32) }
//!   2: ChangeThreshold { new_threshold(u16) }
//!   3: SetTimeLock { new_time_lock(u32) }
//!   4: AddSpendingLimit { create_key(32), vault_index(1), mint(32), amount(u64),
//!                         period(1), members_count(u32), members(N*32),
//!                         destinations_count(u32), destinations(N*32) }
//!   5: RemoveSpendingLimit { spending_limit(32) }
//!   6: SetRentCollector { option_tag(1), [pubkey(32) if tag==1] }

use super::{ParseError, Reader};

/// Maximum config actions we'll parse.
const MAX_CONFIG_ACTIONS: usize = 8;

/// Maximum members/destinations in a spending limit.
const MAX_PUBKEY_LIST: usize = 16;

/// Parsed config action — no_std friendly, no Vec.
#[derive(Clone, Copy)]
pub enum ConfigActionParsed {
    AddMember {
        member: [u8; 32],
        permissions: u8,
    },
    RemoveMember {
        member: [u8; 32],
    },
    ChangeThreshold {
        new_threshold: u16,
    },
    SetTimeLock {
        new_time_lock: u32,
    },
    AddSpendingLimit {
        vault_index: u8,
        mint: [u8; 32],
        amount: u64,
        period: SpendingLimitPeriod,
        num_members: u32,
        num_destinations: u32,
    },
    RemoveSpendingLimit {
        spending_limit: [u8; 32],
    },
    SetRentCollector {
        new_rent_collector: Option<[u8; 32]>,
    },
    Unknown {
        tag: u8,
    },
}

#[derive(Clone, Copy)]
pub enum SpendingLimitPeriod {
    OneTime,
    Day,
    Week,
    Month,
}

impl SpendingLimitPeriod {
    pub fn label(self) -> &'static str {
        match self {
            Self::OneTime => "One-time",
            Self::Day => "Per day",
            Self::Week => "Per week",
            Self::Month => "Per month",
        }
    }
}

/// Parsed config transaction metadata.
pub struct ConfigTxMeta {
    pub actions: [ConfigActionParsed; MAX_CONFIG_ACTIONS],
    pub num_actions: usize,
}

/// Parse a config_transaction_create instruction's data.
pub fn parse_config_tx_create(ix_data: &[u8]) -> Result<ConfigTxMeta, ParseError> {
    let mut r = Reader::new(ix_data);

    // Skip discriminator (already verified by caller)
    r.skip(8)?;

    // Action count
    let action_count = r.read_u32_le()?;
    if action_count as usize > MAX_CONFIG_ACTIONS {
        return Err(ParseError::InvalidStructure);
    }

    let mut actions = [ConfigActionParsed::Unknown { tag: 255 }; MAX_CONFIG_ACTIONS];
    for i in 0..action_count as usize {
        actions[i] = parse_config_action(&mut r)?;
    }

    Ok(ConfigTxMeta {
        actions,
        num_actions: action_count as usize,
    })
}

fn parse_config_action(r: &mut Reader<'_>) -> Result<ConfigActionParsed, ParseError> {
    let tag = r.read_u8()?;
    match tag {
        0 => {
            // AddMember
            let member = *r.read_pubkey()?;
            let permissions = r.read_u8()?;
            Ok(ConfigActionParsed::AddMember {
                member,
                permissions,
            })
        }
        1 => {
            // RemoveMember
            let member = *r.read_pubkey()?;
            Ok(ConfigActionParsed::RemoveMember { member })
        }
        2 => {
            // ChangeThreshold
            let new_threshold = r.read_u16_le()?;
            Ok(ConfigActionParsed::ChangeThreshold { new_threshold })
        }
        3 => {
            // SetTimeLock
            let new_time_lock = r.read_u32_le()?;
            Ok(ConfigActionParsed::SetTimeLock { new_time_lock })
        }
        4 => {
            // AddSpendingLimit
            let _create_key = r.read_pubkey()?; // skip create_key (derived, not displayed)
            let vault_index = r.read_u8()?;
            let mint = *r.read_pubkey()?;
            let amount = r.read_u64_le()?;
            let period_tag = r.read_u8()?;
            let period = match period_tag {
                0 => SpendingLimitPeriod::OneTime,
                1 => SpendingLimitPeriod::Day,
                2 => SpendingLimitPeriod::Week,
                3 => SpendingLimitPeriod::Month,
                _ => return Err(ParseError::InvalidTag { tag: period_tag }),
            };
            // Members list — read count and skip the pubkeys
            let num_members = r.read_u32_le()?;
            if num_members > MAX_PUBKEY_LIST as u32 {
                return Err(ParseError::InvalidStructure);
            }
            r.skip(num_members as usize * 32)?;
            // Destinations list
            let num_destinations = r.read_u32_le()?;
            if num_destinations > MAX_PUBKEY_LIST as u32 {
                return Err(ParseError::InvalidStructure);
            }
            r.skip(num_destinations as usize * 32)?;
            Ok(ConfigActionParsed::AddSpendingLimit {
                vault_index,
                mint,
                amount,
                period,
                num_members,
                num_destinations,
            })
        }
        5 => {
            // RemoveSpendingLimit
            let spending_limit = *r.read_pubkey()?;
            Ok(ConfigActionParsed::RemoveSpendingLimit { spending_limit })
        }
        6 => {
            // SetRentCollector — Option<Pubkey>
            let option_tag = r.read_u8()?;
            let new_rent_collector = match option_tag {
                0 => None,
                1 => Some(*r.read_pubkey()?),
                _ => return Err(ParseError::InvalidTag { tag: option_tag }),
            };
            Ok(ConfigActionParsed::SetRentCollector { new_rent_collector })
        }
        _ => {
            // Unknown action — skip remaining data (best effort)
            Ok(ConfigActionParsed::Unknown { tag })
        }
    }
}
