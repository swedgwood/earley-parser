use std::fmt::Display;

pub struct Tree<T> {
    value: T,
    children: Vec<Tree<T>>,
}

impl<T> Tree<T> {
    pub fn new(value: T, children: Vec<Tree<T>>) -> Self {
        Tree { value, children }
    }
}

impl<T> Display for Tree<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
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

                // Maxium height of all the subtrees, so we know how to vertically pad them
                let max_height = children_strings.iter().map(|s| s.len()).max().unwrap_or(0);

                // This branch is the topmost horizontal line of '_'
                let mut branch_length = 0;

                let children_strings_len = children_strings.len();

                for (i, child_strings) in children_strings.iter_mut().enumerate() {
                    // This should mean every subtree has the same height (max_height + 1) (every subvec has same length)
                    for _ in 0..(max_height - child_strings.len() + 1) {
                        child_strings.push("|".to_owned());
                    }

                    // Maximum width along this subtree, so we know how to pad so other subtrees are aligned
                    let max_subtree_width =
                        child_strings.iter().map(|s| s.len()).max().unwrap_or(0);

                    let right_padding = 1;

                    if i != children_strings_len - 1 {
                        // Each subtree has branch lenght equal to width of tree, besides rightmost where the branch doesn't extend.
                        branch_length += max_subtree_width + right_padding;

                        // If not the rightmost subtree, add some padding
                        for child_string in child_strings.iter_mut() {
                            child_string.push_str(
                                &" ".repeat(max_subtree_width - child_string.len() + right_padding),
                            );
                        }
                    }
                }

                let branch_string = "|".to_owned() + &"_".repeat(branch_length - 1);

                let mut lines: Vec<String> = Vec::new();

                // Merge lines of each subtree to make lines of this whole tree
                for i in 0..max_height + 1 {
                    let mut line = String::new();
                    for s in children_strings.iter() {
                        line.push_str(&s[i]);
                    }
                    lines.push(line);
                }

                // Add the branch and the value of the current tree
                lines.push(branch_string);
                lines.push(self.value.to_string().to_owned());

                // Lines were reversed, so un-reverse them
                lines.reverse();

                lines.join("\n")
            } else if self.children.len() == 1 {
                self.value.to_string() + "\n|\n" + &self.children[0].to_string()
            } else {
                self.value.to_string()
            }
        )
    }
}
