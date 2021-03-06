mod action;
mod logger;
mod player;
mod prompter;

use action::Action;
use anyhow::Result;
use structopt::clap::arg_enum;
use enumset::{EnumSet, EnumSetType};
use logger::local_logger::LocalLogger;
use logger::traits::Logger;
use player::dumb_player::DumbPlayer;
use player::human_player::HumanPlayer;
use player::random_player::RandomPlayer;
use player::traits::Player;
use prompter::{LocalPrompter, Prompter};
use rand::seq::SliceRandom;
use std::cmp::min;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display};
use structopt::StructOpt;

use std::{thread, time};


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

pub enum LoggerType {
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

/// Main struct for the game
pub struct Game {
    driver: GameDriver,
    state: GameState,
    logger: Box<dyn Logger>,
    interactive: bool
}

impl Game {
    // Will need to decide on how to assign players / who is playing
    pub fn new(
        identities: EnumSet<Identity>,
        players: Vec<PlayerConfig>,
        logger_type: LoggerType,
    ) -> Self {
        let logger = match logger_type {
            LoggerType::Local => Box::new(LocalLogger {}) as Box<dyn Logger>,
        };

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
	let mut interactive = false;
        for entry in players {
	    let id = PlayerID(player_id.clone());
            player_id = player_id + 1;
	    let mut name = entry.player_name;
            // Create Player
            let player = match entry.player_type {
                PlayerType::DumbCPU => Box::new(DumbPlayer::new(id.clone())) as Box<dyn Player>,
                PlayerType::RandomCPU => Box::new(RandomPlayer::new(id.clone())) as Box<dyn Player>,
                PlayerType::Local => {
		    // Existence of local player makes game interactive
		    interactive = true;
		    // Overrwrite name for local player
		    let mut player_prompter = LocalPrompter::new();
		    println!("Creating a new local player!\nPlease enter your name:");
		    name = player_prompter.prompt_player(None).unwrap();
		    player_prompter.set_name(name.clone());
		    Box::new(HumanPlayer::new(id.clone(), player_prompter)) as Box<dyn Player>
		}
            };
            state.player_states.insert(
                id.clone(),
                PlayerState::new(name, STARTING_LIVES),
            );

            driver.players.insert(id.clone(), player);
        }
	state.update_turn_order(driver.players.keys().cloned().collect());

        Self {
            driver,
            state,
            logger,
	    interactive,
        }
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

    fn wait(&self) {
	let second = time::Duration::from_millis(1000);
	let now = time::Instant::now();
	thread::sleep(second);
    }

    fn wait_if_interactive(&self) {
	if self.interactive {
	    self.wait();
	}
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

    pub fn setup(&mut self) {
	// TODO - Establish turn order -> Roll for it? Then clockwise?
        // Find a way not to do this twice
        let turn_order = self.driver.players.keys().cloned().collect();
        self.shuffle();
        self.deal(&turn_order);
        self.update_active_players(&turn_order);

	// TODO --> Remove / detect if interactive mode.
	// Could check for local players?
	println!("Let the game begin!");
	print!("The turn order is as follows: ");
	for player in &turn_order {
	    print!("{} ", self.state.get_player_name(player));
	}
	println!("");
    }

    pub fn play(&mut self) {
	self.setup();
	let turn_order = self.driver.players.keys().cloned().collect();
        // Start Game Loop
        while !self.game_over(&turn_order) {
            // Need to check game over everytime state changes. --> Sad
            let active_players = &self.active_players(&turn_order);
            for active_id in active_players {
		self.logger.log(format!(
		    "{}'s turn!", self.state.get_player_name(active_id))
		);
		self.wait_if_interactive();
                let player = self.driver.players.get(active_id).unwrap();

                // Enforce Required Coup
                let action = if player.count_coins(&self.state) < REQUIRE_COUP_COINS {
                    player.choose_action(&self.state)
                } else {
                    Action::Coup(player.choose_forced_coup(&self.state))
                };

                self.logger.log(
                    format!(
                        "{} chose action {}",
                        self.get_player_name(active_id),
                        LocalPrompter::display_action(&self.state, action.clone())
                    )
                    .to_string(),
                );

                // TODO -> This is overly complex, and does not allow things to be challenged if someone wants
                // to block. Would like to be able to choose these at the same time
                let mut block_was_challenged = false;

                // Allow for actions to be blocked
                for blocker_id in active_players {
		    // Don't block yourself
		    if blocker_id == active_id {
			continue;
		    }
		    
                    if action.blockable(blocker_id).is_some() {
                        let blocker = self.driver.players.get(blocker_id).unwrap();
                        // actor steal from blocker
                        if let Some(blocking_action) =
                            blocker.will_block(&self.state, &active_id, &action)
                        {
			    self.wait_if_interactive();
                            // blocker block
                            self.logger.log(format!(
                                "{} is blocking {}'s {} with {}",
                                self.get_player_name(blocker_id),
                                self.get_player_name(active_id),
                                LocalPrompter::display_action(&self.state, action.clone()),
                                LocalPrompter::display_action(&self.state, blocking_action.clone()),
                            ));
                            if let Some(challenge) =
                                self.check_for_challenges(blocker_id, &turn_order, &blocking_action)
                            {
                                // actor challenge your block
                                block_was_challenged = true;
                                if !self.process_challenge(&challenge) {
                                    //Challenge was unsuccessful, action is blocked, turn is over
                                    continue;
                                } // Challenge was successful, block is invalidated, action goes through
                            } else {
                                // Block was unchallenged, do not allow anyone to challenge
                                continue;
                            }
                        }
                    }
                }

                // TODO --> Do not allow for challenging of action if already blocked
                if action.challengable() && !block_was_challenged {
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
	self.wait_if_interactive();
        self.present_game_results();
    }

    fn process_challenge(&mut self, challenge: &Challenge) -> bool {
        let actor_id = &challenge.actor_id;
        let challenger_id = &challenge.challenger_id;
        let action = &challenge.action;
	
	self.wait_if_interactive();
        self.logger.log(format!(
            "{} is challenging {}'s {}",
            self.get_player_name(challenger_id),
            self.get_player_name(actor_id),
            LocalPrompter::display_action(&self.state, action.clone())
        ));

        let mut winner_id = actor_id;
        let mut loser_id = actor_id;

        let actor = self.driver.players.get_mut(actor_id).unwrap();
        if actor.can_do_action(action) {
            loser_id = challenger_id;
        } else {
            winner_id = challenger_id;
        }
	
	self.wait_if_interactive();
        self.logger.log(format!(
            "{} lost the challenge",
            self.get_player_name(loser_id)
        ));
	self.wait_if_interactive();
        self.kill_player(loser_id);
        // let winner = self.driver.players.get_mut(winner_id).unwrap();
        // TODO - Give winner a card from the deck
        return challenger_id == winner_id;
    }

    fn present_game_results(&self) {
        if self.state.active_players.len() != 1 {
            self.logger
                .log(format!("Uh oh... a lot of people won?").to_string());
        } else {
            self.logger.log(
                format!(
                    "{} won!",
                    self.get_player_name(&self.state.active_players[0])
                )
                .to_string(),
            );
        }
    }

    fn kill_player(&mut self, player_id: &PlayerID) {
        let mut victim = self.driver.players.get_mut(player_id).unwrap();
	let num_lives_left = self.state.player_states.get(player_id).unwrap().num_lives;
	if num_lives_left == 0 {
	    self.logger.log(
		format!("Tried to kill {} but they have no lives left!",
			self.get_player_name(player_id)
		)
	    );
	    return;
	}
        let to_discard = victim.choose_card_to_lose(&self.state);
        let discarded = victim.discard(to_discard).unwrap();
        self.logger.log(format!(
            "{} discarded {:#?}",
	    self.get_player_name(player_id),
            discarded
        ));
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
                self.logger
                    .log(format!("Unknown action... Moving on {:?}", action).to_string());
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

    fn get_player_name(&self, player_id: &PlayerID) -> String {
        self.state.get_player_name(&player_id)
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	// Print Player States
	for player in &self.turn_order {
	    let player_state = self.player_states.get(player);
	    match player_state {
		Some(player_state) => {
		    println!("{}", player_state);
		},
		None => {
		    panic!("Turn order and player states out of sync")
		}
	    }
	}
	Ok(())
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
    fn get_player_name(&self, player_id: &PlayerID) -> String {
        self.player_states.get(&player_id).unwrap().get_name()
    }
    fn update_turn_order(&mut self, turn_order: Vec<PlayerID>) {
	self.turn_order = turn_order;
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
    player_name: String,
    num_coins: u8,
    // TODO --> Make this sync with deck somehow?
    num_lives: u8,
}

impl fmt::Display for PlayerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "{}:\n", self.player_name);
	write!(f, "\tLives: {}\n", self.num_lives);
	write!(f, "\tCoins: {}\n", self.num_coins);
	if self.lost_lives.len() > 0 {
	    write!(f, "\tLost Identities: ");
	    for life in &self.lost_lives {
		write!(f, "{:?} ", life);
	    }
	    write!(f, "\n");
	}
	Ok(())
    }
 
}


impl PlayerState {
    pub fn new(player_name: String, num_lives: u8) -> Self {
        let lost_lives = Vec::new();
        Self {
            player_name,
            num_coins: STARTING_COINS,
            num_lives,
            lost_lives,
        }
    }
    pub fn is_alive(&self) -> bool {
        self.num_lives > 0
    }

    pub fn get_name(&self) -> String {
        self.player_name.clone()
    }
}

// Can be used for cards as well?
arg_enum! {
#[derive(Debug,  EnumSetType)]
pub enum Identity {
    Ambassador,
    Assassin,
    Contessa,
    Captain,
    // Inquisitor,
    Duke,
}
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

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

// TODO Max cards needs to sit with this
const STARTING_CARDS: u8 = 2;
const STARTING_COINS: u8 = 2;
const STARTING_LIVES: u8 = 2;
const REQUIRE_COUP_COINS: u8 = 10;

#[derive(StructOpt, Debug)]
#[structopt(name = "Coup Simulator CLI", setting = structopt::clap::AppSettings::ColoredHelp)]
struct GameConfig {
    /// The Identitites to use for this game
    #[structopt(long, possible_values = &Identity::variants(), value_delimiter = ",", default_value = "Ambassador,Assassin,Contessa,Captain,Duke")] //default_value = "Ambassador Assassin Contessa Captain Duke")]
    game_identities: Vec<Identity>,
    /// The number of cards each player begins the game with
    #[structopt(long, default_value = "2")]
    starting_cards: u8,
    /// The number of coins each player begins the game with
    #[structopt(long, default_value = "2")]
    starting_coins: u8,
    /// The number of lives each player begins the game with
    #[structopt(long, default_value = "2")]
    starting_lives: u8,
    /// The limit on the amount of coins each player starts with
    #[structopt(long, default_value = "10")]
    required_coup_coins: u8,
    /// The number of local players in this simulation
    #[structopt(long, default_value = "1")]
    num_local_players: u8,
    /// The names of the Random CPUS in this simulation
    #[structopt(long, value_delimiter = ",", default_value = "Porter,Miela")]
    random_cpus: Vec<String>,
    /// The names of the Dumb CPUS in this simulation
    #[structopt(long, value_delimiter = ",", default_value = "Don")]
    dumb_cpus: Vec<String>
    

}

fn main() -> Result<()> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info));

    let config = GameConfig::from_args();
    
    let game_identities : EnumSet<Identity> = config.game_identities.into_iter().collect();
    let mut players = Vec::new();
    for cpu in &config.dumb_cpus {
	players.push(PlayerConfig::new(PlayerType::DumbCPU, cpu.clone()));
    }

    for cpu in &config.random_cpus {
	players.push(PlayerConfig::new(PlayerType::RandomCPU, cpu.clone()));
    }

    for player in 0..config.num_local_players {
	// TODO make name optional / not needed for local player config
	players.push(PlayerConfig::new(PlayerType::Local, "".to_string()));
    }
    let mut game = Game::new(game_identities, players, LoggerType::Local);
    game.play();
    // game.setup();
    // human_player.choose_action(&game.state);
    Ok(())
}
