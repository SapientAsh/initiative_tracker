use serde::{Deserialize, Serialize};
use std::fs;
use std::{
    fmt,
    io::{self, Write},
};

#[derive(Serialize, Deserialize)]
struct JSONCharacter {
    name: String,
    ac: u8,
    hp: u16,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(from = "JSONCharacter")]
struct Character {
    name: String,
    ac: u8,
    current: u16,
    max: u16,
    score: u8,
    temp: u16,
}

impl From<Character> for JSONCharacter {
    fn from(value: Character) -> Self {
        JSONCharacter { 
            name: value.name,
            ac: value.ac, 
            hp: value.max 
        }
    }
}

impl Character {
    fn new(name: String, ac: u8, max: u16, score: u8) -> Self {
        Character {
            name: name,
            ac: ac,
            current: max,
            max: max,
            score: score,
            temp: 0,
        }
    }

    fn damage(&mut self, mut amount: u16) {
        if self.temp > 0 {
            if self.temp > amount {
                self.temp -= amount;
                return;
            }
        }
        amount -= self.temp;
        self.temp = 0;

        if self.current < amount {
            self.current = 0;
            return;
        }

        self.current -= amount;
    }

    fn heal(&mut self, amount: u16) {
        self.current += amount;
        if self.current > self.max {
            self.current = self.max
        }
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hp_length = match self.temp > 0 {
            true => {
                self.current.to_string().len()
                    + self.max.to_string().len()
                    + self.temp.to_string().len()
                    + 8
            }
            false => self.current.to_string().len() + self.max.to_string().len() + 5,
        };
        let max_width = std::cmp::max(15, self.name.len() + 2);
        let max_width = std::cmp::max(max_width, hp_length + 1);
        let buffer = (max_width + self.name.len() % 2) % 2;

        let display_string = match self.temp > 0 {
            true => format!(
                "┌{}┐ \n\
            │{}{}{}│ \n\
            │ HP {}/{} + {}{}│ \n\
            │ AC {}{}│ \n\
            │ Init {}{}│ \n\
            └{}┘ \n\n",
                "─".repeat(max_width),
                " ".repeat((max_width - self.name.len()) / 2),
                self.name,
                " ".repeat((max_width - self.name.len()) / 2 + buffer),
                self.current,
                self.max,
                self.temp,
                " ".repeat(max_width - hp_length),
                self.ac,
                " ".repeat(max_width - 4 - self.ac.to_string().len()),
                self.score,
                " ".repeat(max_width - 6 - self.score.to_string().len()),
                "─".repeat(max_width)
            ),
            false => format!(
                "┌{}┐ \n\
            │{}{}{}│ \n\
            │ HP {}/{}{}│ \n\
            │ AC {}{}│ \n\
            │ Init {}{}│ \n\
            └{}┘ \n\n",
                "─".repeat(max_width),
                " ".repeat((max_width - self.name.len()) / 2),
                self.name,
                " ".repeat((max_width - self.name.len()) / 2 + buffer),
                self.current,
                self.max,
                " ".repeat(max_width - hp_length),
                self.ac,
                " ".repeat(max_width - 4 - self.ac.to_string().len()),
                self.score,
                " ".repeat(max_width - 6 - self.score.to_string().len()),
                "─".repeat(max_width)
            ),
        };

        write!(f, "{display_string}")
    }
}

impl From<JSONCharacter> for Character {
    fn from(value: JSONCharacter) -> Self {
        let mut score = prompt(format!("{}: ", value.name).as_str()).parse::<u8>();

        while score.is_err() {
            score = prompt("Enter a number between 0-255: ").parse::<u8>();
        }

        Character {
            name: value.name,
            ac: value.ac,
            current: value.hp,
            max: value.hp,
            score: score.unwrap(),
            temp: 0
        }
    }
}

struct Node {
    data: Character,
    next: Option<*mut Node>,
}

impl Node {
    fn new(data: Character) -> Self {
        Node {
            data: data,
            next: None,
        }
    }
}

struct Initiative {
    head: Option<*mut Node>,
    current: Option<*mut Node>,
}

impl Initiative {
    fn new() -> Self {
        Initiative {
            head: None,
            current: None,
        }
    }

