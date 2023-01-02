pub mod conjunctive_query;
pub mod database;
pub mod hyper_graph;
pub mod join_forest;

use conjunctive_query::ConjunctiveQuery;
use hyper_graph::HyperGraph;
use std::{fs::File, path::Path, time::Instant};

use crate::database::Database;

const DATABASE_FILE: &'static str = "database.txt";

fn main() {
    // let cq = ConjunctiveQuery::new("q1(x, y, z) :- R(x, y), R(y, z), R(z, x)"); // Triangle query
    // let cq = ConjunctiveQuery::new("q1() :- R(x, y), S(y, z), T(z, w)"); // Path of length 3 query
    let cq = ConjunctiveQuery::new("q(x, t) :- R(x, y, z), S(y, v), T(y, z, u), U(z, u, w), V(u, w, t)"); // Query from lesson 4
    // let cq = ConjunctiveQuery::new("q(x, t) :- R(x, y, z), S(y, v), T(y, z, u), U(z, u, w), V(u, w, t), W(a, b), X(b, c)"); // Query from lesson 4 modified to be a forest
    // let cq = ConjunctiveQuery::new("q() :- R(x, y), S(z, w)");
    // let cq = ConjunctiveQuery::new("q(w, z) :- R(y, z), G(x, y), S1(y, z, u), S2(z, u, w), T1(y, z), T2(z, u)"); // Query from paper found online
    cq.print();
    let h = HyperGraph::new(&cq);
    h.print();
    println!("Hypergraph is alpha-acyclic: {}", h.gyo());
    if h.gyo() {
        if !Path::new(DATABASE_FILE).exists() {
            let mut database_file =
                File::create(DATABASE_FILE).expect("Error creating the database file!");
            println!("Generating random data...");
            cq.generate_random_data(&mut database_file);
        }
        let mut database_file =
            File::open(DATABASE_FILE).expect("Error opening the database file!");
        println!("Loading generated data...");
        let mut database: Database<u64> = Database::new(&mut database_file);
        println!("Start timing...");
        let now = Instant::now();
        database.yannakakis(&cq);
        let elapsed_time = now.elapsed();
        println!(
            "Yannakakis algorithm ran in {:.3}s",
            elapsed_time.as_secs_f64()
        );
        database.print_query_results();
    }
}
