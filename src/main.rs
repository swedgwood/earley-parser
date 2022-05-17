mod earley;
mod tree;

use earley::{Chart, Nonterminal, Production, Symbol};

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

    let mut chart: Chart<Nt, T> = Chart::new(input_string, productions);
    chart.set_trace(true);
    chart.process_all();

    let num_parses = chart.complete_derivations().len();

    let chart_ordered = chart.trace_chart();

    for (i, (edge, history)) in chart_ordered.iter().enumerate() {
        println!(
            "{:3} | {:15} | {:3},{:3} | {}",
            i,
            edge.dotted_rule(),
            edge.start(),
            edge.end(),
            history
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }

    for derivation_tree in chart.generate_derivation_trees() {
        println!("{}", derivation_tree);
        println!();
    }

    println!("Num parses: {}", num_parses);
}
