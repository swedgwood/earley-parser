/// Symbols (terminals and non-terminals)
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sym {
    T(&'static str),
    Nt(&'static str),
}

impl Display for Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sym::T(s) => write!(f, "{}", s),
            Sym::Nt(s) => write!(f, "{}", s),
        }
    }
}

use core::num;
use std::fmt::Display;

use Sym::*;

/// Production
#[derive(Copy, Clone, PartialEq, Eq)]
struct Prod {
    lhs: &'static str,
    rhs: &'static [Sym],
}

impl Display for Prod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ->", self.lhs)?;
        for sym in self.rhs {
            write!(f, " {}", sym)?;
        }
        Ok(())
    }
}

impl Prod {
    fn new(lhs: &'static str, rhs: &'static [Sym]) -> Self {
        Self { lhs, rhs }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct DottedRule {
    prod: Prod,
    dot_pos: usize,
}

impl Display for DottedRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ->", self.prod.lhs)?;
        for (i, sym) in self.prod.rhs.iter().enumerate() {
            write!(f, "{}{}", if i == self.dot_pos { "•" } else { " " }, sym)?;
        }
        if self.dot_pos == self.prod.rhs.len() {
            write!(f, "•")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ChartEdge {
    rule: DottedRule,
    start: usize,
    end: usize,
    hist: Option<usize>,
}

impl Display for ChartEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:10} | {:2} | {:2} | {:5}",
            self.rule,
            self.start,
            self.end,
            if let Some(s) = self.hist {
                s.to_string()
            } else {
                String::new()
            }
        )
    }
}

fn generate_dotted_rules(productions: &Vec<Prod>) -> Vec<DottedRule> {
    productions
        .iter()
        .flat_map(|prod| {
            (0..prod.rhs.len() + 1).map(|i| DottedRule {
                prod: *prod,
                dot_pos: i,
            })
        })
        .collect()
}

fn push_not_dup(chart: &mut Vec<ChartEdge>, edge_to_add: ChartEdge) {
    let edge_exists = chart.iter().find(|c| **c == edge_to_add);

    if edge_exists.is_none() {
        chart.push(edge_to_add);
    }
}

