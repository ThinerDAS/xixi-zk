use crate::model::{self, GameConfig, PlayerState};
use anyhow::{bail, Result};
use rkyv::Archived;

// Core game engine module
mod game_engine {
    use super::*;

    // Main game simulation entry point
    pub fn simulate_game(config: &Archived<GameConfig>, route: &[u32]) -> Result<PlayerState> {
        let mut game = Game::new(config);
        game.execute_route(route)
    }

    // Top-level game orchestrator
    struct Game<'a> {
        state: GameState,
        context: GameContext<'a>,
    }

    impl<'a> Game<'a> {
        fn new(config: &'a Archived<GameConfig>) -> Self {
            if config.major_adj.is_empty() || config.major_desc.is_empty() {
                panic!("Invalid game config: empty major node data");
            }
            if config.major_adj.len() != config.major_desc.len() {
                panic!("major_adj and major_desc length mismatch");
            }

            Self {
                state: GameState::new(config),
                context: GameContext::new(config),
            }
        }

        fn execute_route(&mut self, route: &[u32]) -> Result<PlayerState> {
            #[cfg(debug_assertions)]
            eprintln!("Initial state: {:?}", self.state.player);

            for &node in route {
                self.execute_major_node(node)?;
            }

            Ok(self.state.player.clone())
        }

        fn execute_major_node(&mut self, node: u32) -> Result<()> {
            let node_idx = node as usize;

            if self.state.is_major_completed(node_idx) {
                bail!("Node {} already completed", node);
            }

            if !self
                .context
                .can_execute_major(node, &self.state.completed_majors)
            {
                bail!("Node {} cannot be executed at this point", node);
            }

            self.context.process_major_node(&mut self.state, node)?;
            self.state.mark_major_completed(node_idx);
            Ok(())
        }
    }

    // Player state with completion tracking
    struct GameState {
        player: PlayerState,
        completed_majors: Vec<bool>,
        completed_minors: Vec<bool>,
    }

    impl GameState {
        fn new(config: &Archived<GameConfig>) -> Self {
            let node_count = config.major_desc.len();
            let mut completed_majors = vec![false; node_count];
            completed_majors[0] = true;

            Self {
                player: PlayerState::from_init_stats(&config.init_stat),
                completed_majors,
                completed_minors: vec![false; config.minor_desc.len()],
            }
        }

        #[inline]
        fn is_major_completed(&self, node: usize) -> bool {
            node < self.completed_majors.len() && self.completed_majors[node]
        }

        #[inline]
        fn mark_major_completed(&mut self, node: usize) {
            if node < self.completed_majors.len() {
                self.completed_majors[node] = true;
            }
        }

        #[inline]
        fn is_minor_completed(&self, node: usize) -> bool {
            node < self.completed_minors.len() && self.completed_minors[node]
        }

        #[inline]
        fn mark_minor_completed(&mut self, node: usize) {
            if node < self.completed_minors.len() {
                self.completed_minors[node] = true;
            }
        }
    }

    // Game context with immutable references
    struct GameContext<'a> {
        config: &'a Archived<GameConfig>,
    }

    impl<'a> GameContext<'a> {
        fn new(config: &'a Archived<GameConfig>) -> Self {
            Self { config }
        }

        /// Checks if a major node can be executed
        ///
        /// # Arguments
        /// * `node` - Node ID to check
        /// * `completed` - Array tracking completed nodes
        ///
        /// # Returns
        /// * `true` if:
        ///   - Node ID is valid (>0)
        ///   - Node exists in config
        ///   - At least one prerequisite node is completed
        /// * `false` otherwise
        fn can_execute_major(&self, node: u32, completed: &[bool]) -> bool {
            // Node 0 is invalid (usually root or special node)
            if node == 0 {
                return false;
            }

            let node_idx = node as usize;
            
            // Get adjacent nodes from config
            let Some(adjacent_nodes) = self.config.major_adj.get(node_idx) else {
                #[cfg(debug_assertions)]
                eprintln!("Node {} not found in major_adj", node);
                return false;
            };

            // Check if any adjacent node is completed
            // Using direct indexing is safe here since we control the node IDs
            adjacent_nodes.iter().any(|&adj_node| {
                let adj_idx = adj_node as usize;
                adj_idx < completed.len() && completed[adj_idx]
            })
        }

        fn process_major_node(&self, state: &mut GameState, node: u32) -> Result<()> {
            // Phase 1: Handle major node effect
            self.apply_major_effect(state, node)?;

            // Phase 2: Process unlockable minor nodes
            self.unlock_minor_nodes(state, node);

            // Phase 3: Post-processing
            self.post_process_state(state)
        }

        fn apply_major_effect(&self, state: &mut GameState, node: u32) -> Result<()> {
            match &self.config.major_desc[node as usize] {
                Archived::<model::MajorDesc>::Enemy(enemy_idx) => {
                    let enemy = &self.config.enemy_data[*enemy_idx as usize];
                    state.player.battle_enemy(enemy)
                }
                Archived::<model::MajorDesc>::Delta(attrs) => {
                    state.player.apply_attribute_changes(attrs)
                }
            }
        }

        fn unlock_minor_nodes(&self, state: &mut GameState, major_node: u32) {
            let Some(minor_nodes) = self.config.major_minor_adj.get(major_node as usize) else {
                return;
            };

            for &minor_idx in minor_nodes.iter() {
                let minor_idx = minor_idx as usize;
                if !state.is_minor_completed(minor_idx) {
                    state.mark_minor_completed(minor_idx);
                    if let Some(minor_desc) = self.config.minor_desc.get(minor_idx) {
                        state.player.apply_minor_bonuses(minor_desc);
                    }
                }
            }
        }

        fn post_process_state(&self, state: &mut GameState) -> Result<()> {
            let node_count = self.config.major_adj.len() as i32;
            state.player.normalize_resources(node_count)?;
            state.player.handle_level_progression(&self.config);
            Ok(())
        }
    }
}

