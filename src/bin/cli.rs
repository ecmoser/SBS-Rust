use sbs_rust::trie::{Trie, TrieNode};
use serde::Serialize;
use std::cmp::max;
use std::error::Error;
use std::io::{self, Write};
use reqwest::blocking::get;
use itertools::{EitherOrBoth, Itertools};

#[derive(Debug, Serialize)]
struct WordEntry {
    word: String,
    length: u32,
    frequency: u32,
    first_appeared: String,
}

fn get_used_words() -> Result<Trie, Box<dyn Error>> {
    let mut used_trie = Trie::new();
    let url = "https://ultrabee.org/list/word-first-desc-full.html";

    let words_entries = scrape_words(url)?;
    for entry in words_entries {
        used_trie.insert(entry.word.to_lowercase().as_ref());
    }

    Ok(used_trie)
}

fn get_all_words() -> Result<Trie, Box<dyn Error>> {
    let mut all_trie = Trie::new();
    let url = "https://gist.githubusercontent.com/deostroll/7693b6f3d48b44a89ee5f57bf750bd32/raw/426f564cf73b4c87d2b2c46ccded8a5b98658ce1/dictionary.txt";
    
    let body = get(url)?.text()?;
    let words: Vec<String> = body.lines().map(|line| line.trim().to_string()).filter(|line| line.len() > 3).collect();

    for word in words {
        all_trie.insert(word.to_lowercase().as_ref());
    }
    
    Ok(all_trie)
}

fn scrape_words(url: &str) -> Result<Vec<WordEntry>, Box<dyn Error>> {
    // Blocking client keeps this simple; swap for `reqwest::Client` + async
    // if you fold this into an async binary later.
    let body = reqwest::blocking::get(url)?.text()?;
    let document = scraper::Html::parse_document(&body);

    // The page is one <table> with a header row, so we just walk <tr><td>.
    let row_selector = scraper::Selector::parse("table tr").unwrap();
    let cell_selector = scraper::Selector::parse("td").unwrap();

    let mut entries = Vec::new();

    for row in document.select(&row_selector) {
        let cells: Vec<String> = row
            .select(&cell_selector)
            .map(|td| td.text().collect::<String>().trim().to_string())
            .collect();

        // Skip the header row (it uses <th>, so `cells` will be empty here).
        if cells.len() != 4 {
            continue;
        }

        let word = cells[0].clone();
        let length: u32 = cells[1].parse().unwrap_or(0);
        let frequency: u32 = cells[2].parse().unwrap_or(0);
        let first_appeared = cells[3].clone();

        entries.push(WordEntry {
            word,
            length,
            frequency,
            first_appeared,
        });
    }

    Ok(entries)
}

fn get_words_from_node(node: &TrieNode, letters: &Vec<char>, prefix: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();

    if node.is_end_of_word {
        words.push(prefix.to_string());
    }
    for letter in letters {
        let new_prefix = format!("{}{}", prefix, letter);
        match node.children.get(&letter) {
            Some(new_node) => {
                words.extend(get_words_from_node(new_node, letters, new_prefix.as_ref()))
            }
            None => continue,
        }
    }

    words.sort();
    words
}

fn solve_puzzle(
    puzzle: &String,
    used_words: &Trie,
    all_words: &Trie,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut used_matches: Vec<String> = Vec::new();
    let mut all_matches: Vec<String> = Vec::new();
    let mut pangrams: Vec<String> = Vec::new();

    let mut letters: Vec<char> = Vec::new();
    let mut center_letter: String = "".to_string();

    for char in puzzle.chars() {
        letters.push(char.to_ascii_lowercase());
        if char.is_uppercase() {
            center_letter = char.to_string().to_lowercase();
        }
    }

    let all_used_words = get_words_from_node(&used_words.root, &letters, "");
    for word in all_used_words {
        if word.contains(&center_letter) {
            let mut all_letters = true;
            for letter in letters.iter() {
                if !word.contains(&letter.to_string()) {
                    all_letters = false;
                    continue;
                }
            }
            if all_letters {
                pangrams.push(word.clone());
            } else {
                used_matches.push(word.clone());
            }
        }
    }

    for word in get_words_from_node(&all_words.root, &letters, "") {
        if word.contains(&center_letter) {
            let mut all_letters = true;
            for letter in letters.iter() {
                if !word.contains(&letter.to_string()) {
                    all_letters = false;
                    continue;
                }
            }
            if all_letters {
                if !pangrams.contains(&word) {
                    pangrams.push(word.clone());
                }
            } else {
                if !used_matches.contains(&word) {
                    all_matches.push(word.clone());
                }
            }
        }
    }

    (used_matches, all_matches, pangrams)
}