    fn add(&mut self, char: Character) {
        unsafe {
            if self.head.is_none() {
                self.head = Some(Box::into_raw(Box::new(Node::new(char))));
                self.current = self.head;
                return;
            }

            if (*self.head.unwrap()).data.score < char.score {
                let mut temp = Node::new(char);
                temp.next = self.head;
                self.head = Some(Box::into_raw(Box::new(temp)));
                return;
            }

            if (*self.head.unwrap()).next.is_none() {
                let temp = Node::new(char);
                (*self.head.unwrap()).next = Some(Box::into_raw(Box::new(temp)));
                return;
            }

            let mut current = self.head;
            while (*current.unwrap()).next.is_some()
                && (*(*current.unwrap()).next.unwrap()).data.score > char.score
            {
                current = (*current.unwrap()).next;
            }

            if (*current.unwrap()).next.is_none() {
                (*current.unwrap()).next = Some(Box::into_raw(Box::new(Node::new(char))));
                return;
            }

            let mut temp = Node::new(char);
            temp.next = (*current.unwrap()).next;
            (*current.unwrap()).next = Some(Box::into_raw(Box::new(temp)));
        }
    }

    fn display(&self) {
        if self.current.is_none() {
            print!("Initiative order is empty");
            return;
        }
        unsafe {
            print!("{}", (*self.current.unwrap()).data);
        }
    }

    fn show(&self, name: String) {
        let target = self.find(name);
        if target.is_some() {
            unsafe {
                print!("{}", (*target.unwrap()).data);
            }
        }
    }

    fn advance(&mut self) {
        unsafe {
            if self.current.is_none() || (*self.current.unwrap()).next.is_none() {
                self.current = self.head;
                return;
            }
            self.current = (*self.current.unwrap()).next;
        }
    }

    fn find(&self, name: String) -> Option<*mut Node> {
        if self.head.is_none() {
            return None;
        }

        let mut current = self.head;

        while current.is_some() {
            unsafe {
                if (*current.unwrap()).data.name == name {
                    return current;
                }
                current = (*current.unwrap()).next;
            }
        }

        return None;
    }

    fn remove(&mut self, name: String) {
        if self.head.is_none() {
            println!("Initiative order is empty");
            return;
        }

        unsafe {
            if (*self.head.unwrap()).data.name == name {
                let next_node = (*self.head.unwrap()).next;
                if self.current.unwrap() == self.head.unwrap() {
                    self.current = next_node;
                }
                self.head = next_node;
                return;
            }

            let mut current = self.head;
            while (*current.unwrap()).next.is_some() {
                if (*(*current.unwrap()).next.unwrap()).data.name == name {
                    (*current.unwrap()).next = (*(*current.unwrap()).next.unwrap()).next;
                    return;
                }
                current = (*current.unwrap()).next;
            }
        }
    }

    fn temp(&self, name: String, amount: u16) {
        let target = self.find(name);
        if target.is_some() {
            unsafe {
                (*target.unwrap()).data.temp = amount;
            }
        }
    }

    fn damage(&self, name: String, amount: u16) {
        let target = self.find(name);
        if target.is_some() {
            unsafe {
                (*target.unwrap()).data.damage(amount);
            }
        }
    }

    fn heal(&self, name: String, amount: u16) {
        let target = self.find(name);
        if target.is_some() {
            unsafe {
                (*target.unwrap()).data.heal(amount);
            }
        }
    }

    fn import(&mut self, path: &str) -> Result<(), &'static str> {
        let f: String = match fs::read_to_string(path) {
            Ok(v) => v,
            Err(_) => return Err("Provided path is not valid"),
        };

        let chars: Vec<Character> = match serde_json::from_str(&f) {
            Ok(v) => v,
            Err(_) => {
                return Err("The provided file is not JSON or is not in the expected format.");
            }
        };

        chars.iter().for_each(|x| {
            self.add(x.clone());
        });

        Ok(())
    }

    fn export(&self, path: &str) -> Result<(), &'static str> {
        if self.head.is_none() {
            return Err("Initiative order is empty");
        }

        let mut chars = Vec::<JSONCharacter>::new();
        let mut current = self.head;

        unsafe {
            while current.is_some() {
                let char: &Character = &(*current.unwrap()).data;
                /*let json_char = JSONCharacter {
                    name: char.name.clone(),
                    ac: char.ac,
                    hp: char.max,
                };*/
                chars.push(char.clone().into());
                current = (*current.unwrap()).next;
            }
        }

        let json = serde_json::to_string_pretty(&chars).unwrap();
        let mut f: fs::File = match fs::File::create_new(path) {
            Ok(f) => f,
            Err(_) => return Err("Path is invalid or file already exists"),
        };

        if f.write_all(json.as_bytes()).is_err() {
            return Err("Could not save to file");
        }

        Ok(())
    }

    fn beginning(&mut self) {
        if self.head.is_some() {
            self.current = self.head;
        }
    }
}