// Player state extensions
trait PlayerStateOperations {
    fn from_init_stats(init: &Archived<PlayerState>) -> Self;
    fn battle_enemy(&mut self, enemy: &Archived<model::Enemy>) -> Result<()>;
    fn apply_attribute_changes(
        &mut self,
        attrs: &Archived<Vec<(model::AttrType, i32)>>,
    ) -> Result<()>;
    fn apply_minor_bonuses(&mut self, minor_desc: &Archived<model::MinorDesc>);
    fn normalize_resources(&mut self, node_count: i32) -> Result<()>;
    fn handle_level_progression(&mut self, config: &Archived<model::GameConfig>);
}

impl PlayerStateOperations for PlayerState {
    fn from_init_stats(init: &Archived<PlayerState>) -> Self {
        Self {
            hp: init.hp,
            atk: init.atk,
            def: init.def,
            mdef: init.mdef,
            exp: init.exp,
            lv: init.lv,
            salt: init.salt,
            big_salt: init.big_salt,
        }
    }

    fn battle_enemy(&mut self, enemy: &Archived<model::Enemy>) -> Result<()> {
        let (damage, penalty) = CombatCalculator::calculate_damage(self, enemy);

        #[cfg(debug_assertions)]
        {
            eprintln!("Before battle with enemy:");
            eprintln!("  HP: {}, Damage: {}", self.hp, damage);
        }

        self.hp = self.hp.saturating_sub(damage);
        self.big_salt = self.big_salt.saturating_add(penalty);
        self.exp = self.exp.saturating_add(enemy.exp);

        Ok(())
    }

    fn apply_attribute_changes(
        &mut self,
        attrs: &Archived<Vec<(model::AttrType, i32)>>,
    ) -> Result<()> {
        for &(ref attr_type, delta) in attrs.as_slice() {
            match attr_type {
                Archived::<model::AttrType>::Hp => self.hp = self.hp.saturating_add(delta),
                Archived::<model::AttrType>::Atk => ResourceManager::apply_safe_attribute_change(
                    &mut self.atk,
                    delta,
                    &mut self.big_salt,
                ),
                Archived::<model::AttrType>::Def => ResourceManager::apply_safe_attribute_change(
                    &mut self.def,
                    delta,
                    &mut self.big_salt,
                ),
                Archived::<model::AttrType>::Mdef => ResourceManager::apply_safe_attribute_change(
                    &mut self.mdef,
                    delta,
                    &mut self.big_salt,
                ),
                Archived::<model::AttrType>::Exp => self.exp = self.exp.saturating_add(delta),
                Archived::<model::AttrType>::Lv => self.lv = (self.lv as i32 + delta).max(0) as u32,
                Archived::<model::AttrType>::Salt => self.salt = self.salt.saturating_add(delta),
                Archived::<model::AttrType>::BigSalt => {
                    self.big_salt = self.big_salt.saturating_add(delta)
                }
            }
        }
        Ok(())
    }

    // this is the most ridiculous one - this function must be called tons of times
    // but marking it cold improves performance
    #[cold]
    fn apply_minor_bonuses(&mut self, minor_desc: &Archived<model::MinorDesc>) {
        if minor_desc.atk != 0 {
            self.atk = self.atk.saturating_add(minor_desc.atk);
        }
        if minor_desc.def != 0 {
            self.def = self.def.saturating_add(minor_desc.def);
        }
        if minor_desc.hp != 0 {
            self.hp = self.hp.saturating_add(minor_desc.hp);
        }
        if minor_desc.mdef != 0 {
            self.mdef = self.mdef.saturating_add(minor_desc.mdef);
        }

        #[cfg(debug_assertions)]
        {
            eprintln!("Applied minor node bonuses:");
            eprintln!("  ATK: +{} -> {}", minor_desc.atk, self.atk);
            eprintln!("  DEF: +{} -> {}", minor_desc.def, self.def);
            eprintln!("  HP: +{} -> {}", minor_desc.hp, self.hp);
            eprintln!("  MDEF: +{} -> {}", minor_desc.mdef, self.mdef);
        }
    }

