// SPDX-License-Identifier: GPL-3.0-or-later
//! Cruncha's reactive dependency graph (design 02 §3.1).
//!
//! Cells are nodes; a cell that reads a name depends on the one cell that defines it.
//! [`Graph::new`] takes the cells in document order and works out:
//!
//! - an **evaluation order** that follows dependencies, not position (ties keep document
//!   order, so independent cells run top to bottom);
//! - **issues**: a name defined by two cells (rule 1: an error on both), cycles (rule 4: an
//!   error on every cell in the cycle), and a name used above its definition (rule 3: a lint,
//!   not an error);
//! - which cells an edit **invalidates** ([`Graph::affected`]): the edited cells and everything
//!   downstream of them, including readers of names that were renamed away (rule 5).
//!
//! The graph knows nothing about values or languages, so calc, Python and Rust cells share it.

use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::sync::Arc;

/// One cell as the graph sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub defines: Vec<String>,
    pub reads: Vec<String>,
}

impl Node {
    pub fn new(id: impl Into<String>, defines: &[&str], reads: &[&str]) -> Node {
        Node {
            id: id.into(),
            defines: defines.iter().map(|s| s.to_string()).collect(),
            reads: reads.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Something wrong with a cell's place in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Issue {
    /// `name` is also defined by the cells `others`. An error: the cell is not evaluated, and
    /// readers of `name` can't be either.
    DuplicateDefinition { name: String, others: Vec<String> },
    /// The cell is part of a dependency cycle through `cycle` (cell ids, in document order;
    /// shared by every member).
    Cycle { cycle: Arc<[String]> },
    /// Reads `name`, which `definer` defines further down the document. A lint only.
    UsedAboveDefinition { name: String, definer: String },
    /// Reads `name`, whose definition is in error (duplicated, or part of a cycle).
    BrokenInput { name: String },
}

impl Issue {
    /// Errors stop a cell from being evaluated; lints don't.
    pub fn is_error(&self) -> bool {
        !matches!(self, Issue::UsedAboveDefinition { .. })
    }
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Issue::DuplicateDefinition { name, others } => {
                let n = others.len();
                write!(
                    f,
                    "{name} is defined in {} other cell{}; every name needs exactly one definition",
                    n,
                    if n == 1 { "" } else { "s" }
                )
            }
            Issue::Cycle { cycle } => {
                write!(f, "circular dependency through {} cells", cycle.len())
            }
            Issue::UsedAboveDefinition { name, .. } => {
                write!(f, "{name} is used here but defined further down")
            }
            Issue::BrokenInput { name } => write!(f, "{name} has an error where it is defined"),
        }
    }
}

/// The dependency graph of a document.
#[derive(Debug, Clone)]
pub struct Graph {
    nodes: Vec<Node>,
    index: HashMap<String, usize>,
    /// For each name, the cells that define it (in document order).
    definers: HashMap<String, Vec<usize>>,
    /// Edges: `deps[i]` are the cells `i` reads from (each a unique definer).
    deps: Vec<BTreeSet<usize>>,
    readers: Vec<BTreeSet<usize>>,
    order: Vec<usize>,
    issues: Vec<Vec<Issue>>,
}

impl Graph {
    /// Build the graph for `nodes` in document order. Ids must be unique.
    pub fn new(nodes: Vec<Node>) -> Graph {
        let n = nodes.len();
        let index: HashMap<String, usize> =
            nodes.iter().enumerate().map(|(i, c)| (c.id.clone(), i)).collect();
        let mut definers: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, c) in nodes.iter().enumerate() {
            for name in unique(&c.defines) {
                definers.entry(name.clone()).or_default().push(i);
            }
        }
        let mut issues: Vec<Vec<Issue>> = vec![Vec::new(); n];
        let mut broken = vec![false; n];

        // Rule 1: one definition per name.
        for (name, ds) in &definers {
            if ds.len() > 1 {
                for &d in ds {
                    let others = ds
                        .iter()
                        .filter(|&&o| o != d)
                        .map(|&o| nodes[o].id.clone())
                        .collect();
                    issues[d].push(Issue::DuplicateDefinition {
                        name: name.clone(),
                        others,
                    });
                    broken[d] = true;
                }
            }
        }

