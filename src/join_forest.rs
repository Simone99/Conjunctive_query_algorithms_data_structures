use std::{cell::RefCell, rc::Rc};

use crate::conjunctive_query::{Atom, ConjunctiveQuery};

const DEBUG: bool = false;

#[derive(Clone)]
pub struct JoinForestNode {
    value: Atom,
    children: Vec<(Rc<RefCell<JoinForestNode>>, Vec<String>)>,
    parent: Option<(Rc<RefCell<JoinForestNode>>, Vec<String>)>,
}
pub struct JoinForest {
    roots: Vec<Rc<RefCell<JoinForestNode>>>,
}

impl JoinForestNode {
    fn new(value: Atom) -> JoinForestNode {
        JoinForestNode {
            value: value,
            children: Vec::new(),
            parent: None,
        }
    }

    fn add_child(&mut self, new_node: (Rc<RefCell<JoinForestNode>>, Vec<String>)) {
        new_node.0.borrow_mut().parent =
            Some((Rc::new(RefCell::new(self.clone())), new_node.1.clone()));
        self.children.push(new_node);
    }

    fn search_wrapper(
        root: Rc<RefCell<JoinForestNode>>,
        value: &Atom,
    ) -> Option<Rc<RefCell<JoinForestNode>>> {
        let root_borrowed = root.borrow();
        if root_borrowed.children.len() == 0 {
            if root_borrowed.value == *value {
                return Some(Rc::clone(&root));
            }
            return None;
        }
        if root_borrowed.value == *value {
            return Some(Rc::clone(&root));
        }
        for child in root_borrowed.children.iter() {
            let result = JoinForestNode::search(child, value);
            if result.is_some() {
                return result;
            }
        }
        return None;
    }

    fn search(
        root: &(Rc<RefCell<JoinForestNode>>, Vec<String>),
        value: &Atom,
    ) -> Option<Rc<RefCell<JoinForestNode>>> {
        if root.0.borrow().children.len() == 0 {
            if root.0.borrow().value == *value {
                return Some(Rc::clone(&root.0));
            }
            return None;
        }
        if root.0.borrow().value == *value {
            return Some(Rc::clone(&root.0));
        }
        for child in root.0.borrow().children.iter() {
            let result = JoinForestNode::search(child, value);
            if result.is_some() {
                return result;
            }
        }
        return None;
    }

    fn walk_on_path_wrapper(
        start: Rc<RefCell<JoinForestNode>>,
        mut path: Vec<String>,
        value: &Atom,
        atom_pairs: Vec<(&Atom, &Atom, Vec<String>)>,
    ) {
        let mut start_borrowed = start.borrow_mut();
        if start_borrowed.children.len() == 0 {
            if start_borrowed.parent.is_none() {
                // If the node has no parent and no children, insert the new node
                start_borrowed.add_child((
                    Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                    path.clone(),
                ));
                return;
            }
            // If it has one parent, check it with the same logic of walk_on_path function
            let parent = start_borrowed.parent.as_ref();
            if parent.unwrap().1.iter().any(|x| path.contains(x)) {
                let path_original = path.clone();
                path.retain(|x| !parent.unwrap().1.contains(x));
                if !JoinForestNode::walk_on_path(&parent.unwrap(), path, value, atom_pairs) {
                    start_borrowed.add_child((
                        // If I wasn't able to insert the new node in the parent sub-tree I insert it in the current node
                        Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                        path_original.clone(),
                    ));
                    return;
                }
                return;
            } else {
                start_borrowed.add_child((
                    Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                    path.clone(),
                ));
                return;
            }
        } else if start_borrowed.parent.is_none() {
            for child in &start_borrowed.children {
                let mut path_copy = path.clone();
                if child.1.iter().any(|x| path_copy.contains(x)) {
                    path_copy.retain(|x| !child.1.contains(x));
                    if JoinForestNode::walk_on_path(child, path_copy, value, atom_pairs.clone()) {
                        return;
                    }
                }
            }
            start_borrowed.add_child((
                Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                path.clone(),
            ));
            return;
        } else {
            for child in &start_borrowed.children {
                let mut path_copy = path.clone();
                if child.1.iter().any(|x| path_copy.contains(x)) {
                    path_copy.retain(|x| !child.1.contains(x));
                    if JoinForestNode::walk_on_path(child, path_copy, value, atom_pairs.clone()) {
                        return;
                    }
                }
            }
            let parent = start_borrowed.parent.as_ref();
            if parent.unwrap().1.iter().any(|x| path.contains(x)) {
                let path_original = path.clone();
                path.retain(|x| !parent.unwrap().1.contains(x));
                if !JoinForestNode::walk_on_path(&parent.unwrap(), path, value, atom_pairs) {
                    start_borrowed.add_child((
                        Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                        path_original.clone(),
                    ));
                    return;
                }
                return;
            } else {
                start_borrowed.add_child((
                    Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                    path.clone(),
                ));
                return;
            }
        }
    }

