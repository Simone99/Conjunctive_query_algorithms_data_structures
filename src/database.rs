use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    fs::File,
    hash::Hash,
    io::Read,
    str::FromStr,
};

use crate::{conjunctive_query::ConjunctiveQuery, join_forest::JoinForest};

const DEBUG: bool = false;
const QUERY_RESULT: &'static str = "Query result";

#[derive(Clone)]
struct Table<T: Display + Hash + PartialEq + Eq + Clone + FromStr + Copy> {
    name: String,
    attributes: Vec<String>,
    records: Vec<Vec<T>>,
}

#[derive(Clone)]
pub struct Database<T: Display + Hash + PartialEq + Eq + Clone + FromStr + Copy> {
    tables: HashMap<String, Table<T>>,
}

impl<T: Display + Hash + PartialEq + Eq + Clone + FromStr + Copy> Table<T> {
    pub fn new(name: String) -> Table<T> {
        Table {
            name: name,
            attributes: Vec::new(),
            records: Vec::new(),
        }
    }

    // Not used in the current implementation
    /*pub fn select(&self, attribute: &String, value: T) -> Vec<Vec<T>> {
        let attribute_index = self
            .attributes
            .iter()
            .position(|x| x == attribute)
            .expect(format!("Attribute {} not in table {}", attribute, self.name).as_str());
        let mut result_tmp = HashSet::new();
        for record in &self.records {
            if record[attribute_index] == value {
                result_tmp.insert(record.clone());
            }
        }
        let mut result = Vec::new();
        for record in result_tmp {
            result.push(record.clone());
        }
        result
    }*/

    pub fn project(&self, attributes: Vec<String>) -> Vec<Vec<T>> {
        // Retrieve all attribute indexes in the table
        let mut attributes_indexes = Vec::new();
        for attribute in &attributes {
            attributes_indexes.push(
                self.attributes
                    .iter()
                    .position(|x| x == attribute)
                    .expect(format!("Attribute {} not in table {}", attribute, self.name).as_str()),
            );
        }
        let mut result_tmp = HashSet::new();
        for record in &self.records {
            let mut record_with_projection = Vec::new();
            for index in &attributes_indexes {
                record_with_projection.push(record[*index]);
            }
            result_tmp.insert(record_with_projection);
        }
        let mut result = Vec::new();
        for record in result_tmp {
            result.push(record.clone());
        }
        result
    }

    pub fn natural_join(&self, other_table: &Table<T>) -> Table<T> {
        // Implementation of the classic hash join algorithm
        let mut join_result = Table::new(format!("{} join {}", self.name, other_table.name));
        let common_attributes = (&self.attributes)
            .into_iter()
            .filter(|x| other_table.attributes.contains(x))
            .collect::<Vec<&String>>();

        for attribute in &self.attributes {
            if !common_attributes.contains(&attribute) {
                join_result.attributes.push(attribute.clone());
            }
        }
        for common_attribute in common_attributes.clone() {
            join_result.attributes.push(common_attribute.clone());
        }
        for attribute in &other_table.attributes {
            if !common_attributes.contains(&attribute) {
                join_result.attributes.push(attribute.clone());
            }
        }

        let mut common_attribute_indexes_table1 = Vec::new();
        let mut common_attribute_indexes_table2 = Vec::new();
        for attribute in common_attributes {
            common_attribute_indexes_table1.push(
                self.attributes
                    .iter()
                    .position(|x| x == attribute)
                    .expect(format!("Attribute {} not in table {}", attribute, self.name).as_str()),
            );
            common_attribute_indexes_table2.push(
                other_table
                    .attributes
                    .iter()
                    .position(|x| x == attribute)
                    .expect(format!("Attribute {} not in table {}", attribute, self.name).as_str()),
            );
        }

        let mut hash_table1 = HashMap::new();
        for record in &self.records {
            let mut key1 = Vec::new();
            for index in &common_attribute_indexes_table1 {
                key1.push(record[*index]);
            }
            if !hash_table1.contains_key(&key1) {
                hash_table1.insert(key1.clone(), HashSet::new());
            }
            let mut value1 = Vec::new();
            for i in 0..record.len() {
                if !common_attribute_indexes_table1.contains(&i) {
                    value1.push(record[i]);
                }
            }
            hash_table1.get_mut(&key1).unwrap().insert(value1);
        }
        for record in &other_table.records {
            let mut key2 = Vec::new();
            for index in &common_attribute_indexes_table2 {
                key2.push(record[*index]);
            }
            if hash_table1.contains_key(&key2) {
                let table1_records = hash_table1.get(&key2).unwrap();
                for table1_record in table1_records {
                    let mut join_record = Vec::new();
                    for value in table1_record {
                        join_record.push(*value);
                    }
                    for value in &key2 {
                        join_record.push(*value);
                    }
                    for index in 0..record.len() {
                        if !common_attribute_indexes_table2.contains(&index) {
                            join_record.push(record[index]);
                        }
                    }
                    join_result.records.push(join_record);
                }
            }
        }
        join_result
    }