        // Edges from readers to the unique definer of each name read (not to themselves).
        let mut deps = vec![BTreeSet::new(); n];
        let mut readers = vec![BTreeSet::new(); n];
        for (i, c) in nodes.iter().enumerate() {
            for name in unique(&c.reads) {
                if let Some(ds) = definers.get(name)
                    && ds.len() == 1
                    && ds[0] != i
                {
                    deps[i].insert(ds[0]);
                    readers[ds[0]].insert(i);
                }
            }
        }

        // Rule 4: cycles (Tarjan's strongly connected components).
        for scc in sccs(&deps) {
            let cyclic = scc.len() > 1;
            if cyclic {
                let mut ids: Vec<usize> = scc.clone();
                ids.sort_unstable();
                let cycle: Arc<[String]> = ids.iter().map(|&i| nodes[i].id.clone()).collect();
                for &i in &scc {
                    issues[i].push(Issue::Cycle { cycle: cycle.clone() });
                    broken[i] = true;
                }
            }
        }

        // Evaluation order: Kahn's algorithm, lowest document index first among ready cells.
        // Cells in cycles never become ready; they are appended at the end (and not evaluated).
        let mut indegree: Vec<usize> = deps.iter().map(BTreeSet::len).collect();
        let mut ready: BinaryHeap<Reverse<usize>> =
            (0..n).filter(|&i| indegree[i] == 0).map(Reverse).collect();
        let mut order = Vec::with_capacity(n);
        let mut placed = vec![false; n];
        while let Some(Reverse(i)) = ready.pop() {
            order.push(i);
            placed[i] = true;
            for &r in &readers[i] {
                indegree[r] -= 1;
                if indegree[r] == 0 {
                    ready.push(Reverse(r));
                }
            }
        }
        order.extend((0..n).filter(|&i| !placed[i]));

        // Readers of broken definitions, transitively, can't be evaluated either.
        for &i in &order {
            if broken[i] {
                continue;
            }
            let bad: Vec<String> = unique(&nodes[i].reads)
                .filter(|name| {
                    definers.get(*name).is_some_and(|ds| {
                        ds.len() > 1 || ds.iter().any(|&d| d != i && broken[d])
                    })
                })
                .cloned()
                .collect();
            if !bad.is_empty() {
                broken[i] = true;
                issues[i].extend(bad.into_iter().map(|name| Issue::BrokenInput { name }));
            }
        }

        // Rule 3: used above its definition (a lint).
        for (i, c) in nodes.iter().enumerate() {
            for name in unique(&c.reads) {
                if let Some(ds) = definers.get(name)
                    && ds.len() == 1
                    && ds[0] > i
                {
                    issues[i].push(Issue::UsedAboveDefinition {
                        name: name.clone(),
                        definer: nodes[ds[0]].id.clone(),
                    });
                }
            }
        }

