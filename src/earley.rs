use crate::tree::Tree;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
    hash::Hash,
};

pub trait Terminal: Clone + Copy + Eq + Hash {}

impl<T> Terminal for T where T: Clone + Copy + Eq + Hash {}

pub trait Nonterminal: Clone + Copy + Eq + Hash {
    /// Returns the starting non-terminal
    fn start() -> Self;
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Symbol<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    Nonterminal(N),
    Terminal(T),
}

impl<N, T> Display for Symbol<N, T>
where
    N: Nonterminal + Display,
    T: Terminal + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Terminal(s) => f.pad(&format!("{}", s)),
            Symbol::Nonterminal(s) => f.pad(&format!("{}", s)),
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Production<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    lhs: N,
    rhs: Vec<Symbol<N, T>>,
}

impl<N, T> Production<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    pub fn new(lhs: N, rhs: Vec<Symbol<N, T>>) -> Self {
        Self { lhs, rhs }
    }
    fn into_dotted_rule(self, dot_pos: usize) -> DottedRule<N, T> {
        if dot_pos > self.rhs.len() {
            panic!(
                "Attempted to create dotted rule with dot_pos={}, but rhs.len()={}",
                dot_pos,
                self.rhs.len()
            );
        }

        DottedRule {
            production: self,
            dot_pos,
        }
    }

    pub fn lhs(&self) -> &N {
        &self.lhs
    }

    pub fn rhs(&self) -> &Vec<Symbol<N, T>> {
        &self.rhs
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct DottedRule<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    production: Production<N, T>,
    dot_pos: usize,
}

impl<N, T> DottedRule<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    fn next_symbol(&self) -> Option<Symbol<N, T>> {
        if self.dot_pos >= self.production.rhs.len() {
            None
        } else {
            Some(self.production.rhs[self.dot_pos].clone())
        }
    }

    fn advanced_dot(mut self) -> Self {
        if self.dot_pos >= self.production.rhs.len() {
            panic!("Attempted to advance dot when nowhere to advance to");
        }
        self.dot_pos += 1;
        self
    }

    pub fn is_complete(&self) -> bool {
        self.dot_pos >= self.production.rhs.len()
    }

    pub fn production(&self) -> &Production<N, T> {
        &self.production
    }
}

impl<N, T> Display for DottedRule<N, T>
where
    N: Nonterminal + Display,
    T: Terminal + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!("{:2} ->", self.production.lhs);
        for (i, sym) in self.production.rhs.iter().enumerate() {
            s.push_str(&format!(
                "{}{}",
                if i == self.dot_pos { "•" } else { " " },
                sym
            ));
        }
        if self.dot_pos == self.production.rhs.len() {
            s.push_str("•");
        }

        f.pad(&s)?;
        Ok(())
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ChartEdge<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    dotted_rule: DottedRule<N, T>,
    start: usize,
    end: usize,
    history: Vec<ChartEdge<N, T>>,
}

impl<N, T> ChartEdge<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    pub fn dotted_rule(&self) -> &DottedRule<N, T> {
        &self.dotted_rule
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn history(&self) -> &Vec<ChartEdge<N, T>> {
        &self.history
    }

    pub fn generate_derivation_tree(&self) -> Tree<Symbol<N, T>> {
        let mut children: Vec<Tree<Symbol<N, T>>> = self
            .history()
            .into_iter()
            .map(Self::generate_derivation_tree)
            .collect();

        for sym in self.dotted_rule().production().rhs() {
            if let Symbol::Terminal(t) = sym {
                children.push(Tree::new(Symbol::Terminal(t.clone()), vec![]))
            }
        }

        Tree::new(
            Symbol::Nonterminal(self.dotted_rule().production().lhs().clone()),
            children,
        )
    }
}

pub struct Chart<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    /// String to parse
    input_string: Vec<T>,
    /// Maps from nonterminals to its productions
    productions_by_lhs: HashMap<N, Vec<Production<N, T>>>,
    /// All edges in a set for quick member check
    all_edges: HashSet<ChartEdge<N, T>>,
    /// Edges left to predict/scan/complete
    to_process: VecDeque<ChartEdge<N, T>>,

    /// Complete derivations stored here
    complete_derivations: Vec<ChartEdge<N, T>>,

    /// Entire chart in order (mainly just for printing it out),
    /// the second item in the pair is the history in the form of indices
    /// back into this `Vec`, as this is easier to print in a table.
    /// This will only be populated if trace is true
    trace_chart: Vec<(ChartEdge<N, T>, Vec<usize>)>,
    trace: bool,
}