fn main() {
    let prod_s = Prod::new("S", &[Nt("NP"), Nt("VP")]);

    let productions: Vec<Prod> = vec![
        // S -> NP VP
        prod_s,
        // ==================================================
        // NP -> N
        Prod::new("NP", &[Nt("N")]),
        // NP -> N PP
        Prod::new("NP", &[Nt("N"), Nt("PP")]),
        // ==================================================
        // PP -> P NP
        Prod::new("PP", &[Nt("P"), Nt("NP")]),
        // ==================================================
        // VP -> V
        Prod::new("VP", &[Nt("V")]),
        // VP -> V NP
        Prod::new("VP", &[Nt("V"), Nt("NP")]),
        // VP -> V VP
        Prod::new("VP", &[Nt("V"), Nt("VP")]),
        // VP -> VP PP
        Prod::new("VP", &[Nt("VP"), Nt("PP")]),
        // ==================================================
        // N -> can
        Prod::new("N", &[T("can")]),
        // N -> fish
        Prod::new("N", &[T("fish")]),
        // N -> rivers
        Prod::new("N", &[T("rivers")]),
        // N -> they
        Prod::new("N", &[T("they")]),
        // N -> december
        Prod::new("N", &[T("december")]),
        // N -> ...
        // Prod::new("N", &[T("...")]),
        // ==================================================
        // P -> in
        Prod::new("P", &[T("in")]),
        // P -> ...
        // Prod::new("N", &[T("...")]),
        // ==================================================
        // V -> can
        Prod::new("V", &[T("can")]),
        // V -> fish
        Prod::new("V", &[T("fish")]),
        // V -> ...
        // Prod::new("V", vec![T("...")]),
    ];

    let dotted_rules = generate_dotted_rules(&productions);

    println!("Productions:");
    for (i, p) in productions.iter().enumerate() {
        println!("{}) {}", i, p)
    }

    println!("");
    println!("Dotted rules:");
    for (i, r) in dotted_rules.iter().enumerate() {
        println!("{}) {}", i, r);
    }

    let mut chart: Vec<ChartEdge> = Vec::new();

    chart.push(ChartEdge {
        rule: DottedRule {
            prod: prod_s,
            dot_pos: 0,
        },
        start: 0,
        end: 0,
        hist: None,
    });

    let mut chart_pos = 0;

    //                             NP      VP
    //                             N       V      VP
    //                                            VP      PP
    //                                            V       P     NP
    //                                                          N

    let input_string = ["they", "can", "fish", "in", "rivers", "in", "december"];
    // let input_string = ["they", "can", "fish"];

    let mut num_parses = 0;

    while chart_pos < chart.len() {
        let ChartEdge {
            rule,
            start,
            end,
            hist,
        } = chart[chart_pos];

        if rule.dot_pos == rule.prod.rhs.len() || !["N", "P", "V"].contains(&rule.prod.lhs) {
            println!("{:3} | {}", chart_pos, chart[chart_pos]);
        }

        if start == 0
            && end == input_string.len()
            && rule.dot_pos == rule.prod.rhs.len()
            && rule.prod.lhs == "S"
        {
            println!("Full parse!!! woo!!!");
            num_parses += 1;
        }

        let DottedRule { prod, dot_pos } = rule;

        if dot_pos < prod.rhs.len() {
            let sym = prod.rhs[dot_pos];

            match sym {
                T(s) => {
                    if end >= input_string.len() {
                        chart_pos += 1;
                        continue;
                    }
                    // If next symbol after dot is a terminal, read it in from the input
                    let next_token = input_string[end];
                    if s == next_token {
                        push_not_dup(
                            &mut chart,
                            ChartEdge {
                                rule: DottedRule {
                                    prod,
                                    dot_pos: dot_pos + 1,
                                },
                                start: start,
                                end: end + 1,
                                hist: None,
                            },
                        )
                    }
                }
                Nt(s) => {
                    // If next symbol after dot is a nonterminal, add edges for productions
                    for prod in productions.iter() {
                        if prod.lhs == s {
                            let edge = chart
                                .iter()
                                .enumerate()
                                .find(|(_, c)| {
                                    c.rule.prod == *prod
                                        && c.rule.dot_pos == c.rule.prod.rhs.len()
                                        && end == c.start
                                })
                                .map(|(i, x)| (i, *x));

                            if let Some((i, edge)) = edge {
                                push_not_dup(
                                    &mut chart,
                                    ChartEdge {
                                        rule: DottedRule {
                                            prod: rule.prod,
                                            dot_pos: rule.dot_pos + 1,
                                        },
                                        start,
                                        end: edge.end,
                                        hist: Some(i),
                                    },
                                );
                            } else {
                                let edge_to_add = ChartEdge {
                                    rule: DottedRule {
                                        prod: *prod,
                                        dot_pos: 0,
                                    },
                                    start: end,
                                    end,
                                    hist: None,
                                };
                                push_not_dup(&mut chart, edge_to_add);
                            }
                        }
                    }
                }
            }
        } else {
            // Dot is fully on the right, i.e. node explored, so need to propagate
            for ChartEdge {
                rule:
                    DottedRule {
                        prod: prod2,
                        dot_pos,
                    },
                start: s2,
                end: e2,
                ..
            } in chart.clone()
            // TODO: This clone is costly and definitely not needed
            {
                // Check Correct Position
                if e2 != start {
                    continue;
                }

                // Check there dot is not at end
                if dot_pos >= prod2.rhs.len() {
                    continue;
                }

                // Check that symbol after dot is this non-terminal
                if prod2.rhs[dot_pos] != Nt(prod.lhs) {
                    continue;
                }

                // If all checks pass, then advance dot
                push_not_dup(
                    &mut chart,
                    ChartEdge {
                        rule: DottedRule {
                            prod: prod2,
                            dot_pos: dot_pos + 1,
                        },
                        start: s2,
                        end: end,
                        hist: Some(chart_pos),
                    },
                );
            }
        }

        chart_pos += 1;
    }

    println!("Num parse: {}", num_parses);
}
