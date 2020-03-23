mod action;
mod player;

use action::Action;
use anyhow::Result;
use enumset::{EnumSet, EnumSetType};
use player::dumb_player::DumbPlayer;
use player::human_player::HumanPlayer;
use player::traits::Player;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::convert::TryInto;

// TODO Max cards needs to sit with this
const STARTING_CARDS: u8 = 2;
const STARTING_COINS: u8 = 2;
const STARTING_LIVES: u8 = 2;

// Game change turns
// Every Player
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlayerID(u8);

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
    pub fn new(identities: EnumSet<Identity>, num_players: u8) -> Self {
        // TODO -> Yuck panic on 0?
        let num_cards = match num_players {
            1..=4 => 3,
            _ => 4,
        };

        let mut driver = GameDriver::new(identities, num_cards);
        let turn_order = driver.players.keys().cloned().collect();
        let mut state = GameState::new(num_cards, turn_order);
        for id in 0..num_players {
            // Create Player
            let id = PlayerID(id);
            state
                .player_states
                .insert(id.clone(), PlayerState::new(STARTING_LIVES));
            driver
                .players
                .insert(id.clone(), Box::new(DumbPlayer::new(id.clone())));
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

    pub fn play(&mut self) {
        // TODO - Establish turn order -> Roll for it? Then clockwise?
        // Find a way not to do this twice
        let turn_order = self.driver.players.keys().cloned().collect();
	
	self.shuffle();
        self.deal(&turn_order);

        // Start Game Loop
        while !self.game_over(&turn_order) {
            // Need to check game over everytime state changes. --> Sad
	    let active_players = &self.active_players(&turn_order);
            for active_id in active_players {
                // Check if player is alive, otherwise pass on their turn
                let player = self.driver.players.get(active_id).unwrap();
                let action = player.choose_action(&self.state);

                // Allow for actions to be blocked
                for blocker_id in active_players {
                    if action.blockable(blocker_id) {
                        let blocker = self.driver.players.get(blocker_id).unwrap();
                        if let Some(blocking_action) = blocker.will_block(&self.state, &active_id, &action) {
                            println!("Blocker wants to block! TBI {:?}", blocking_action);
                        }
                    }
                }

                // Allow for challenging
		if action.challengable() {
                    for challenger_id in active_players {
			// Can't challenge yourself
			if challenger_id == active_id {
			    continue;
			}
                        let challenger = self.driver.players.get(challenger_id).unwrap();
                        if challenger.will_challenge(&self.state, &active_id, &action) {
			    // process the challenge
			    self.process_challenge(active_id.clone(), challenger_id.clone());
			}
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

    fn process_challenge(&mut self, actor_id: PlayerID, challenger_id: PlayerID) {
	
    }

    fn present_game_results(&self) {
        if self.state.active_players.len() != 1 {
            println!("Uh oh... a lot of people won?");
        } else {
            println!("Player {:?} won!", self.state.active_players[0]);
        }
    }

    fn process_action(&mut self, action: &Action, actor: &PlayerID) {
        println!("Player {:?} chose action {:#?}", actor, action);
        match action {
            Action::Income => {
                let mut player = self.state.player_states.get_mut(&actor).unwrap();
                player.num_coins += 1;
            }
            Action::Coup(target) => {
                let mut attacker = self.state.player_states.get_mut(&actor).unwrap();
                attacker.num_coins -= 7;

                let mut victim = self.driver.players.get_mut(&target).unwrap();
		// TODO This is going to be it's own function likely for recursions sake
		let to_discard = victim.choose_card_to_lose(&self.state);
                let discarded = victim.discard(to_discard).unwrap();
                println!("Target Player {:?} discarded {:#?}", &target, discarded);
                let mut victim_state = self.state.player_states.get_mut(&target).unwrap();
                victim_state.lost_lives.push(discarded);
                victim_state.num_lives -= 1;
            }
            _ => panic!("Unimplemented..."),
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
    let num_players = 3;
    let mut game = Game::new(game_identities, num_players);
    game.play();

    // let player = HumanPlayer::new(PlayerID(1));
    
    // Game Driver code
    // Create Players

    Ok(())
}