    pub fn print(&self) {
        println!("Table name: {}", self.name);
        print!("Attributes: ");
        let mut i = 0;
        for attribute in &self.attributes {
            print!("{}", attribute);
            if i == (self.attributes.len() - 1) {
                print!("\n");
            } else {
                print!(" ");
            }
            i += 1;
        }
        for record in &self.records {
            i = 0;
            for element in record {
                print!("{}", element);
                if i == (record.len() - 1) {
                    print!("\n");
                } else {
                    print!(" ");
                }
                i += 1;
            }
        }
    }
}

impl<T: Display + Hash + PartialEq + Eq + Clone + FromStr + Copy> Database<T>
where
    <T as FromStr>::Err: Debug,
{
    pub fn new(database_file: &mut File) -> Database<T> {
        let mut result = Database {
            tables: HashMap::new(),
        };
        let mut database_string = String::new();
        database_file
            .read_to_string(&mut database_string)
            .expect("Error reading the database file!");
        let database_lines: Vec<&str> = database_string
            .trim()
            .split('\n') /*.map(|x| String::from(x))*/
            .collect();
        let name = "[a-zA-Z_][a-zA-Z_0-9]*";
        let name_list = format!("({name}(?:, {name})*)");
        let re = Regex::new(format!("({name})\\( {name_list}? \\)").as_str())
            .expect("Something went wrong compiling the regex!");
        let mut current_table = String::from("");
        for line in database_lines {
            if re.is_match(line) {
                for group in re.captures_iter(line) {
                    let mut table = Table::new(String::from(&group[1]));
                    for attribute in (&group[2]).split(',') {
                        table.attributes.push(String::from(attribute.trim()));
                    }
                    result.tables.insert(table.name.clone(), table.clone());
                    current_table = table.name.clone();
                }
            } else {
                result.tables.get_mut(&current_table).unwrap().records.push(
                    line.split(" ")
                        .into_iter()
                        .map(|x| x.parse::<T>().expect("Error parsing the file!"))
                        .collect(),
                );
            }
        }
        result
    }

    pub fn yannakakis(&mut self, cq: &ConjunctiveQuery) {
        // Simplified version of origina Yannakakis algorithm
        for root in &JoinForest::new(cq).get_roots() {
            let post_order_tree = root.borrow().post_order_wrapper();

            for r in &post_order_tree {
                if DEBUG {
                    println!("R: {}", &r.borrow().get_relation_name());
                }
                for s in &r.borrow().get_children() {
                    if DEBUG {
                        println!("S: {}", &s.borrow().get_relation_name());
                    }
                    let tmp_table_r = self.tables.get(&r.borrow().get_relation_name()).unwrap();
                    let tmp_table_s = self.tables.get(&s.borrow().get_relation_name()).unwrap();

                    if DEBUG {
                        println!("Join between {} and {}", tmp_table_r.name, tmp_table_s.name);
                    }

                    let mut result_tmp = tmp_table_r.natural_join(tmp_table_s);

                    if DEBUG {
                        println!("Join result:");
                        result_tmp.print();
                    }

                    let mut projection_variables = Vec::new();
                    for variable in &r.borrow().get_variables() {
                        if !projection_variables.contains(variable) {
                            projection_variables.push(variable.clone());
                        }
                    }
                    for variable in &cq.head() {
                        if (tmp_table_r.attributes.contains(variable)
                            || tmp_table_s.attributes.contains(variable))
                            && !projection_variables.contains(variable)
                        {
                            projection_variables.push(variable.clone());
                        }
                    }

                    if DEBUG {
                        print!("Projection variables: ");
                        let mut i = 0;
                        for variable in &projection_variables {
                            print!("{}", variable);
                            if i == (projection_variables.len() - 1) {
                                print!("\n");
                            } else {
                                print!(" ");
                            }
                            i += 1;
                        }
                    }

                    let tmp = result_tmp.project(projection_variables.clone());

                    if DEBUG {
                        println!("Projection result:");
                        for record in &tmp {
                            let mut i = 0;
                            for element in record {
                                print!("{}", element);
                                if i == (record.len() - 1) {
                                    print!("\n");
                                } else {
                                    print!(" ");
                                }
                                i += 1;
                            }
                        }
                    }

                    result_tmp = Table::new(r.borrow().get_relation_name());
                    result_tmp.attributes = projection_variables.clone();
                    result_tmp.records = tmp;
                    self.tables.insert(result_tmp.name.clone(), result_tmp);
                }
            }
            let mut tmp = self
                .tables
                .get(&root.borrow().get_relation_name())
                .unwrap()
                .clone();
            self.tables.remove(&root.borrow().get_relation_name());
            tmp.name = format!("{} {}", QUERY_RESULT, &root.borrow().get_relation_name());
            tmp.attributes = cq.head();
            tmp.records = tmp.project(cq.head());
            self.tables.insert(tmp.name.clone(), tmp);
        }
    }

    pub fn print_query_results(&self) {
        let mut query_result_available = false;
        for key in self.tables.keys() {
            if key.starts_with(QUERY_RESULT) {
                query_result_available = true;
                self.tables.get(key).unwrap().print();
            }
        }
        if !query_result_available {
            println!("No query has been executed on the database!");
        }
    }

    pub fn print(&self) {
        println!("Database:");
        for table in self.tables.values() {
            table.print();
            println!();
        }
    }
}
