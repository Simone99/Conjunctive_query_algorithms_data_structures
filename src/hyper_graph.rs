use crate::conjunctive_query::ConjunctiveQuery;

#[derive(Clone)]
pub struct HyperEdge {
    vertices: Vec<String>,
}

#[derive(Clone)]
pub struct HyperGraph {
    v: Vec<String>,
    e: Vec<HyperEdge>,
}

impl HyperEdge {
    pub fn new(v: Vec<String>) -> HyperEdge {
        HyperEdge { vertices: v }
    }

    pub fn contains(&self, element: &String) -> bool {
        self.vertices.contains(element)
    }

    pub fn print(&self) {
        let mut i = 0;
        print!("[ ");
        for vertex in &self.vertices {
            print!("{}", vertex);
            if i != (self.vertices.len() - 1) {
                print!(", ");
            } else {
                print!(" ");
            }
            i += 1;
        }
        print!("]")
    }
}

impl PartialEq for HyperEdge {
    fn eq(&self, other: &Self) -> bool {
        self.vertices == other.vertices
    }
}

impl HyperGraph {
    pub fn new(cq: &ConjunctiveQuery) -> HyperGraph {
        let mut result = HyperGraph {
            v: Vec::new(),
            e: Vec::new(),
        };
        for var in cq.var() {
            result.v.push(var);
        }
        for atom in cq.atoms() {
            result.e.push(HyperEdge::new(atom.get_variables()));
        }
        result
    }

    pub fn ears(&self) -> Vec<&HyperEdge> {
        let mut ears_list = Vec::new();
        for hyperedge in &self.e {
            let mut not_exclusive_vertices = Vec::new();
            for element in &hyperedge.vertices {
                for he in &self.e {
                    if he == hyperedge {
                        continue;
                    }
                    if he.contains(element) {
                        not_exclusive_vertices.push(element);
                    }
                }
            }
            if not_exclusive_vertices.len() == 0 {
                ears_list.push(hyperedge);
            } else {
                for he in &self.e {
                    if hyperedge == he {
                        continue;
                    }
                    let mut contains_all = true;
                    for item in &not_exclusive_vertices {
                        if !he.contains(*item) {
                            contains_all = false;
                            break;
                        }
                    }
                    if contains_all {
                        ears_list.push(hyperedge);
                    }
                }
            }
        }
        ears_list
    }

    pub fn gyo(&self) -> bool {
        let mut h_ = self.clone();
        loop {
            let ears_list = h_.ears();

            if ears_list.len() == 0 {
                break;
            }
            let e = ears_list[0];
            let mut exclusive_vertices = e.clone();
            for element in &e.vertices {
                for he in &h_.e {
                    if he == e {
                        continue;
                    }
                    if he.contains(element) && exclusive_vertices.contains(element) {
                        exclusive_vertices.vertices.swap_remove(
                            exclusive_vertices
                                .vertices
                                .iter()
                                .position(|x| x == element)
                                .expect("Generic error!"),
                        );
                    }
                }
            }

            h_.e.swap_remove(h_.e.iter().position(|x| x == e).expect("Generic error!"));
        }
        let result = h_.e.len() == 0;
        result
    }

    pub fn print(&self) {
        println!("Hypergraph:");
        println!("Number of vertices: {}", self.v.len());
        println!("Number of hyper-edges: {}", self.e.len());
        print!("Vertices: ");
        for vertex in &self.v {
            print!("{} ", vertex);
        }
        println!();
        println!("Hyper-edges:");
        for hyperedge in &self.e {
            hyperedge.print();
            println!();
        }
    }
}