    // If I'm in this function, the node I'm currently on can't be the starting one and the value it's not in the tree yet
    fn walk_on_path(
        start: &(Rc<RefCell<JoinForestNode>>, Vec<String>),
        mut path: Vec<String>,
        value: &Atom,
        atom_pairs: Vec<(&Atom, &Atom, Vec<String>)>,
    ) -> bool {
        let mut start_borrowed = start.0.borrow_mut();
        if path.len() == 0 {
            // Look for a pair with both the current node and the value I'm trying to insert
            for pair in &atom_pairs {
                if pair.0 == value && pair.1 == &start_borrowed.value
                    || pair.1 == value && pair.0 == &start_borrowed.value
                {
                    // Given that path.len() = 0 I don't check for the existence of all path values in pair.2 and I just insert the node
                    start_borrowed.add_child((
                        Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                        pair.2.clone(),
                    ));
                    return true;
                }
            }
            return false;
        } else {
            if start_borrowed.children.len() == 0 {
                if start_borrowed.parent.is_none() {
                    // TODO: make sure this case actually exists, because if I'm in the current function it means that I've consumed at least one element in the path, so the current node should have at least one child or one parent
                    for pair in &atom_pairs {
                        if pair.0 == value && pair.1 == &start_borrowed.value
                            || pair.1 == value && pair.0 == &start_borrowed.value
                        {
                            // Given that path.len() = 0 I don't check for the existence of path values in pair.2 and I just insert the node
                            start_borrowed.add_child((
                                Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                                pair.2.clone(),
                            ));
                            return true;
                        }
                    }
                    return false;
                }
                // TODO: as in the previous TODO comment, check if this case exists, because if the node has no child and one parent it means that we came to the current node from the parent
                // Check the parent node
                let parent = start_borrowed.parent.as_ref();
                if parent.unwrap().1.iter().any(|x| path.contains(x)) {
                    // If the parent contains at least one variable in the path, go to the parent and consume the common variables
                    path.retain(|x| !parent.unwrap().1.contains(x));
                    if !JoinForestNode::walk_on_path(
                        &parent.unwrap(),
                        path.clone(),
                        value,
                        atom_pairs.clone(),
                    ) {
                        // If we weren't able to insert the node, we check if there is a pair with the current node and the node we want to insert
                        for pair in &atom_pairs {
                            // All the values in path have to be contained in pair.2
                            if pair.0 == value && pair.1 == &start_borrowed.value
                                || pair.1 == value
                                    && pair.0 == &start_borrowed.value
                                    && path.iter().all(|x| pair.2.contains(x))
                            {
                                // If all the values in path are contained in pair.2, I insert the node with pair.2 variables as common variables
                                start_borrowed.add_child((
                                    Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                                    pair.2.clone(),
                                ));
                                return true;
                            }
                        }
                        return false;
                    }
                    return true;
                } else {
                    // If If the parent doesn't contain at least one variable in the path, check the current node
                    for pair in &atom_pairs {
                        // All the values in path have to be contained in pair.2
                        if pair.0 == value && pair.1 == &start_borrowed.value
                            || pair.1 == value
                                && pair.0 == &start_borrowed.value
                                && path.iter().all(|x| pair.2.contains(x))
                        {
                            // If all the values in path are contained in pair.2, I insert the node with pair.2 variables as common variables
                            start_borrowed.add_child((
                                Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                                pair.2.clone(),
                            ));
                            return true;
                        }
                    }
                    return false;
                }
            } else if start_borrowed.parent.is_none() {
                // If the node has no parent, check all the children
                for child in &start_borrowed.children {
                    let mut path_copy = path.clone();
                    // Check that at least one child has variables contained in path
                    if child.1.iter().any(|x| path.contains(x)) {
                        // If so go to the child consuming variables in the path
                        path_copy.retain(|x| !child.1.contains(x));
                        if JoinForestNode::walk_on_path(child, path_copy, value, atom_pairs.clone())
                        {
                            return true;
                        }
                    }
                }
                // If we weren't able to insert the node in one child, check the current node
                for pair in &atom_pairs {
                    // All the values in path have to be contained in pair.2
                    if pair.0 == value && pair.1 == &start_borrowed.value
                        || pair.1 == value
                            && pair.0 == &start_borrowed.value
                            && path.iter().all(|x| pair.2.contains(x))
                    {
                        // If all the values in path are contained in pair.2, I insert the node with pair.2 variables as common variables
                        start_borrowed.add_child((
                            Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                            pair.2.clone(),
                        ));
                        return true;
                    }
                }
                return false;
            } else {
                // The node has both parents and children, check the children first and the parent last with the same logic implemented in the previous cases
                for child in &start_borrowed.children {
                    let mut path_copy = path.clone();
                    if child.1.iter().any(|x| path.contains(x)) {
                        path_copy.retain(|x| !child.1.contains(x));
                        if JoinForestNode::walk_on_path(child, path_copy, value, atom_pairs.clone())
                        {
                            return true;
                        }
                    }
                }
                let parent = start_borrowed.parent.as_ref();
                if parent.unwrap().1.iter().any(|x| path.contains(x)) {
                    path.retain(|x| !parent.unwrap().1.contains(x));
                    if !JoinForestNode::walk_on_path(
                        &parent.unwrap(),
                        path.clone(),
                        value,
                        atom_pairs.clone(),
                    ) {
                        for pair in &atom_pairs {
                            if pair.0 == value && pair.1 == &start_borrowed.value
                                || pair.1 == value
                                    && pair.0 == &start_borrowed.value
                                    && path.iter().all(|x| pair.2.contains(x))
                            {
                                start_borrowed.add_child((
                                    Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                                    pair.2.clone(),
                                ));
                                return true;
                            }
                        }
                        return false;
                    }
                    return true;
                } else {
                    for pair in &atom_pairs {
                        if pair.0 == value && pair.1 == &start_borrowed.value
                            || pair.1 == value
                                && pair.0 == &start_borrowed.value
                                && path.iter().all(|x| pair.2.contains(x))
                        {
                            start_borrowed.add_child((
                                Rc::new(RefCell::new(JoinForestNode::new(value.clone()))),
                                pair.2.clone(),
                            ));
                            return true;
                        }
                    }
                    return false;
                }
            }
        }
    }