        Graph {
            nodes,
            index,
            definers,
            deps,
            readers,
            order,
            issues,
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Cell ids in evaluation order. Cells with errors are included; check [`Graph::can_run`].
    pub fn order(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(|&i| self.nodes[i].id.as_str())
    }

    /// The issues of cell `id` (empty if none, or if the id is unknown).
    pub fn issues(&self, id: &str) -> &[Issue] {
        self.index.get(id).map_or(&[], |&i| &self.issues[i])
    }

    /// Whether cell `id` can be evaluated (no errors, only lints).
    pub fn can_run(&self, id: &str) -> bool {
        self.issues(id).iter().all(|i| !i.is_error())
    }

    /// The cell that defines `name`, if exactly one does.
    pub fn definer(&self, name: &str) -> Option<&str> {
        match self.definers.get(name).map(Vec::as_slice) {
            Some([d]) => Some(self.nodes[*d].id.as_str()),
            _ => None,
        }
    }

    /// The cells `id` reads from.
    pub fn inputs(&self, id: &str) -> impl Iterator<Item = &str> {
        let deps = self.index.get(id).map(|&i| &self.deps[i]);
        deps.into_iter().flatten().map(|&d| self.nodes[d].id.as_str())
    }

    /// Cells to re-evaluate after the cells in `changed` were edited, in evaluation order: the
    /// changed cells that still exist, and everything downstream of them. `previous` is the
    /// graph before the edit: readers of names that changed definer (renamed, deleted or newly
    /// duplicated) are included too, so nothing keeps a stale value (rule 5).
    pub fn affected(&self, changed: &[&str], previous: Option<&Graph>) -> Vec<&str> {
        let mut start: HashSet<usize> = changed
            .iter()
            .filter_map(|id| self.index.get(*id).copied())
            .collect();
        if let Some(prev) = previous {
            let mut names: HashSet<&str> = HashSet::new();
            for id in changed {
                if let Some(&p) = prev.index.get(*id) {
                    names.extend(prev.nodes[p].defines.iter().map(String::as_str));
                }
                if let Some(&i) = self.index.get(*id) {
                    names.extend(self.nodes[i].defines.iter().map(String::as_str));
                }
            }
            for (i, c) in self.nodes.iter().enumerate() {
                if c.reads.iter().any(|r| names.contains(r.as_str())) {
                    start.insert(i);
                }
            }
        }
        let mut seen = vec![false; self.nodes.len()];
        let mut stack: Vec<usize> = start.into_iter().collect();
        while let Some(i) = stack.pop() {
            if std::mem::replace(&mut seen[i], true) {
                continue;
            }
            stack.extend(self.readers[i].iter().copied());
        }
        self.order
            .iter()
            .filter(|&&i| seen[i])
            .map(|&i| self.nodes[i].id.as_str())
            .collect()
    }
}

fn unique(names: &[String]) -> impl Iterator<Item = &String> {
    let mut seen = HashSet::new();
    names.iter().filter(move |n| seen.insert(n.as_str()))
}

/// Strongly connected components (iterative Tarjan). Each component is a list of node indices.
fn sccs(deps: &[BTreeSet<usize>]) -> Vec<Vec<usize>> {
    let n = deps.len();
    let mut index = vec![usize::MAX; n];
    let mut low = vec![0; n];
    let mut on_stack = vec![false; n];
    let mut stack = Vec::new();
    let mut out = Vec::new();
    let mut next = 0;
    for root in 0..n {
        if index[root] != usize::MAX {
            continue;
        }
        // (node, iterator position over its deps)
        let mut work: Vec<(usize, Vec<usize>, usize)> = Vec::new();
        index[root] = next;
        low[root] = next;
        next += 1;
        stack.push(root);
        on_stack[root] = true;
        work.push((root, deps[root].iter().copied().collect(), 0));
        while let Some((v, succ, pos)) = work.last_mut() {
            let v = *v;
            if *pos < succ.len() {
                let w = succ[*pos];
                *pos += 1;
                if index[w] == usize::MAX {
                    index[w] = next;
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    work.push((w, deps[w].iter().copied().collect(), 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                work.pop();
                if let Some((parent, _, _)) = work.last() {
                    low[*parent] = low[*parent].min(low[v]);
                }
                if low[v] == index[v] {
                    let mut comp = Vec::new();
                    loop {
                        let w = stack.pop().expect("tarjan stack");
                        on_stack[w] = false;
                        comp.push(w);
                        if w == v {
                            break;
                        }
                    }
                    out.push(comp);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(g: &Graph) -> Vec<&str> {
        g.order().collect()
    }

    #[test]
    fn follows_dependencies_then_document_order() {
        // M reads w and L; L is defined below M (allowed, with a lint).
        let g = Graph::new(vec![
            Node::new("W", &["w"], &[]),
            Node::new("M", &["M"], &["w", "L"]),
            Node::new("L", &["L"], &[]),
            Node::new("T", &[], &[]),
        ]);
        assert_eq!(order(&g), ["W", "L", "M", "T"]);
        assert!(g.can_run("M"));
        assert_eq!(
            g.issues("M"),
            [Issue::UsedAboveDefinition { name: "L".into(), definer: "L".into() }]
        );
        assert_eq!(g.definer("w"), Some("W"));
        assert_eq!(g.inputs("M").collect::<Vec<_>>(), ["W", "L"]);
    }

    #[test]
    fn duplicate_definitions_are_errors_on_both_and_block_readers() {
        let g = Graph::new(vec![
            Node::new("A", &["x"], &[]),
            Node::new("B", &["x"], &[]),
            Node::new("C", &["y"], &["x"]),
            Node::new("D", &["z"], &["y"]),
            Node::new("E", &["e"], &[]),
        ]);
        assert!(!g.can_run("A") && !g.can_run("B"));
        assert!(matches!(&g.issues("A")[0], Issue::DuplicateDefinition { others, .. } if others == &["B"]));
        assert_eq!(g.issues("C"), [Issue::BrokenInput { name: "x".into() }]);
        assert_eq!(g.issues("D"), [Issue::BrokenInput { name: "y".into() }]);
        assert!(g.can_run("E"));
        assert_eq!(g.definer("x"), None);
        assert!(g.issues("A")[0].to_string().contains("1 other cell;"));
    }

    #[test]
    fn cycles_are_errors_on_every_member() {
        let g = Graph::new(vec![
            Node::new("A", &["a"], &["c"]),
            Node::new("B", &["b"], &["a"]),
            Node::new("C", &["c"], &["b"]),
            Node::new("D", &["d"], &["c"]),
            Node::new("S", &["s"], &["s"]), // reading your own name is not a cycle edge
        ]);
        for id in ["A", "B", "C"] {
            assert!(!g.can_run(id), "{id}");
            assert!(g.issues(id).iter().any(|i| matches!(i, Issue::Cycle { cycle } if **cycle == ["A", "B", "C"])));
        }
        assert_eq!(g.issues("D"), [Issue::BrokenInput { name: "c".into() }]);
        assert!(g.can_run("S"));
        assert_eq!(order(&g).len(), 5);
    }

    #[test]
    fn affected_is_downstream_in_order_and_catches_renames() {
        let nodes = vec![
            Node::new("W", &["w"], &[]),
            Node::new("L", &["L"], &[]),
            Node::new("M", &["M"], &["w", "L"]),
            Node::new("S", &["S"], &["M"]),
            Node::new("X", &["x"], &[]),
        ];
        let before = Graph::new(nodes.clone());
        assert_eq!(before.affected(&["L"], None), ["L", "M", "S"]);
        assert_eq!(before.affected(&["X"], None), ["X"]);

        // Rename L to span: M still reads L, which no longer exists, so M must re-run.
        let mut renamed = nodes.clone();
        renamed[1] = Node::new("L", &["span"], &[]);
        let after = Graph::new(renamed);
        assert_eq!(after.affected(&["L"], Some(&before)), ["L", "M", "S"]);

        // Deleting W: its readers re-run (and will report w undefined).
        let deleted = Graph::new(nodes.iter().filter(|n| n.id != "W").cloned().collect());
        assert_eq!(deleted.affected(&["W"], Some(&before)), ["M", "S"]);
    }

    #[test]
    fn large_chains_are_fine() {
        let n = 20_000;
        let nodes: Vec<Node> = (0..n)
            .map(|i| {
                let reads = if i == 0 { vec![] } else { vec![format!("v{}", i - 1)] };
                Node { id: format!("c{i}"), defines: vec![format!("v{i}")], reads }
            })
            .collect();
        let g = Graph::new(nodes);
        assert_eq!(g.order().count(), n);
        assert_eq!(g.affected(&["c0"], None).len(), n);
        // A cycle closing the chain.
        let mut nodes: Vec<Node> = g.nodes().to_vec();
        nodes[0].reads = vec![format!("v{}", n - 1)];
        let g = Graph::new(nodes);
        assert!(!g.can_run("c123"));
    }
}
