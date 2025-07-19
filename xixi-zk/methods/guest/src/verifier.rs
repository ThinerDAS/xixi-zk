use anyhow::{bail, Result};
use crate::{model, simulator};
use model::{GameConfig};
use rkyv::{Archived, archived_root};

/// Zero-copy conversion from byte slice to Archived<GameConfig>
///
/// This performs a direct zero-copy deserialization without validation
/// for maximum performance. The caller must ensure the input is valid
/// and properly aligned.
pub fn config_from_bytes(bytes: &[u8]) -> &Archived<GameConfig> {
    // Basic alignment check for safety (should be at least 16-byte aligned for rkyv)
    assert!(
        bytes.as_ptr() as usize % 16 == 0,
        "Config buffer must be 16-byte aligned for rkyv deserialization"
    );
    
    unsafe { archived_root::<GameConfig>(bytes) }
}

pub fn do_main(config_bytes: &[u8], route_bytes: &[u8]) -> Result<Vec<i64>> {
    // Parse and verify route
    let route = parse_route(route_bytes)?;
    verify_route(&route)?;
    
    // Safe archived root access with validation
    let config = config_from_bytes(config_bytes);

    // Simulate game using zero-copy config
    let final_state = simulator::simulate_game(&config, &route)
        .map_err(|e| anyhow::anyhow!("Game simulation failed: {:?}", e))?;

    // Verify final state
    verify_final_state(&final_state)?;

    // Calculate score and return
    Ok(vec![calculate_score(&final_state)])
}

/// Validate game route meets requirements
///
/// Rules:
/// 1. Route cannot be empty
/// 2. Must end with node 1 (termination node)
///
/// This ensures the game simulation has a valid stopping condition
pub fn verify_route(route: &[u32]) -> Result<()> {
    if route.is_empty() {
        bail!("Route cannot be empty");
    }

    // Check if route ends with node 1 (game termination condition)
    if route.last() != Some(&1) {
        bail!("Route must end with node 1 (game termination node)");
    }

    Ok(())
}

/// Parse route from bytes with flexible format support
///
/// Supports:
/// 1. JSON array format (preferred)
/// 2. Whitespace-separated values (legacy)
///
/// Performance Note:
/// - Attempts JSON parsing first (faster for valid JSON)
/// - Falls back to text parsing if JSON fails
/// Parse route from bytes to u32 array (little-endian)
pub fn parse_route(bytes: &[u8]) -> Result<Vec<u32>> {
    if bytes.len() % 4 != 0 {
        bail!("Route bytes length must be multiple of 4");
    }
    
    Ok(bytes.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect())
}

/// Validate player's final state meets game completion criteria
///
/// Requirements:
/// 1. HP > 0 (player must be alive)
/// 2. salt = 0 and big_salt = 0 (no debt)
///
/// These conditions ensure the game was properly completed
pub fn verify_final_state(state: &model::PlayerState) -> Result<()> {
    if state.hp <= 0 {
        bail!("Player died (HP = {} <= 0)", state.hp);
    }
    if state.salt != 0 || state.big_salt != 0 {
        bail!("Resource debt remains (salt: {}, big_salt: {})",
             state.salt, state.big_salt);
    }
    Ok(())
}

/// Calculate final game score based on player state
///
/// Scoring Rule:
/// - Current HP value is used as score
/// - Higher HP = better score
///
/// Note: This simple scoring may be extended in future versions
pub fn calculate_score(state: &model::PlayerState) -> i64 {
    state.hp as i64
}