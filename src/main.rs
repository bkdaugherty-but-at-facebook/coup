mod action;
mod player;

use action::Action;
use anyhow::Result;
use enumset::{EnumSet, EnumSetType};
use player::dumb_player::DumbPlayer;
use player::human_player::HumanPlayer;
use player::random_player::RandomPlayer;
use player::traits::Player;
use rand::seq::SliceRandom;
use std::cmp::min;
use std::collections::HashMap;

// TODO Max cards needs to sit with this
const STARTING_CARDS: u8 = 2;
const STARTING_COINS: u8 = 2;
const STARTING_LIVES: u8 = 2;
const REQUIRE_COUP_COINS: u8 = 10;
// TODO if num_lives > num_cards just reduce num_lives

// Game change turns
// Every Player
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlayerID(u8);

#[derive(Debug)]
struct Challenge {
    actor_id: PlayerID,
    challenger_id: PlayerID,
    action: Action,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum PlayerType {
    DumbCPU,
    RandomCPU,
    Local,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlayerConfig {
    player_type: PlayerType,
    player_name: String,
}

impl PlayerConfig {
    fn new(player_type: PlayerType, player_name: String) -> Self {
        PlayerConfig {
            player_type,
            player_name,
        }
    }
}

// Holds internal state about the current game
#[derive(Debug)]
pub struct GameState {
    // TODO = Convenience Cache consider removing
    active_players: Vec<PlayerID>,
    num_cards: u8,
    player_states: HashMap<PlayerID, PlayerState>,
    turn_order: Vec<PlayerID>,
    // history -> Vec of Turns?
}

pub struct GameDriver {
    field: GameField,
    // Need to be stored?
    // Do I need to make static / store playerID to player map
    // Holds the autonomous players
    players: HashMap<PlayerID, Box<dyn Player>>,
}

pub struct Game {
    driver: GameDriver,
    state: GameState,
}

impl Game {
    // Will need to decide on how to assign players / who is playing
    pub fn new(identities: EnumSet<Identity>, players: Vec<PlayerConfig>) -> Self {
        // TODO -> Yuck panic on 0?
        let num_players = players.len();
        let num_cards = match num_players {
            1..=4 => 3,
            _ => 4,
        };

        let mut driver = GameDriver::new(identities, num_cards);

        // TODO --> This is bad. Not populated yet?
        let turn_order = driver.players.keys().cloned().collect();
        let mut state = GameState::new(num_cards, turn_order);

        let mut player_id = 0;
        for entry in players {
            let id = PlayerID(player_id.clone());
            player_id = player_id + 1;

            // Create Player
            let player = match entry.player_type {
                PlayerType::DumbCPU => Box::new(RandomPlayer::new(id.clone())),
                PlayerType::RandomCPU => Box::new(RandomPlayer::new(id.clone())),
                PlayerType::Local => panic!("Unimplemented"),
            };

            state
                .player_states
                .insert(id.clone(), PlayerState::new(STARTING_LIVES));
            driver.players.insert(id.clone(), player);
        }

        Self { driver, state }
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let deck_length = self.driver.field.deck.len().clone();
        let mut deck = &mut self.driver.field.deck;
        &deck[0..deck_length].shuffle(&mut rng);
    }

    // Should this be in driver? Should driver be flattened to game?
    fn deal(&mut self, player_order: &Vec<PlayerID>) {
        for _ in 0..STARTING_CARDS {
            for id in player_order {
                let mut deck = &mut self.driver.field.deck;
                let card = deck.remove(0);
                self.driver
                    .players
                    .get_mut(&id)
                    .unwrap()
                    .take_card(&self.state, card);
            }
        }
    }

    fn update_active_players(&mut self, player_order: &Vec<PlayerID>) {
        self.state.active_players = self.active_players(player_order);
    }

    fn check_for_challenges(
        &self,
        actor_id: &PlayerID,
        turn_order: &Vec<PlayerID>,
        action: &Action,
    ) -> Option<Challenge> {
        for challenger_id in &self.active_players(turn_order) {
            // Can't challenge yourself
            if challenger_id == actor_id {
                continue;
            }
            let challenger = self.driver.players.get(challenger_id).unwrap();
            if challenger.will_challenge(&self.state, actor_id, action) {
                return Some(Challenge {
                    actor_id: actor_id.clone(),
                    challenger_id: challenger_id.clone(),
                    action: action.clone(),
                });
            }
        }
        None
    }

    pub fn play(&mut self) {
        // TODO - Establish turn order -> Roll for it? Then clockwise?
        // Find a way not to do this twice
        let turn_order = self.driver.players.keys().cloned().collect();
        self.shuffle();
        self.deal(&turn_order);
        self.update_active_players(&turn_order);

        // Start Game Loop
        while !self.game_over(&turn_order) {
            // Need to check game over everytime state changes. --> Sad
            let active_players = &self.active_players(&turn_order);
            for active_id in active_players {
                let player = self.driver.players.get(active_id).unwrap();

                // Enforce Required Coup
                let action = if player.count_coins(&self.state) < REQUIRE_COUP_COINS {
                    player.choose_action(&self.state)
                } else {
                    Action::Coup(player.choose_forced_coup(&self.state))
                };

                println!("{:?} chose action {:?}", active_id, action);

                // Allow for actions to be blocked
                for blocker_id in active_players {
                    if action.blockable(blocker_id).is_some() {
                        let blocker = self.driver.players.get(blocker_id).unwrap();
                        if let Some(blocking_action) =
                            blocker.will_block(&self.state, &active_id, &action)
                        {
                            if let Some(challenge) =
                                self.check_for_challenges(blocker_id, &turn_order, &blocking_action)
                            {
                                // TODO Need to know who won the challenge --> to resolve block
                                self.process_challenge(&challenge);
                            }
                        }
                    }
                }

                // Allow for challenging
                if action.challengable() {
                    if let Some(challenge) =
                        self.check_for_challenges(active_id, &turn_order, &action)
                    {
                        self.process_challenge(&challenge);
                    }
                }
                self.process_action(&action, active_id);
                self.update_active_players(&turn_order);

                // TODO --> This makes me very sad
                if self.game_over(&turn_order) {
                    break;
                }
            }
        }

        self.present_game_results();
    }

    fn process_challenge(&mut self, challenge: &Challenge) {
        println!("{:?}", challenge);
        // TODO - UNPACK
        let actor_id = &challenge.actor_id;
        let challenger_id = &challenge.challenger_id;
        let action = &challenge.action;

        let mut winner_id = actor_id;
        let mut loser_id = actor_id;

        let actor = self.driver.players.get_mut(actor_id).unwrap();
        if actor.can_do_action(action) {
            loser_id = challenger_id;
        } else {
            winner_id = challenger_id;
        }

        self.kill_player(loser_id);
        // let winner = self.driver.players.get_mut(winner_id).unwrap();
        // TODO - Give winner a card from the deck
    }

    fn present_game_results(&self) {
        if self.state.active_players.len() != 1 {
            println!("Uh oh... a lot of people won?");
        } else {
            println!("Player {:?} won!", self.state.active_players[0]);
        }
    }

    fn kill_player(&mut self, player_id: &PlayerID) {
        let mut victim = self.driver.players.get_mut(player_id).unwrap();
        let to_discard = victim.choose_card_to_lose(&self.state);
        let discarded = victim.discard(to_discard).unwrap();
        println!("Dying Player {:?} discarded {:#?}", player_id, discarded);
        let mut victim_state = self.state.player_states.get_mut(player_id).unwrap();
        victim_state.lost_lives.push(discarded);
        victim_state.num_lives -= 1;
    }

    fn process_action(&mut self, action: &Action, actor: &PlayerID) {
        match action {
            // TODO All constants should be defined
            Action::Income => {
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins += 1;
            }
            Action::ForeignAid => {
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins += 2;
            }
            Action::Tax => {
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins += 3;
            }
            Action::Steal(target) => {
                let mut target = self.state.player_states.get_mut(&target).unwrap();
                let coins_to_steal = min(target.num_coins, 2);
                target.num_coins -= coins_to_steal;
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins += coins_to_steal;
            }
            // TODO Trying a blocked assassination should still result in side effect
            Action::Assassinate(target) => {
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins -= 3;
                self.kill_player(target);
            }
            Action::Coup(target) => {
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins -= 7;
                self.kill_player(target);
            }
            _ => {
                println!("Unknown action... Moving on");
            }
        }
    }

    fn game_over(&self, players: &Vec<PlayerID>) -> bool {
        let mut num_alive = 0;
        for id in players {
            if self.state.player_states.get(&id).unwrap().num_lives > 0 {
                num_alive += 1;
            }
        }
        num_alive <= 1
    }
    fn active_players(&self, players: &Vec<PlayerID>) -> Vec<PlayerID> {
        players
            .into_iter()
            .cloned()
            .filter(|id| self.is_player_alive(id))
            .collect()
    }

    fn is_player_alive(&self, player_id: &PlayerID) -> bool {
        self.state.player_states.get(&player_id).unwrap().is_alive()
    }
}

impl GameState {
    fn new(num_cards: u8, turn_order: Vec<PlayerID>) -> Self {
        let player_states = HashMap::new();
        Self {
            num_cards,
            player_states,
            active_players: turn_order.iter().cloned().collect(),
            turn_order,
        }
    }
}

impl GameDriver {
    fn new(identities: EnumSet<Identity>, num_cards: u8) -> Self {
        let field = GameField::new(identities, num_cards);
        let players = HashMap::new();
        Self { field, players }
    }
}

// This is public information about a player
#[derive(Debug)]
pub struct PlayerState {
    lost_lives: Vec<Identity>,
    num_coins: u8,
    // TODO --> Make this sync with deck somehow?
    num_lives: u8,
}

impl PlayerState {
    pub fn new(num_lives: u8) -> Self {
        let lost_lives = Vec::new();
        Self {
            num_coins: STARTING_COINS,
            num_lives,
            lost_lives,
        }
    }
    pub fn is_alive(&self) -> bool {
        self.num_lives > 0
    }
}

// Can be used for cards as well?
#[derive(Debug, EnumSetType)]
pub enum Identity {
    Ambassador,
    Assassin,
    Contessa,
    Captain,
    // Inquisitor,
    Duke,
}

pub struct GameField {
    deck: Vec<Identity>,
}

impl GameField {
    fn new(identities: EnumSet<Identity>, num_cards: u8) -> Self {
        let mut deck = Vec::new();
        for identity in identities {
            for _ in 0..num_cards {
                deck.push(identity.clone())
            }
        }
        Self { deck }
    }
}

fn main() -> Result<()> {
    let game_identities = Identity::Ambassador
        | Identity::Assassin
        | Identity::Contessa
        | Identity::Captain
        | Identity::Duke;
    let players = vec![
        PlayerConfig::new(PlayerType::DumbCPU, "Martha".to_string()),
        PlayerConfig::new(PlayerType::RandomCPU, "George".to_string()),
    ];
    let mut game = Game::new(game_identities, players);
    game.play();

    // let player = HumanPlayer::new(PlayerID(1));

    // Game Driver code
    // Create Players

    Ok(())
}