    pub fn post_order_wrapper(&self) -> Vec<Rc<RefCell<JoinForestNode>>> {
        let mut result = Vec::new();
        for child in &self.children {
            JoinForestNode::post_order(child, &mut result);
        }
        result.push(Rc::new(RefCell::new(self.clone())));
        result
    }

    fn post_order(
        start: &(Rc<RefCell<JoinForestNode>>, Vec<String>),
        result_vec: &mut Vec<Rc<RefCell<JoinForestNode>>>,
    ) {
        let start_borrowed = start.0.borrow();
        if start_borrowed.children.len() == 0 {
            result_vec.push(Rc::clone(&start.0));
            return;
        }
        for child in &start_borrowed.children {
            JoinForestNode::post_order(child, result_vec);
        }
        result_vec.push(Rc::clone(&start.0));
        return;
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<JoinForestNode>>> {
        let mut result = Vec::new();
        for child in &self.children {
            result.push(Rc::clone(&child.0));
        }
        result
    }

    pub fn get_relation_name(&self) -> String {
        return self.value.get_name();
    }

    pub fn get_variables(&self) -> Vec<String> {
        return self.value.get_variables();
    }

    pub fn print(&self) {
        if self.children.len() == 0 {
            print!("Leaf: ");
        }
        if self.parent.is_none() {
            print!("Root: ");
        }
        self.value.print();
        if self.parent.is_some() {
            print!(" Parent: ");
            self.parent.as_ref().unwrap().0.borrow().value.print();
            print!(" Values shared with parent: ");
            print!("[ ");
            let mut i = 0;
            let tmp = &self.parent.as_ref().unwrap().1;
            for element in tmp {
                print!("{}", element);
                if i == (tmp.len() - 1) {
                    print!(" ");
                } else {
                    print!(", ");
                }
                i += 1;
            }
            print!("]")
        }
        println!();
        if self.children.len() != 0 {
            println!("Children:");
            for child in &self.children {
                child.0.borrow().print();
            }
        } else {
            println!("----------");
        }
    }
}

impl JoinForest {
    pub fn new(cq: &ConjunctiveQuery) -> JoinForest {
        let mut result = JoinForest { roots: Vec::new() };

        let atoms_list = cq.atoms();
        let mut atom_pairs: Vec<(&Atom, &Atom, Vec<String>)> = Vec::new();
        for i in 0..atoms_list.len() {
            let atom = &atoms_list[i];
            for j in (i + 1)..atoms_list.len() {
                let atom_to_compare = &atoms_list[j];
                let intersection = atom.intersect(atom_to_compare);
                if intersection.len() != 0 {
                    atom_pairs.push((atom, atom_to_compare, intersection));
                }
            }
        }

        if DEBUG {
            for pair in &atom_pairs {
                pair.0.print();
                print!(" - ");
                pair.1.print();
                print!(" - [ ");
                let mut i = 0;
                for element in &pair.2 {
                    print!("{}", element);
                    if i == (pair.2.len() - 1) {
                        print!(" ");
                    } else {
                        print!(", ");
                    }
                    i += 1;
                }
                println!("]")
            }
        }

        /*
           Check if exists an atom that shares some variables with all the other atoms.
           If so set it as root node, if not the conjunctive query can be represented as a forest only, so set the root nodes accordingly.
           For the tree case:
               Reorder the atom pairs list so that all the pairs with the root atom are first with the root atom as first element in the pair.
               For all the pairs available:
                   Check if in the tree exists a path from the first pair node that allows you to consume some variables in the intersection, if not insert the atom as a child of the first pair node and set the intersection to the pair intersection.
                   If a path that consumes some variable exists then check for a pair with both the returned node and the atom we are analyzing.
                       If such pair doesn't exist add the atom as a child of the current node.
                       If such pair exists check if the pair contains in its intersection all variables that have not been consumed during the path to the node.
                           If it contains all the non-consumed variables or all variables has been consumed then insert the atom as a child of the returned node with intersection given by the pair intersection.
                           If not insert the atom as a child of the current node.
           For the forest case:
               Find the roots collecting into different list all the pairs belonging to different trees.
               Inside each tree find the atom that shares some variables with all the other atom.
               For each tree apply the tree case algorithm with the root and the associated pairs.
        */

        // Check if exists an atom that shares some variables with all the other atoms.
        let atoms_list = atoms_list.clone();
        let mut atom_occurrencies_counter = Vec::new();
        for atom in &atoms_list {
            atom_occurrencies_counter.push((atom.clone(), 0));
        }

        let atoms_list = atoms_list.clone();
        let mut root: Option<JoinForestNode> = None;
        for element in &mut atom_occurrencies_counter {
            for pair in &atom_pairs {
                if element.0 == *pair.0 || element.0 == *pair.1 {
                    element.1 += 1;
                }
                if element.1 == (atoms_list.len() - 1) {
                    root = Some(JoinForestNode {
                        value: element.0.clone(),
                        children: Vec::new(),
                        parent: None,
                    });
                    break;
                }
            }
        }

        let tree_function =
            |root: Rc<RefCell<JoinForestNode>>, atom_pairs: Vec<(&Atom, &Atom, Vec<String>)>| {
                /*
                    Sort the pairs in order to have all the pairs with the root at the beginning and "on the left" as first element in the pair.
                    In this way I'm sure that all the atoms at the first position in the pair have been already inserted in the tree.
                */
                let mut atom_pairs_sorted = Vec::new();
                for pair in &atom_pairs {
                    if *pair.0 == root.borrow().value {
                        atom_pairs_sorted.push(pair.clone());
                    } else if *pair.1 == root.borrow().value {
                        let tmp = (pair.1, pair.0, pair.2.clone());
                        atom_pairs_sorted.push(tmp);
                    }
                }
                for pair in &atom_pairs {
                    if *pair.0 != root.borrow().value && *pair.1 != root.borrow().value {
                        atom_pairs_sorted.push(pair.clone());
                    }
                }

                if DEBUG {
                    println!();
                    for pair in &atom_pairs_sorted {
                        pair.0.print();
                        print!(" - ");
                        pair.1.print();
                        print!(" - [ ");
                        let mut i = 0;
                        for element in &pair.2 {
                            print!("{}", element);
                            if i == (pair.2.len() - 1) {
                                print!(" ");
                            } else {
                                print!(", ");
                            }
                            i += 1;
                        }
                        println!("]")
                    }
                }

                for pair in &atom_pairs_sorted {
                    // For all the pairs available:
                    // Check if in the tree exists a path from the first pair node that allows you to consume some variables in the intersection
                    let node2_tmp = JoinForestNode::search_wrapper(Rc::clone(&root), pair.1);
                    let node2 = node2_tmp.as_ref();
                    if node2.is_none() {
                        let node_tmp = JoinForestNode::search_wrapper(Rc::clone(&root), pair.0);
                        let node1 = node_tmp.as_ref();
                        JoinForestNode::walk_on_path_wrapper(
                            Rc::clone(node1.unwrap()),
                            pair.2.clone(),
                            pair.1,
                            atom_pairs_sorted.clone(),
                        );
                    }
                }
            };

        if root.is_none() {
            // Forest case
            atom_occurrencies_counter.sort_by(|a, b| b.1.cmp(&a.1));
            if DEBUG {
                for atom_occurrence in &atom_occurrencies_counter {
                    atom_occurrence.0.print();
                    println!(" : {}", atom_occurrence.1);
                }
            }
            let mut atom_pairs_copy = atom_pairs.clone();
            let mut i = 0;
            while atom_pairs_copy.len() != 0 {
                let selected_atom = atom_occurrencies_counter[i].0.clone();
                if atom_pairs_copy
                    .iter()
                    .any(|x| x.0 == &selected_atom || x.1 == &selected_atom)
                {
                    result.roots.push(Rc::new(RefCell::new(JoinForestNode::new(
                        selected_atom.clone(),
                    ))));
                    // Check all the atoms linked by one pair to the selected atom
                    let mut atoms_linked_with_selected_atom = Vec::new();
                    for atom in &atom_pairs {
                        if atom.0 == &selected_atom {
                            atoms_linked_with_selected_atom.push(atom.1);
                        }
                        if atom.1 == &selected_atom {
                            atoms_linked_with_selected_atom.push(atom.0);
                        }
                    }
                    // Delete all the pairs that contains the selected atom or one of the atoms linked to it
                    atom_pairs_copy.retain(|x| {
                        x.0 != &selected_atom
                            && x.1 != &selected_atom
                            && !atoms_linked_with_selected_atom.contains(&x.0)
                            && !atoms_linked_with_selected_atom.contains(&x.1)
                    });
                }
                i += 1;
            }

            // Check for isolated nodes
            for atom_occurrence in &atom_occurrencies_counter {
                if atom_occurrence.1 == 0 {
                    result.roots.push(Rc::new(RefCell::new(JoinForestNode::new(
                        atom_occurrence.0.clone(),
                    ))));
                }
            }

            // Run the tree algorithm for each root
            for root in &result.roots {
                // First, find all the atoms linked with the current root
                let mut atoms_linked_with_root = Vec::new();
                for atom in &atom_pairs {
                    if *atom.0 == root.borrow().value {
                        atoms_linked_with_root.push(atom.1);
                    }
                    if *atom.1 == root.borrow().value {
                        atoms_linked_with_root.push(atom.0);
                    }
                }
                if DEBUG {
                    println!("Atoms linked with root: ");
                    for atom in &atoms_linked_with_root {
                        atom.print();
                        println!();
                    }

                    let tmp: Vec<(&Atom, &Atom, Vec<String>)> = atom_pairs
                        .clone()
                        .into_iter()
                        .filter(|x| {
                            x.0 == &root.borrow().value
                                || x.1 == &root.borrow().value
                                || atoms_linked_with_root.contains(&x.0)
                                || atoms_linked_with_root.contains(&x.1)
                        })
                        .collect();
                    for pair in tmp {
                        pair.0.print();
                        print!(" - ");
                        pair.1.print();
                        print!(" - [ ");
                        let mut i = 0;
                        for element in &pair.2 {
                            print!("{}", element);
                            if i == (pair.2.len() - 1) {
                                print!(" ");
                            } else {
                                print!(", ");
                            }
                            i += 1;
                        }
                        println!("]")
                    }
                }

                // Second run the algorithm using all the pairs that contains the root or one node linked with the root
                tree_function(
                    Rc::clone(root),
                    atom_pairs
                        .clone()
                        .into_iter()
                        .filter(|x| {
                            x.0 == &root.borrow().value
                                || x.1 == &root.borrow().value
                                || atoms_linked_with_root.contains(&x.0)
                                || atoms_linked_with_root.contains(&x.1)
                        })
                        .collect(),
                );
            }
        } else {
            // Tree case
            // Reorder the atom pairs list so that all the pairs with the root atom are first with the root atom as first element in the pair.
            result.roots.push(Rc::new(RefCell::new(root.unwrap())));
            tree_function(Rc::clone(&result.roots[0]), atom_pairs.clone());
        }
        result
    }

    pub fn get_roots(&self) -> Vec<Rc<RefCell<JoinForestNode>>> {
        return self.roots.clone();
    }

    pub fn print(&self) {
        println!("Join forest:");
        for root in &self.roots {
            root.borrow().print();
        }
    }
}