impl<N, T> Chart<N, T>
where
    N: Nonterminal,
    T: Terminal,
{
    /// Create new chart
    pub fn new(input_string: Vec<T>, productions: Vec<Production<N, T>>) -> Self {
        let mut productions_by_lhs = HashMap::new();
        let mut to_process = VecDeque::new();
        let mut all_edges = HashSet::new();

        for production in productions {
            let prods = productions_by_lhs
                .entry(production.lhs)
                .or_insert_with(Vec::new);

            prods.push(production.clone());
        }

        for production in productions_by_lhs
            .get(&N::start())
            .expect("No starting productions")
        {
            let edge = ChartEdge {
                dotted_rule: production.clone().into_dotted_rule(0),
                start: 0,
                end: 0,
                history: Vec::new(),
            };
            to_process.push_back(edge.clone());
            all_edges.insert(edge);
        }

        Self {
            input_string: input_string,
            productions_by_lhs,
            all_edges,
            to_process,
            trace_chart: Vec::new(),
            trace: false,
            complete_derivations: Vec::new(),
        }
    }

    pub fn set_trace(&mut self, trace: bool) {
        self.trace = trace;
    }

    fn add_to_trace_chart(&mut self, edge: &ChartEdge<N, T>) {
        if self.trace {
            let history: Vec<usize> = edge
                .history()
                .iter()
                .map(|e| {
                    for (j, (oe, _)) in self.trace_chart.iter().enumerate() {
                        if e == oe {
                            return j;
                        }
                    }
                    return usize::MAX;
                })
                .collect();

            self.trace_chart.push((edge.clone(), history));
        }
    }

    pub fn trace_chart(&self) -> &Vec<(ChartEdge<N, T>, Vec<usize>)> {
        &self.trace_chart
    }

    pub fn process_all(&mut self) {
        while self.more_to_process() {
            self.process_one();
        }
    }

    pub fn more_to_process(&self) -> bool {
        !self.to_process.is_empty()
    }

    /// Processes one edge from to_process. Panics if nothing to do.
    pub fn process_one(&mut self) -> ChartEdge<N, T> {
        if let Some(edge) = self.to_process.pop_front() {
            match edge.dotted_rule.next_symbol() {
                // Predict
                Some(Symbol::Nonterminal(nonterminal)) => {
                    let productions = self
                        .productions_by_lhs
                        .get(&nonterminal)
                        .expect("Expected non-terminal to have a production");

                    let new_edges: Vec<ChartEdge<N, T>> = productions
                        .iter()
                        .map(|production| ChartEdge {
                            dotted_rule: production.clone().into_dotted_rule(0),
                            start: edge.end,
                            end: edge.end,
                            history: Vec::new(),
                        })
                        .collect();

                    self.add_edges(new_edges);
                }
                // Scan
                Some(Symbol::Terminal(terminal)) => {
                    if self.input_string.get(edge.end) == Some(&terminal) {
                        let new_edge = ChartEdge {
                            dotted_rule: edge.dotted_rule.clone().advanced_dot(),
                            start: edge.start,
                            end: edge.end + 1,
                            history: Vec::new(),
                        };

                        self.add_edge(new_edge);
                    }
                }
                // Complete
                None => {
                    let completed_nonterminal = edge.dotted_rule.production.lhs;

                    if completed_nonterminal == N::start()
                        && edge.start() == 0
                        && edge.end() == self.input_string.len()
                    {
                        self.complete_derivations.push(edge.clone());
                    }

                    let new_edges: Vec<ChartEdge<N, T>> = self
                        .all_edges
                        .iter()
                        .filter_map(|other_edge| {
                            if Some(Symbol::Nonterminal(completed_nonterminal))
                                == other_edge.dotted_rule.next_symbol()
                                && other_edge.end == edge.start
                            {
                                let mut new_hist = other_edge.history.clone();
                                new_hist.push(edge.clone());

                                Some(ChartEdge {
                                    dotted_rule: other_edge.dotted_rule.clone().advanced_dot(),
                                    start: other_edge.start,
                                    end: edge.end,
                                    history: new_hist,
                                })
                            } else {
                                None
                            }
                        })
                        .collect();

                    self.add_edges(new_edges);
                }
            }

            return edge;
        } else {
            panic!("No processing left!");
        }
    }

    fn add_edge(&mut self, new_edge: ChartEdge<N, T>) {
        if !self.all_edges.contains(&new_edge) {
            self.add_to_trace_chart(&new_edge);
            self.to_process.push_back(new_edge.clone());
            self.all_edges.insert(new_edge);
        }
    }

    fn add_edges<I>(&mut self, new_edges: I)
    where
        I: IntoIterator<Item = ChartEdge<N, T>>,
    {
        for new_edge in new_edges {
            self.add_edge(new_edge);
        }
    }

    pub fn complete_derivations(&self) -> &Vec<ChartEdge<N, T>> {
        &self.complete_derivations
    }

    pub fn generate_derivation_trees(&self) -> Vec<Tree<Symbol<N, T>>> {
        self.complete_derivations()
            .iter()
            .map(|e| e.generate_derivation_tree())
            .collect()
    }
}
