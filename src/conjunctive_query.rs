use rand::Rng;
use regex::Regex;
use std::{collections::HashSet, fs::File, io::Write};

const N_MAX_RECORDS: usize = 10000000;
const MAX_RECORD_VALUE: usize = 300000;

#[derive(Clone)]
pub struct Atom {
    name: String,
    variables: Vec<String>,
}
#[derive(Clone)]
pub struct ConjunctiveQuery {
    atoms_list: Vec<Atom>,
    query_name: String,
    head_variables: Vec<String>,
    is_boolean: bool,
}

impl Atom {
    pub fn new() -> Atom {
        Atom {
            name: String::from(""),
            variables: Vec::new(),
        }
    }

    pub fn get_variables(&self) -> Vec<String> {
        self.variables.clone()
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn intersect(&self, other: &Atom) -> Vec<String> {
        let mut result = Vec::new();
        for variable in &self.variables {
            for v in &other.variables {
                if variable == v {
                    result.push(variable.clone());
                }
            }
        }
        result
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        let mut i = 0;
        result += format!("{}( ", self.name).as_str();
        for variable in &self.variables {
            result += format!("{}", variable).as_str();
            if i == (self.variables.len() - 1) {
                result += " ";
            } else {
                result += ", ";
            }
            i += 1;
        }
        result += ")";
        result
    }

    pub fn print(&self) {
        let mut i = 0;
        print!("{}( ", self.name);
        for variable in &self.variables {
            print!("{}", variable);
            if i == (self.variables.len() - 1) {
                print!(" ");
            } else {
                print!(", ");
            }
            i += 1;
        }
        print!(")");
    }
}

impl PartialEq for Atom {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.variables == other.variables
    }
}

impl ConjunctiveQuery {
    pub fn new(text_description: &str) -> ConjunctiveQuery {
        let mut result = ConjunctiveQuery {
            atoms_list: Vec::new(),
            query_name: String::from("Tmp"),
            head_variables: Vec::new(),
            is_boolean: true,
        };
        let name = "[a-zA-Z_][a-zA-Z_0-9]*";
        let name_list = format!("({name}(?:, {name})*)");
        let mut re = Regex::new(
            format!(
                "{name}\\({name_list}?\\) :- {name}\\({name_list}\\)(, {name}\\({name_list}\\))*"
            )
            .as_str(),
        )
        .expect("Something went wrong when compiling the regex!");
        if re.is_match(text_description) {
            re = Regex::new(format!("({name})\\({name_list}?\\)").as_str())
                .expect("Something went wrong compiling the regex!");
            let mut iterator = re.captures_iter(text_description);
            let head = iterator.next().expect("Boh");
            result.query_name = String::from(&head[1]);
            if head.get(2).is_some() {
                result.is_boolean = false;
                for variable in (&head[2]).split(',') {
                    result.head_variables.push(String::from(variable.trim()));
                }
            }
            for group in iterator {
                let mut atom = Atom::new();
                atom.name = String::from(&group[1]);
                for variable in (&group[2]).split(',') {
                    atom.variables.push(String::from(variable.trim()));
                }
                result.atoms_list.push(atom);
            }
        } else {
            panic!("Error! The input is not a conjunctive query!");
        }
        result
    }

    pub fn atoms(&self) -> Vec<Atom> {
        self.atoms_list.clone()
    }

    pub fn var(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        for atom in &self.atoms_list {
            for variable in &atom.variables {
                result.insert(variable.clone());
            }
        }
        result
    }

    pub fn head(&self) -> Vec<String> {
        return self.head_variables.clone();
    }

    pub fn generate_random_data(&self, database_file: &mut File) {
        let err_msg = "Error writing on the file!";
        for atom in &self.atoms_list {
            writeln!(database_file, "{}", atom.to_string()).expect(err_msg);
            let n_columns = atom.variables.len();
            let mut rng = rand::thread_rng();
            for _ in 0..rng.gen_range(0, N_MAX_RECORDS) {
                for _ in 0..(n_columns - 1) {
                    write!(database_file, "{} ", rng.gen_range(0, MAX_RECORD_VALUE))
                        .expect(err_msg);
                }
                writeln!(database_file, "{}", rng.gen_range(0, MAX_RECORD_VALUE)).expect(err_msg);
            }
        }
    }

    pub fn print(&self) {
        println!("Query name: {}", self.query_name);
        print!("Head variables: ");
        for variable in &self.head_variables {
            print!("{} ", variable);
        }
        println!();
        println!("Is query boolean? {}", self.is_boolean);
        println!("Number of atoms: {}", self.atoms_list.len());
        println!("Atoms: ");
        for atom in &self.atoms_list {
            print!("name: {} ", atom.name);
            print!("variables: ");
            for variable in &atom.variables {
                print!("{} ", variable);
            }
            println!();
        }
    }
}
