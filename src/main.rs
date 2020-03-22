use anyhow::Result;
use enumset::{EnumSet, EnumSetType};
// use common_macros::hash_set;
use std::collections::{HashMap, HashSet};
use std::mem;
// use std::io;

const MAX_CARDS: u8 = 2;
const STARTING_LIVES: u8 = 2;

#[derive(Debug)]
enum Action {
    Income,
    ForeignAid,
    Tax,
    Assassinate(PlayerID),
    Coup(PlayerID),
    Exchange,
}

// Game change turns
// Every Player
#[derive(Debug, Clone)]
struct PlayerID(u8);

// Holds internal state about the current game
#[derive(Debug)]
struct GameState {
    num_cards: u8,
    player_states: HashMap<PlayerID, PlayerState>,
    // history -> Vec of Turns?
}

struct GameDriver {
    // Holds the autonomous players
    field: GameField,
    // Need to be stored?
    // Do I need to make static / store playerID to player map
    players: HashMap<PlayerID, Box<Player>>,
}

struct Game {
    driver: GameDriver,
    state: GameState,
}

impl Game {
    // Will need to decide on how to assign players / who is playing
    fn new(identities: HashSet<Identity>, num_players: u8) -> Self {
        // TODO -> Yuck panic on 0?
        let num_cards = match num_players {
            1..=4 => 3,
            _ => 4,
        };

        let mut driver = GameDriver::new(identities, num_cards);
        let mut state = GameState::new(num_cards);
        for id in 0..num_players {
            // Create Player
            let id = PlayerID(id);
            state
                .player_states
                .insert(id, PlayerState::new(STARTING_LIVES));
            driver.players.insert(id, DumbPlayer::new());
        }

        Self { driver, state }
    }
}

impl GameState {
    fn new(num_cards: u8) -> Self {
        let player_states = HashMap::new();
        Self {
            num_cards,
            player_states,
        }
    }
}

impl GameDriver {
    fn new(identities: HashSet<Identity>, num_cards: u8) -> Self {
        let field = GameField::new(identities, num_cards);
        let players = HashMap::new();
        Self { players }
    }
}

// This is public information about a player
struct PlayerState {
    lost_lives: Vec<Identity>,
    num_coins: u8,
    num_lives: u8,
}

impl PlayerState {
    fn new(num_lives: u8) -> Self {
        let lost_lives = Vec::new();
        Self {
            num_lives,
            lost_lives,
        }
    }
}

trait Player {
    fn take_action(&self, state: &GameState) -> Action;
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    // How do I show this?
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool;
    fn take_card(&self, state: &GameState, card: Identity);
    fn who_am_i(&self) -> &PlayerID;

    // Start built-in functions
    fn count_coins(&self, state: &GameState) -> u8 {
        let player_state = state.player_states.get(self.who_am_i()).unwrap();
        player_state.num_coins.clone()
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
    fn new(identities: HashSet<Identity>, num_cards: u8) -> Self {
        let deck = Vec::new();
        for identity in identities {
            for _ in 0..num_cards {
                deck.insert(identity.clone())
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
        let num_coins = 0;
        DumbPlayer {
            id,
            hand,
            num_coins,
        }
    }
}

impl Player for DumbPlayer {
    fn take_action(&self, state: &GameState) -> Action {
        Action::Income
    }
    fn will_challenge(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        false
    }
    // How do I show this?
    fn will_block(&self, state: &GameState, player_id: &PlayerID, action: &Action) -> bool {
        false
    }
    fn take_card(&self, state: &GameState, card: Identity) {
        // Yeah?
        if self.hand.len() >= MAX_CARDS {
            mem::replace(self.hand[0], card);
        } else {
            self.hand.push(card);
        }
    }
    fn who_am_i(&self) -> PlayerID {
        self.id.clone()
    }
}

fn main() -> Result<()> {
    let game_identities = Identity::Ambassador
        | Identity::Assassin
        | Identity::Contessa
        | Identity::Captain
        | Identity::Duke;
    let num_players = 3;
    let game = Game::new(game_identities, num_players);
    // Game Driver code
    // Create Players

    Ok(())
}
