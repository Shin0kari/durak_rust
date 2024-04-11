use std::{collections::HashMap, time::Instant};

use rand::Rng;

#[derive(Debug, Clone)]
struct DeckCards {
    cards: HashMap<u8, Card>,
    // нужно для присваивания id игрокам
    id: u8,
}

type Id = u8;

struct Cards {
    cards: HashMap<u8, Card>,
    // 1 нападение, 2 защита, 3, 4 и тд по ре
    player_state: u8,
}

#[derive(Debug, Clone)]
struct Card {
    suit: char,
    trump: bool,
}

fn main() {
    let now = Instant::now();
    // hows the size of a deck of cards
    let vec_len = 36;
    let mut rand_vec_cards: Vec<u8> = vec![0; vec_len];
    // generating a random deck of cards
    rand_vec_cards = gen_rand_deck(rand_vec_cards);
    // println!("Rand_deck: {:?}", rand_vec_cards);

    // let suits = "♣♠♥♦".chars();
    let trump_volume = *rand_vec_cards.first().unwrap(); // last volume in array
    rand_vec_cards = rand_vec_cards[1..].to_vec(); // removing a trump card from the queue
    rand_vec_cards.push(trump_volume); // adding a trump card to the queue

    let mut filled_card_hash = filled_card_hash(&rand_vec_cards, vec_len, trump_volume);

    let bot1_start_deck = gen_deck_in_hand(&filled_card_hash);
    filled_card_hash = remove_hand_cards(filled_card_hash, &bot1_start_deck);

    let bot2_start_deck = gen_deck_in_hand(&filled_card_hash);
    filled_card_hash = remove_hand_cards(filled_card_hash, &bot2_start_deck);

    let elapsed_time = now.elapsed();

    println!(
        "The array was generated in {} micros",
        elapsed_time.as_micros()
    );
    // println!("Bot_start_deck: {:?}", bot1_start_deck);
    // println!("Bot_start_deck: {:?}", bot2_start_deck);
    // println!("New Rand_deck: {:?}", filled_card_hash);
}

struct GameData {
    who_attacker: Id,
    attacker_deck: AttackerDeck,
    defending_deck: DefendingDeck,
    deck_of_cards: DeckCards,
    players_cards: HashMap<Id, Cards>,
}

// нападающий
struct AttackerDeck {
    attacking_cards: HashMap<u8, Card>,
}

// отбивающийся
struct DefendingDeck {
    pairs_beaten_cards: HashMap<u8, u8>,
    defending_cards: HashMap<u8, Card>,
}

fn who_is_first() {}
fn play_card() {}

fn throw_card() {}

fn take_all_card() {}

fn beat_card() {}

fn take_cards_from_deck() {}

fn remove_hand_cards(mut filled_card_hash: DeckCards, card_hash: &Cards) -> DeckCards {
    card_hash
        .cards
        .clone()
        .into_iter()
        .for_each(|(card_volume, _)| {
            if filled_card_hash.cards.contains_key(&card_volume) {
                filled_card_hash.cards.remove(&card_volume);
            }
        });

    filled_card_hash
}

fn filled_card_hash(rand_vec: &[u8], vec_len: usize, trump_volume: u8) -> DeckCards {
    let mut filled_card_hash = DeckCards {
        cards: HashMap::new(),
        id: 0,
    };
    rand_vec.iter().for_each(|card| {
        filled_card_hash
            .cards
            .insert(*card, filling_in_card_data(*card, vec_len, trump_volume));
    });
    filled_card_hash
}

fn filling_in_card_data(card_volume: u8, trump_card_location: usize, trump_volume: u8) -> Card {
    // println!(
    //     "Card volume: {:?}, location: {:?}, trump_volume: {:?}",
    //     card_volume, trump_card_location, trump_volume
    // );
    let div_from_volume = match trump_card_location {
        24 => 6,
        36 => 9,
        52 => 13,
        54 => 15,
        _ => panic!("A PANIC!"),
    };
    // Checking if the card is a trump card
    let trump_data = (((card_volume - 1) / div_from_volume) as i32)
        == ((trump_volume - 1) / div_from_volume) as i32;
    match (card_volume - 1) / div_from_volume {
        0 => Card {
            suit: 'c',
            trump: trump_data,
        },
        1 => Card {
            suit: 's',
            trump: trump_data,
        },
        2 => Card {
            suit: 'h',
            trump: trump_data,
        },
        3 => Card {
            suit: 'd',
            trump: trump_data,
        },
        _ => panic!("B PANIC!"),
    }
}

fn gen_deck_in_hand(filled_card_hash: &DeckCards) -> Cards {
    let mut card_in_hand = Cards {
        cards: HashMap::new(),
        player_state: 0,
    };
    let del_data = if (filled_card_hash.cards.len() - 7) > 0 {
        7
    } else {
        filled_card_hash.cards.len()
    };
    filled_card_hash
        .cards
        .iter()
        .take(del_data)
        .for_each(|(volume, card_info)| {
            card_in_hand.cards.insert(*volume, card_info.clone());
        });
    card_in_hand
}

// это можно оптимизировать, но ~120 микро секунд для 54 карт терпимо
fn gen_rand_deck(mut rand_vec: Vec<u8>) -> Vec<u8> {
    let vec_len = rand_vec.len();
    for i in 1..=vec_len {
        let mut rng = rand::thread_rng();
        let mut rand_int: usize = rng.gen_range(0..vec_len);
        while rand_vec[rand_int] != 0 {
            rand_int = rng.gen_range(0..vec_len);
        }
        rand_vec[rand_int] = i as u8;
    }
    rand_vec
}
