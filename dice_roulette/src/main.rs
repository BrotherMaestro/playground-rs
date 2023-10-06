///////////////////////////////////////////////////////////////////////////////
//    Copyright 2023 Hayden Mark Sip
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
///////////////////////////////////////////////////////////////////////////////

//! Project Motivation:
//! Mini automated dice rolling game.
//! Purpose of this project was to become familiar with rusts async channels.

//! DICE ROULETTE
//! Create a single player probability 'game'.
//! A player starts with X dice. Each dice roll adds to the total score.
//! The objective is to get the highest score!
//! Each even number rolled, subtracts the number of dice in your 'hand'.
//! Each odd number rolled, adds to the number of dice in your 'hand'.
//! When you have less than 1 die in your hand, the game ends and the final score is printed!

use std::{
    collections::BTreeSet,
    io,
    sync::mpsc::{self, Sender},
    thread::{self},
};

use rand::Rng;

enum GameUpdate {
    Message(String),
    Score(i64),
}

struct Dice {
    value: i8,
}

impl Dice {
    // Returns a rolled dice with an integer value from 1 to the number of sides
    fn new(number_of_sides: i8) -> Self {
        // random modulo sides is equivalent to 0..(number_of_sides - 1)
        // so correct with +1
        let value = rand::thread_rng().gen_range(0..number_of_sides) + 1;
        Self { value }
    }
}

#[derive(Clone, Copy)]
struct DiceHand {
    number_of_dice: i32,
    number_of_sides: i8,
}

#[derive(Debug, PartialEq)]
struct DiceRollTotal {
    even: i64,
    odd: i64,
}

impl DiceRollTotal {
    /// difference = odd - even
    /// Parity is the property of being odd or even
    fn parity_difference(&self) -> i64 {
        self.odd - self.even
    }

    fn sum(&self) -> i64 {
        self.odd + self.even
    }
}

const SCORE_FILE_PATH: &str = "scores.msgpack";

fn main() {
    // Track best scores in local file. Will save state after each game
    let mut scores = read_state_from_file(SCORE_FILE_PATH);

    let starting_hand = DiceHand {
        number_of_dice: 12,
        number_of_sides: 7,
    };

    // Main game loop
    loop {
        // For each iteration of the game,
        // Start with menu and user input
        print_menu();
        let user_input = get_user_input();
        match user_input.as_str() {
            "start" => {
                // Start a new game
                println!("Starting New Game...");
                println!();

                let score = game_loop(starting_hand);
                println!("Game Over!");
                if score > scores.last().copied().unwrap_or_default() {
                    println!("New high score: {}", score);
                } else {
                    println!("Total score: {}", score);
                }
                println!();

                // Update scores (and save top 10 scores in file)
                scores.insert(score);
                let score_slice: Vec<_> = scores.iter().rev().take(10).copied().collect();
                save_state_to_file(SCORE_FILE_PATH, &score_slice);
            }
            "rules" => {
                print_rules(starting_hand);
            }
            "scores" => {
                // Print the first 10 scores (reversed for largest -> smallest)
                print_top_scores(scores.iter().rev(), 10);
            }
            "exit" => {
                // End the game
                println!();
                println!("Hope you enjoyed the game!");
                println!();
                break;
            }
            _ => {
                // Misunderstood input has no action!
                continue;
            }
        }
    }
}

fn print_menu() {
    println!("Dice Factions!");
    println!("Please enter an action from the follow list:");
    println!("Start, Rules, Scores, Exit:");
}

fn print_rules(starting_hand: DiceHand) {
    let number_of_dice = starting_hand.number_of_dice;
    let number_of_sides = starting_hand.number_of_sides;

    // Begin and end with a new line to form isolated paragraph
    println!();
    println!("Dice Factions Rules:");
    println!(concat!(
        "The objective of this probability game is to get the highest score! ",
        "To score, the player rolls the dice in their hand. ",
        "The cummulative value of the roll is added to your score. ",
        "The same roll is tallied into even and odd scores. ",
        "The even scores are then subtracted from the odd ",
        "and the result determines how many dice are in your next hand."
    ));
    println!("The player begins the game with {number_of_dice} {number_of_sides}-sided dice in their hand.");
    println!();
}

/// Prints the first how_many scores of the iterator.
/// Will print "no scores recorded" if the iterator is empty.
fn print_top_scores<'a, I>(scores: I, how_many: usize)
where
    I: IntoIterator<Item = &'a i64>,
{
    // Begin and end with a new line to form isolated paragraph
    println!();

    // Peekable iterator allows checking for empty (without consuming the first item)
    let mut peekable = scores.into_iter().peekable();
    if peekable.peek().is_none() {
        println!("No scores recorded");
    } else {
        println!("Top {how_many} Scores:");
        for (place, score) in peekable.take(how_many).enumerate() {
            println!("  {}. {score}", place + 1);
        }
    }
    println!();
}

