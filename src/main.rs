use anyhow::Result;
use enumset::{EnumSet, EnumSetType};
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem;
// use std::io;

const MAX_CARDS: u8 = 2;
const STARTING_COINS: u8 = 2;
const STARTING_LIVES: u8 = 1;

#[derive(Debug)]
enum Action {
    Income,
    ForeignAid,
    Tax,
    Assassinate(PlayerID),
    Coup(PlayerID),
    Exchange,
    // BlockForeignAid
    // BlockAssassination
}

impl Action {
    // Dependent on id of target
    fn blockable(&self, id: &PlayerID) -> bool {
        match self {
            Action::Income | Action::Coup(..) | Action::Exchange | Action::Tax => false,
            // Can only block if they assassinate you
            Action::Assassinate(target) => target == id,
            Action::ForeignAid => true,
        }
    }
    // Defines if an action is challengable
    fn challengable(&self) -> bool {
        match self {
            Action::Income | Action::ForeignAid | Action::Coup(..) => false,
            _ => true,
        }
    }
}

// Game change turns
// Every Player
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PlayerID(u8);

// Holds internal state about the current game
#[derive(Debug)]
struct GameState {
    // TODO = Convenience Cache consider removing
    active_players: Vec<PlayerID>,
    num_cards: u8,
    player_states: HashMap<PlayerID, PlayerState>,
    turn_order: Vec<PlayerID>,
    // history -> Vec of Turns?
}

struct GameDriver {
    field: GameField,
    // Need to be stored?
    // Do I need to make static / store playerID to player map
    // Holds the autonomous players
    players: HashMap<PlayerID, Box<dyn Player>>,
}

struct Game {
    driver: GameDriver,
    state: GameState,
}

impl Game {
    // Will need to decide on how to assign players / who is playing
    fn new(identities: EnumSet<Identity>, num_players: u8) -> Self {
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

    // Should this be in driver? Should driver be flattened to game?
    fn deal(&mut self, player_order: &Vec<PlayerID>) {
        // Get rand iterator over deck
        let mut rng = rand::thread_rng();

        for id in player_order {
            // TODO Get random iterator over cards and deal
            let card = Identity::Ambassador; // self.driver.field.deck.into_iter().cloned().choose(&mut rng).clone();
            self.driver
                .players
                .get_mut(&id)
                .unwrap()
                .take_card(&self.state, card.clone());
        }
    }

    fn play(&mut self) {
        // Give everyone one turn
        // TODO - Establish turn order -> Roll for it? Then clockwise?
        // Find a way not to do this twice
        let turn_order = self.driver.players.keys().cloned().collect();
        self.deal(&turn_order);

        // Start Game Loop
        while !self.game_over(&turn_order) {
            // Need to check game over everytime state changes.

            let active_players = &self.active_players(&turn_order);

            for active_id in active_players {
                // Check if player is alive, otherwise pass on their turn
                let player = self.driver.players.get(active_id).unwrap();
                let action = player.choose_action(&self.state);

                // Allow for actions to be blocked
                for blocker_id in active_players {
                    if action.blockable(blocker_id) {
                        let blocker = self.driver.players.get(blocker_id).unwrap();
                        if blocker.will_block(&self.state, &active_id, &action) {
                            println!("Blocker wants to block! TBI");
                        }
                    }
                }

                // Allow for challenging
                for challenger_id in active_players {
                    if action.challengable() {
                        let challenger = self.driver.players.get(challenger_id).unwrap();
                        if challenger.will_challenge(&self.state, &active_id, &action) {
                            println!("challenger wants to challenge! TBI");
                        }
                    }
                }

                self.process_action(&action, active_id);
                // Could change every turn... Need a check to make sure I'm still active... Need to do this everywhere :(
                self.state.active_players = self.active_players(&turn_order);

                // TODO --> This makes me very sad
                if self.game_over(&turn_order) {
                    break;
                }
            }
        }

        self.present_game_results();
    }

    fn present_game_results(&self) {
        if self.state.active_players.len() != 1 {
            println!("Uh oh...");
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
                let discarded = victim.discard_identity(&self.state);
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
struct PlayerState {
    lost_lives: Vec<Identity>,
    num_coins: u8,
    num_lives: u8,
}

impl PlayerState {
    fn new(num_lives: u8) -> Self {
        let lost_lives = Vec::new();
        Self {
            num_coins: STARTING_COINS,
            num_lives,
            lost_lives,
        }
    }
    fn is_alive(&self) -> bool {
        self.num_lives > 0
    }
}

trait Player {
    fn choose_action(&self, state: &GameState) -> Action;
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> usize;

    // Utility functions on player state
    fn get_hand(&self) -> Vec<Identity>;
    fn set_hand(&mut self, hand: Vec<Identity>);
    fn who_am_i(&self) -> &PlayerID;
    fn discard_identity(&mut self, state: &GameState) -> Identity;

    // Start built-in functions
    fn replace_card(&mut self, to_replace: usize, card: Identity) {
	let mut hand = self.get_hand();
	mem::replace(&mut hand[to_replace], card.clone());
	self.set_hand(hand);
    }
    
    fn hand_full(&self) -> bool {
	self.get_hand().len()>= MAX_CARDS.try_into().unwrap()
    }
    
    fn count_coins(&self, state: &GameState) -> u8 {
        let player_state = state.player_states.get(self.who_am_i()).unwrap();
        player_state.num_coins.clone()
    }

    fn take_card(&mut self, state: &GameState, card: Identity) {
        // Yeah this is silly
        if self.hand_full() {
            let to_replace = self.choose_card_to_replace(state, &card);
            self.replace_card(to_replace, card);
        } else {
	    let mut hand = self.get_hand();
	    hand.push(card);
            self.set_hand(hand);
        }
    }
}

// Can be used for cards as well?
#[derive(Debug, EnumSetType)]
enum Identity {
    Ambassador,
    Assassin,
    Contessa,
    Captain,
    // Inquisitor,
    Duke,
}

struct GameField {
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

struct DumbPlayer {
    // Not necessarily two?
    id: PlayerID,
    hand: Vec<Identity>,
}

impl DumbPlayer {
    fn new(id: PlayerID) -> Self {
        let hand = Vec::new();
        DumbPlayer { id, hand }
    }
}

impl Player for DumbPlayer {
    fn choose_action(&self, state: &GameState) -> Action {
        if self.count_coins(state) < 10 {
            Action::Income
        } else {
            // Need to choose player? for coup?
            // choose player after you in order
            // Lol Jank
            for player_id in &state.active_players {
                if player_id != self.who_am_i() {
                    return Action::Coup(player_id.clone());
                }
            }
            panic!("No other players to coup!");
        }
    }
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        false
    }
    // How do I show this?
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        false
    }
    // Index in hand to replace
    fn choose_card_to_replace(&self, state: &GameState, card: &Identity) -> usize {
        0
    }
    
    fn get_hand(&self) -> Vec<Identity> {
	self.hand.iter().cloned().collect()
    }

    fn set_hand(&mut self, hand: Vec<Identity>) {
	self.hand = hand;
    }

    fn who_am_i(&self) -> &PlayerID {
        &self.id
    }

    // TODO Deal with errors better
    fn discard_identity(&mut self, state: &GameState) -> Identity {
        let num_cards = self.hand.len();
        if num_cards > 0 {
            // TODO Refactor as util function remove from hand --> Can I make
	    // all traits have hand?
            let remove_index = num_cards - 1;
            let removed = self.hand.remove(remove_index);
            return removed;
        } else {
            panic!("Oh God!");
        }
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
    // Game Driver code
    // Create Players

    Ok(())
}
