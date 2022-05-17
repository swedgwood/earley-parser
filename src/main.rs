mod earley;
mod tree;

use core::num;
use std::fmt::Display;

use earley::{Chart, ChartEdge, Nonterminal, Production, Symbol, Terminal};
use tree::Tree;

type Nt = &'static str;
type T = &'static str;

impl Nonterminal for Nt {
    fn start() -> Self {
        "S"
    }
}

fn parse_simple_prods(prods_text: &'static str) -> Vec<Production<Nt, T>> {
    prods_text
        .split("\n")
        .filter(|s| !s.trim().is_empty())
        .flat_map(|s| {
            let (mut lhs, rhses) = s.split_once("->").expect("Failed simple parse (1)");
            lhs = lhs.trim();

            rhses.trim().split("|").map(move |rhs_raw| {
                let rhs: Vec<Symbol<Nt, T>> = rhs_raw
                    .trim()
                    .split(" ")
                    .map(|sym| {
                        if sym
                            .chars()
                            .nth(0)
                            .expect("Failed simple parse (2)")
                            .is_ascii_uppercase()
                        {
                            Symbol::Nonterminal(sym)
                        } else {
                            Symbol::Terminal(sym)
                        }
                    })
                    .collect();

                Production::new(lhs, rhs)
            })
        })
        .collect()
}

fn derivation_tree(deriv: &ChartEdge<Nt, T>) -> Tree<Symbol<Nt, T>> {
    let mut children: Vec<Tree<Symbol<Nt, T>>> =
        deriv.history().into_iter().map(derivation_tree).collect();

    for sym in deriv.dotted_rule().production().rhs() {
        if let Symbol::Terminal(t) = sym {
            children.push(Tree::new(Symbol::Terminal(t), vec![]))
        }
    }

    Tree::new(
        Symbol::Nonterminal(deriv.dotted_rule().production().lhs()),
        children,
    )
}

fn main() {
    let productions = parse_simple_prods(
        " 
        S -> NP VP
        NP -> N
        NP -> N PP
        PP -> P NP
        VP -> V
        VP -> V NP
        VP -> V VP
        VP -> VP PP
        N -> can
        N -> fish
        N -> rivers
        N -> they
        N -> december
        P -> in
        V -> can
        V -> fish
    ",
    );

    let input_string = vec!["they", "can", "fish", "in", "rivers", "in", "december"];
    let input_string_len = input_string.len();

    let mut chart: Chart<Nt, T> = Chart::new(input_string, productions);

    let mut chart_ordered: Vec<ChartEdge<Nt, T>> = Vec::new();
    let mut complete_derivations: Vec<ChartEdge<Nt, T>> = Vec::new();
    let mut num_parses = 0;
    while chart.more_to_process() {
        let edge = chart.process_one();
        chart_ordered.push(edge.clone());

        if edge.dotted_rule().production().lhs() == &"S"
            && edge.dotted_rule().is_complete()
            && edge.start() == 0
            && edge.end() == input_string_len
        {
            num_parses += 1;
            complete_derivations.push(edge);
        }
    }

    for (i, edge) in chart_ordered.iter().enumerate() {
        let history: String = edge
            .history()
            .iter()
            .map(|e| {
                for (j, oe) in chart_ordered.iter().enumerate() {
                    if e == oe {
                        return j.to_string();
                    }
                }
                return "-1".to_owned();
            })
            .collect::<Vec<String>>()
            .join(",");

        println!(
            "{:3} | {:15} | {:3},{:3} | {}",
            i,
            edge.dotted_rule(),
            edge.start(),
            edge.end(),
            history
        );
    }

    for derivation in complete_derivations {
        println!("{}", derivation_tree(&derivation).to_string());
        println!();
    }

    println!("Num parses: {}", num_parses);
}