fn save_state_to_file(file_path: &str, scores: &[i64]) {
    match std::fs::File::create(file_path) {
        Ok(mut file) => {
            if let Err(error) = rmp_serde::encode::write(&mut file, scores) {
                println!("Failed to write scores. {}", error);
            }
        }
        Err(error) => {
            println!("Failed to save scores. Existing with IO error: {}", error);
        }
    }
}

fn read_state_from_file(file_path: &str) -> BTreeSet<i64> {
    if let Ok(file) = std::fs::File::open(file_path) {
        if let Ok(values) = rmp_serde::decode::from_read::<std::fs::File, Vec<i64>>(file) {
            return values.into_iter().collect();
        }
    }
    BTreeSet::<i64>::new()
}

fn get_user_input() -> String {
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => {
            buffer = buffer.trim().to_lowercase();
        }
        Err(_) => {
            buffer.clear();
        }
    }
    buffer
}

/// Main game loop.
/// Rolls dice each round. Calculates the total score of the round.
/// Also determines how many dice are available for the next round.
/// The game loop ends once the dice held is less than zero.
fn game_loop(starting_hand: DiceHand) -> i64 {
    let mut total_score: i64 = 0;

    // Transmitter - Reciever structure
    // Hand thread ... needs reciever that sends the next number of dice to roll
    // Should accept a number of sides parameter (propagated from game loop input - not yet setup)
    // Result thread... should recieve DiceRollTotals from Hand thread.
    // Result thread should then send next dice count to Hand thread
    // Result thread should send next score back to this (main) thread.
    // Result is tallied here, when all channels are closed, result the result.
    let (tx_hand, rx_hand) = mpsc::channel();
    let (tx_total, rx_total) = mpsc::channel();
    let (tx_update, rx_update) = mpsc::channel();

    // Hand thread accept rx_hand (to get next hand values),
    // plus tx_total to send turn values to result thread.
    // Result thread should take both hand and score transmitters,
    // plus rx_total to reciece turn values from hand thread
    // Finally, rx_score remains in this thread to recieve turn scores from result thread.

    // Send starting value
    let number_of_dice = starting_hand.number_of_dice;
    println!("Rolling first hand of {number_of_dice} dice...");
    tx_hand.send(number_of_dice).unwrap();

    // THREADS
    // Manage the hand
    thread::spawn(move || {
        let number_of_sides = starting_hand.number_of_sides;
        for number_of_dice in rx_hand {
            let dice_totals = roll_dice(DiceHand {
                number_of_dice,
                number_of_sides,
            });
            tx_total.send(dice_totals).unwrap();
        }
    });

    // Manage the logic
    thread::spawn(move || {
        for dice_totals in rx_total {
            // Send the score to be processed
            tx_update
                .send(GameUpdate::Score(dice_totals.sum()))
                .unwrap();

            // Update player on even & odd scores:
            let even = dice_totals.even;
            let odd = dice_totals.odd;
            tx_update
                .send(GameUpdate::Message(format!(
                    "Rolled total scores of:\n\t{even} even\n\t{odd} odd\n\n"
                )))
                .unwrap();
            // Determine the next move in the game (game finished OR roll a new hand of X dice)
            match dice_totals.parity_difference().clamp(0, i32::MAX as i64) as i32 {
                0 => {
                    tx_update
                        .send(GameUpdate::Message(
                            concat!(
                                "The even score is greater than the odd total this round. ",
                                "No more dice left in your hand!\n"
                            )
                            .to_string(),
                        ))
                        .unwrap();
                    break;
                }
                next_hand => {
                    tx_update
                        .send(GameUpdate::Message(format!(
                            "Rolling next hand of {next_hand} dice...\n"
                        )))
                        .unwrap();
                    tx_hand.send(next_hand).unwrap();
                }
            }
        }
    });

    // Tally the score
    for update in rx_update {
        match update {
            GameUpdate::Score(score) => {
                total_score += score;
            }
            GameUpdate::Message(message) => {
                // leave a trailing space for the next message
                print!("{message} ");
            }
        }
    }
    // leave an empty space after the in-game messages!
    println!();

    total_score
}