    fn normalize_resources(&mut self, node_count: i32) -> Result<()> {
        ResourceManager::convert_resources(self, node_count)
    }

    fn handle_level_progression(&mut self, config: &Archived<model::GameConfig>) {
        level_up::process(self, config);
    }
}

// Combat calculation subsystem
struct CombatCalculator;
impl CombatCalculator {
    fn calculate_damage(player: &PlayerState, enemy: &Archived<model::Enemy>) -> (i32, i32) {
        if player.atk <= enemy.def {
            return Self::calculate_damage_low_attack(player, enemy);
        }

        let per = if enemy.solid {
            1
        } else {
            player.atk.saturating_sub(enemy.def)
        }
        .max(1);

        let mut n = (enemy.hp - 1) / per;
        if enemy.speedy {
            n = n.saturating_add(1);
        }
        n = n.saturating_mul(enemy.attimes);

        let per_e = if enemy.magic {
            enemy.atk
        } else {
            enemy.atk.saturating_sub(player.def)
        }
        .max(0);

        match per_e.checked_mul(n) {
            Some(total) => (total.saturating_sub(player.mdef).max(0), 0),
            None => Self::calculate_damage_overflow(),
        }
    }

    #[cold]
    fn calculate_damage_low_attack(
        player: &PlayerState,
        enemy: &Archived<model::Enemy>,
    ) -> (i32, i32) {
        let diff = enemy.def.saturating_sub(player.atk);
        (diff.saturating_mul(256), 2)
    }

    #[cold]
    fn calculate_damage_overflow() -> (i32, i32) {
        (i32::MAX, 1)
    }
}

// Resource management helper
struct ResourceManager;
impl ResourceManager {
    fn apply_safe_attribute_change(value: &mut i32, delta: i32, big_salt: &mut i32) {
        match value.checked_add(delta) {
            Some(new_val) if new_val >= 0 => *value = new_val,
            _ => Self::apply_safe_attribute_change_cold(value, delta, big_salt),
        }
    }

    #[cold]
    fn apply_safe_attribute_change_cold(value: &mut i32, delta: i32, big_salt: &mut i32) {
        match value.checked_add(delta) {
            Some(new_val) if new_val >= 0 => *value = new_val,
            Some(new_val) => {
                *big_salt = big_salt.saturating_add(new_val.abs());
                *value = 0;
            }
            None if delta > 0 => *value = i32::MAX,
            None => {
                *big_salt = big_salt.saturating_add(delta.abs().saturating_sub(*value));
                *value = 0;
            }
        }
    }

    fn convert_resources(player: &mut PlayerState, node_count: i32) -> Result<()> {
        // Quick path when no resource conversion needed
        if player.hp > 0 && player.big_salt == 0 && player.salt == 0 {
            return Ok(());
        }
        Self::convert_resources_slow_path(player, node_count)
    }

    #[cold]
    // this way works greatly gives a noticeable performance boost
    fn convert_resources_slow_path(player: &mut PlayerState, node_count: i32) -> Result<()> {
        // HP normalization

        // Handle HP overflow case
        if player.hp < i32::MIN / 2 {
            player.big_salt = player
                .big_salt
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("big_salt overflow during HP overflow handling"))?;
            player.hp = 1;
        }
        // Handle normal low HP case
        else if player.hp <= 0 {
            player.salt = player
                .salt
                .checked_add(1 - player.hp)
                .ok_or_else(|| anyhow::anyhow!("salt overflow during HP conversion"))?;
            player.hp = 1;
        }

        // Big salt conversion
        if player.big_salt > 0 {
            player.salt = player
                .salt
                .checked_add(player.big_salt.saturating_mul(65536))
                .ok_or_else(|| anyhow::anyhow!("salt overflow during big_salt conversion"))?;
        }

        // Passive salt generation
        if player.salt > 0 {
            player.salt = player
                .salt
                .checked_add(player.salt / node_count + 1)
                .ok_or_else(|| anyhow::anyhow!("salt overflow during salt generation"))?;
        }

        Ok(())
    }
}

// Level up processing module
mod level_up {
    use super::*;

    pub fn process(player: &mut PlayerState, config: &Archived<model::GameConfig>) {
        while let Some(req) = config.levelup_desc.get(player.lv as usize) {
            if player.exp < req.need {
                break;
            }
            process_level_up(player, config, req);
        }
    }
    #[cold]
    fn process_level_up(
        player: &mut PlayerState,
        config: &Archived<model::GameConfig>,
        req: &Archived<model::LevelUp>,
    ) {
        // seems not very good to mark it cold but this really improve performance in our case
        if req.clear {
            player.exp = player.exp.saturating_sub(req.need);
        }
        player.lv += 1;

        if let Some(minor_desc) = config.minor_desc.get(req.minor as usize) {
            player.apply_minor_bonuses(minor_desc);
        }
    }
}

// Re-export the main function
pub use game_engine::simulate_game;