impl fmt::Display for Initiative {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.head.is_none() {
            return write!(f, "Initiative order is empty \n");
        }

        let mut current = self.head;
        while current.is_some() {
            unsafe {
                write!(f, "{}", (*current.unwrap()).data).unwrap();
                current = (*current.unwrap()).next;
            }
        }
        return Ok(());
    }
}

fn prompt(prompt: &str) -> String {
    let input = &mut String::new();

    input.clear();
    print!("{prompt}");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();

    input.trim().to_string()
}

fn main() {
    println!("┌──────────────────────────────────┐");
    println!("│                                  │");
    println!("│ SapientAsh's Initiative Tracker! │");
    println!("│                                  │");
    println!("└──────────────────────────────────┘");
    let mut init = Initiative::new();

    loop {
        let input = prompt("> ");
        let input = input.trim();
        println!();

        match input {
            "help" => {
                println!(
                    "Available commands: \n\
                import: Add characters to initiative order from a compatible JSON file \n\
                export: Save initiative order to JSON file that can be imported \n\
                add: Add character to initiative order manually \n\
                next: Advance initiative order to the next turn \n\
                exit: Close this program \n\
                display: Print the full initiative order to the console \n\
                current: Print the current turn to the console \n\
                show: Print a specific character to the console \n\
                damage: Deal damage to a specified character \n\
                heal: Heal a specified character \n\
                temp: Grant temporary HP to a specified character \n\
                remove: Remove a specified character from the initiative order \n\
                top: Set the current turn to the first in initiative order (useful after adding initial characters) \
                "
                )
            }
            "import" => {
                let path: String = prompt("Enter path to JSON: ");

                let result: Result<(), &'static str> = init.import(path.as_str());
                if result.is_err() {
                    println!("{}", result.unwrap_err());
                }
            }
            "export" => {
                let path = prompt("Enter target path for JSON file: ");

                let result: Result<(), &'static str> = init.export(path.as_str());
                if result.is_err() {
                    println!("{}", result.unwrap_err());
                }
            }
            "add" => {
                let name = prompt("Name: ");
                let mut ac = prompt("AC: ").trim().parse::<u8>();
                while ac.is_err() {
                    ac = prompt("Enter a number between 0-255: ").parse::<u8>();
                }
                let mut max = prompt("HP: ").trim().parse::<u16>();
                while max.is_err() {
                    max = prompt("Enter a number between 0-65535: ").parse::<u16>();
                }
                let mut score = prompt("Score: ").trim().parse::<u8>();
                while score.is_err() {
                    score = prompt("Enter a number between 0-255: ").parse::<u8>();
                }

                println!();

                init.add(Character::new(
                    name,
                    ac.unwrap(),
                    max.unwrap(),
                    score.unwrap(),
                ));
            }
            "next" => {
                init.advance();
                init.display();
            }
            "exit" => {
                return;
            }
            "display" => {
                print!("{init}");
            }
            "current" => {
                init.display();
            }
            "show" => {
                let name = prompt("Name: ");
                println!();
                init.show(name);
            }
            "damage" => {
                let name = prompt("Name: ");
                let mut amount = prompt("Amount: ").parse::<u16>();
                while amount.is_err() {
                    amount = prompt("Enter a number between 0-65535: ").parse::<u16>();
                }
                init.damage(name, amount.unwrap());
            }
            "heal" => {
                let name = prompt("Name: ");
                let mut amount = prompt("Amount: ").parse::<u16>();
                while amount.is_err() {
                    amount = prompt("Enter a number between 0-65535: ").parse::<u16>();
                }
                init.heal(name, amount.unwrap());
            }
            "temp" => {
                let name = prompt("Name: ");
                let mut amount = prompt("Amount: ").parse::<u16>();
                while amount.is_err() {
                    amount = prompt("Enter a number between 0-65535: ").parse::<u16>();
                }
                init.temp(name, amount.unwrap());
            }
            "remove" => {
                let name = prompt("Name: ");
                init.remove(name);
            }
            "top" => {
                init.beginning();
                init.display();
            }
            _ => {
                println!("Sorry, I didn't understand that.");
            }
        };
        println!();
    }
}