/// Roll a hand of dice, and return the total score of (evens and odds)
fn roll_dice(hand: DiceHand) -> DiceRollTotal {
    // Create a channel to pass information back to this thread
    let (tx, rx) = mpsc::channel();

    // Spawn dice rolling threads
    spawn_die(tx, hand);

    // Collect dice rolls
    let mut odd_total = 0;
    let mut even_total = 0;
    for recieved_roll in rx {
        match recieved_roll.value {
            roll if (roll % 2) == 0 => {
                even_total += roll as i64;
            }
            roll => {
                odd_total += roll as i64;
            }
        }
    }

    DiceRollTotal {
        even: even_total,
        odd: odd_total,
    }
}

// Take ownership of transmitter (limiting its lifetime to the function)
// Start #threads equal to dice_to_roll
// Randomised roll restricted between 1 and number_of_sides
fn spawn_die(tx: Sender<Dice>, hand: DiceHand) {
    // spawn dice rolling threads
    for _ in 0..hand.number_of_dice {
        let tx_die = tx.clone();
        thread::spawn(move || {
            let dice = Dice::new(hand.number_of_sides);
            tx_die.send(dice).unwrap();

            // Later write thread safe logging code
            // if let Err(_) = tx_die.send(dice) {
            //     println!("Failed to send dice roll to reciever!");
            // }
        });
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Test that the number of spawned dice roll threads
    /// matches the number of dice given to roll
    #[test]
    fn number_of_dice_rolled() {
        // Restrict randomisation to always return a value '1'
        let dice_to_roll_first = 8;
        let dice_to_roll_second = 150_000;
        let number_of_sides = 1;

        assert_eq!(
            roll_dice(DiceHand {
                number_of_dice: dice_to_roll_first,
                number_of_sides
            }),
            DiceRollTotal {
                even: 0,
                odd: dice_to_roll_first as i64,
            }
        );

        // Perform a much more demanding roll
        // (Also checking result isn't a fluke)
        assert_eq!(
            roll_dice(DiceHand {
                number_of_dice: dice_to_roll_second,
                number_of_sides
            }),
            DiceRollTotal {
                even: 0,
                odd: dice_to_roll_second as i64,
            }
        );
    }

    /// Test even and odd counting works
    #[test]
    fn even_and_odd_split() {
        // Roll single die, to ensure the result is always between the 'number of sides'
        // Do this a significant amount of times
        const NUMBER_OF_ATTEMPTS: i32 = 1000;
        const NUMBER_OF_SIDES: i8 = 12;
        const STARTING_HAND: DiceHand = DiceHand {
            number_of_dice: 1,
            number_of_sides: NUMBER_OF_SIDES,
        };

        for _ in 0..NUMBER_OF_ATTEMPTS {
            let roll_total = roll_dice(STARTING_HAND);
            match roll_total {
                // Check an even number was rolled
                DiceRollTotal {
                    even: even @ 1..=12, // 2, 4, 6...
                    odd: 0,
                } if even % 2 == 0 => {
                    continue;
                }
                // Check an odd number was rolled
                DiceRollTotal {
                    even: 0,
                    odd: odd @ 1..=12, // 1, 3, 5...
                } if odd % 2 == 1 => {
                    continue;
                }
                // Fail case
                DiceRollTotal { even, odd } => {
                    unreachable!("(Even: {even}, Odd: {odd})")
                }
            }
        }
    }

    /// Test simple game begin & end logic. Check for expected scores!
    #[test]
    fn game_logic_test() {
        // Sides > 1 : otherwise causes an infinite game loop...
        // Start with a simple game, 1 die, 2 sides
        // Will exit as soon as a 2 is rolled, therefore 2 is the minimum score
        match game_loop(DiceHand {
            number_of_dice: 2,
            number_of_sides: 2,
        }) {
            x if x < 2 => {
                unreachable!("Result for 1 die of 2 sides must be at least 2");
            }
            _ => {
                // must be greater than or equal to 3
            }
        }

        // Repeat the experiment for more dice!
        // Minimum score of 8 (4 roll 1, 2 roll 2 => diff == 0)
        // Repeat this many times to estimate successful implementation
        for _ in 1..100 {
            match game_loop(DiceHand {
                number_of_dice: 6,
                number_of_sides: 2,
            }) {
                x if x < 8 => {
                    unreachable!("Result for 6 die of 2 sides must be at least 8");
                }
                _ => {
                    // must be greater than or equal to 3
                }
            }
        }
    }

    /// Test score file saving state
    #[test]
    fn score_state_test() {
        const FILE_PATH: &str = "test_scores.msgpack";

        // Stored scores in file will be read in ascending order 
        let scores = vec!(50,30,20,25,27,35);
        save_state_to_file(FILE_PATH, scores.as_slice());

        let buffer = read_state_from_file(FILE_PATH);
        let mut it = buffer.iter().copied();
        if let Some(value) = it.next() {
            assert_eq!(value, 20);
        } else {
            panic!("No values read from file");
        }
    }
}