fn print_results(
    used_word_matches: Vec<String>,
    all_word_matches: Vec<String>,
    pangram_matches: Vec<String>,
) {
    let num_used = used_word_matches.len();
    let num_pangram = pangram_matches.len();
    let num_all = all_word_matches.len();
    
    let mut longest_used_len = 0;
    let mut longest_pangram_len = 0;
    let mut longest_all_len = 0;

    for word in used_word_matches.iter() {
        if word.len() > longest_used_len {
            longest_used_len = word.len();
        }
    }
    for word in pangram_matches.iter() {
        if word.len() > longest_pangram_len {
            longest_pangram_len = word.len();
        }
    }
    for word in all_word_matches.iter() {
        if word.len() > longest_all_len {
            longest_all_len = word.len();
        }
    }

    let longest_title = "Previously Used".to_string().len();
    let longest_word_len = max(max(longest_used_len, longest_pangram_len), longest_all_len);
    let space_size = max(longest_title, longest_word_len) + 1;

    let mut result = format!("\nTotal Words: {}\n", num_all + num_pangram + num_used);
    result += format!("Previously Used Words: {}\n", num_used + num_pangram).as_ref();
    result += "\n";
    result += "Previously Used";
    result += &" ".repeat(space_size - "Previously Used".len());
    result += "|Pangrams";
    result += &" ".repeat(space_size - "Pangrams".len());
    result += "|All Words";
    result += &" ".repeat(space_size - "All Words".len());
    result += "\n";
    result += &"-".repeat(space_size * 3 + 3);
    result += "\n";

    let zipped = used_word_matches.iter()
                    .zip_longest(&pangram_matches)
                    .zip_longest(&all_word_matches);

    for item in zipped {
        let (used, pangram, all): (String, String, String) = match item {
            EitherOrBoth::Both(inner, all) => {
                match inner {
                    EitherOrBoth::Both(used, pangram) => (used.to_string(), pangram.to_string(), all.to_string()),
                    EitherOrBoth::Left(used) => (used.to_string(), "".to_string(), all.to_string()),
                    EitherOrBoth::Right(pangram) => ("".to_string(), pangram.to_string(), all.to_string())
                }
            }
            EitherOrBoth::Left(inner) => {
                match inner {
                    EitherOrBoth::Both(used, pangram) => (used.to_string(), pangram.to_string(), "".to_string()),
                    EitherOrBoth::Left(used) => (used.to_string(), "".to_string(), "".to_string()),
                    EitherOrBoth::Right(pangram) => ("".to_string(), pangram.to_string(), "".to_string())
                }
            }
            EitherOrBoth::Right(all) => {
                ("".to_string(), "".to_string(), all.to_string())
            }
        };

        result += used.as_ref();
        result += &" ".repeat(space_size - used.len());
        result += "|";
        result += pangram.as_ref();
        result += &" ".repeat(space_size - pangram.len());
        result += "|";
        result += all.as_ref();
        result += &" ".repeat(space_size - all.len());
        result += "\n";
    }

    println!("{}", result);
}

fn main() -> Result<(), Box<dyn Error>> {
    let used_trie = get_used_words()?;
    let all_trie = get_all_words()?;

    //let all: get_all_words();
    loop {
        print!("What would you like to do? Solve(s)/Quit(q) ");
        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        let mut trimmed_input = input.trim().to_lowercase();

        while trimmed_input != "s".to_string() && trimmed_input != "q".to_string() {
            input = "".to_string();
            print!("Please enter 's' to solve or 'q' to quit: ");
            io::stdout().flush().expect("failed to flush stdout");

            io::stdin()
                .read_line(&mut input)
                .expect("failed to read line");

            trimmed_input = input.trim().to_lowercase();
        }

        if trimmed_input == "s".to_string() {
            print!("Enter the puzzle with the center letter capitalized: ");
            io::stdout().flush().expect("failed to flush stdout");

            let mut puzzle = String::new();
            io::stdin()
                .read_line(&mut puzzle)
                .expect("failed to read line");
            puzzle = puzzle.trim().to_string();

            let mut caps = 0;
            for letter in puzzle.chars() {
                if letter.is_uppercase() {
                    caps += 1;
                }
            }
            while puzzle.len() != 7 || caps != 1 {
                puzzle = "".to_string();
                print!("Please enter a 7-letter puzzle in the format of abcDefg: ");
                io::stdout().flush().expect("failed to flush stdout");
                io::stdin()
                    .read_line(&mut puzzle)
                    .expect("failed to read line");

                puzzle = puzzle.trim().to_string();
                caps = 0;

                for letter in puzzle.chars() {
                    if letter.is_uppercase() {
                        caps += 1;
                    }
                }
            }

            let used_word_matches: Vec<String>;
            let all_word_matches: Vec<String>;
            let pangram_matches: Vec<String>;

            (used_word_matches, all_word_matches, pangram_matches) =
                solve_puzzle(&puzzle, &used_trie, &all_trie);

            print_results(used_word_matches, all_word_matches, pangram_matches);
        } else if trimmed_input == "q".to_string() {
            println!("Quitting!");
            return Ok(());
        }
    }
}
