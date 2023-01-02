# Conjunctive query algorithms and data structures
In this project some algorithms and data structures used for conjunctive queries evaluation on databases have been implemented using Rust.</br>
Some simplifications have been made:
- The database, the input query is executed on, is automatically generated, so that it matches the conjunctive queries atoms.
- All atoms in conjunctive queries have to be named using different names, because in the program each atom corresponds to a relation in the database.

## Program features:
- Parsing a string representing a conjunctive query.
- Building the hypergraph associated with a conjunctive query.
- Implementation of Graham-Yu-Ozsoyoglu (GYO) algorithm to check if an hypergraph is $\alpha$-acyclic.
- Implementation of an algorithm to build a join forest out of an $\alpha$-acyclic conjunctive query.
- Random data generation for a specific conjunctive query.
- Simplified version of Yannakakis algorithm for $\alpha$-acyclic conjunctive query evaluation.
- Implementation of a simplified version of classic hash join algorithm for natural join operations on tables.
- Implementation of standard database operations like projection and selection.