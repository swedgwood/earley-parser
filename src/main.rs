mod earley;

use core::num;
use std::fmt::Display;

use earley::{Chart, ChartEdge, Nonterminal, Production, Symbol, Terminal};

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

struct Tree {
    node: Symbol<Nt, T>,
    children: Vec<Tree>,
}

impl ToString for Tree {
    fn to_string(&self) -> String {
        if self.children.len() > 1 {
            // Each subtree, but we reverse the rows so its easier to add to the end
            let mut children_strings: Vec<Vec<String>> = self
                .children
                .iter()
                .map(|t| {
                    let mut subtree_strings: Vec<String> =
                        t.to_string().split("\n").map(|s| s.to_owned()).collect();
                    subtree_strings.reverse();
                    subtree_strings
                })
                .collect();
            let max_height = children_strings.iter().map(|s| s.len()).max().unwrap_or(0);

            let mut branch_length = 0;
            let children_strings_len = children_strings.len();

            for (i, child_strings) in children_strings.iter_mut().enumerate() {
                // This should mean every subtree vec in children_strings is the same length (max_height + 1)
                for _ in 0..(max_height - child_strings.len() + 1) {
                    child_strings.push("|".to_owned());
                }

                let max_subtree_width = child_strings.iter().map(|s| s.len()).max().unwrap_or(0);

                let right_padding = if i == children_strings_len - 1 { 0 } else { 1 };

                if i != children_strings_len - 1 {
                    branch_length += max_subtree_width + right_padding;
                }

                for child_string in child_strings.iter_mut() {
                    child_string.push_str(
                        &" ".repeat(max_subtree_width - child_string.len() + right_padding),
                    );
                }
            }

            let branch_string = "|".to_owned() + &"_".repeat(branch_length - 1);

            let mut lines: Vec<String> = Vec::new();

            for i in (0..max_height + 1) {
                let mut line = String::new();
                for s in children_strings.iter() {
                    line.push_str(&s[i]);
                }
                lines.push(line);
            }

            lines.push(branch_string);
            lines.push(self.node.to_string().to_owned());

            lines.reverse();

            lines.join("\n")
        } else if self.children.len() == 1 {
            self.node.to_string() + "\n|\n" + &self.children[0].to_string()
        } else {
            self.node.to_string()
        }
    }
}

fn derivation_tree(deriv: &ChartEdge<Nt, T>) -> Tree {
    let mut children: Vec<Tree> = deriv.history().into_iter().map(derivation_tree).collect();

    for sym in deriv.dotted_rule().production().rhs() {
        if let Symbol::Terminal(t) = sym {
            children.push(Tree {
                node: Symbol::Terminal(t),
                children: vec![],
            })
        }
    }

    Tree {
        node: Symbol::Nonterminal(deriv.dotted_rule().production().lhs()),
        children,
    }
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
