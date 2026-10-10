use microlp::{ComparisonOp, OptimizationDirection, Problem, SolveOutcome};

/// `d[i][j]` = number of voters ranking option i strictly better (lower rank) than j.
pub fn duels(n: usize, ballots: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut d = vec![vec![0; n]; n];
    for b in ballots {
        for i in 0..n {
            for j in 0..n {
                if b[i] < b[j] {
                    d[i][j] += 1;
                }
            }
        }
    }
    d
}

pub fn condorcet_winner(d: &[Vec<u32>]) -> Option<usize> {
    (0..d.len()).find(|&i| (0..d.len()).all(|j| i == j || d[i][j] > d[j][i]))
}

/// Maximal lottery: the optimal mixed strategy of the zero-sum duel game.
/// Among optimal lotteries, the one maximizing the smallest probability is chosen,
/// so perfect ties split evenly instead of favoring the first option.
pub fn maximal_lottery(d: &[Vec<u32>]) -> Vec<f64> {
    let n = d.len();
    let mut pb = Problem::new(OptimizationDirection::Maximize);
    let p: Vec<_> = (0..n).map(|_| pb.add_var(0.0, (0.0, 1.0))).collect();
    let t = pb.add_var(1.0, (0.0, 1.0));
    pb.add_constraint(
        p.iter().map(|&v| (v, 1.0)).collect::<Vec<_>>(),
        ComparisonOp::Eq,
        1.0,
    );
    for j in 0..n {
        // The lottery must not lose, on average, against option j.
        let expr: Vec<_> = (0..n)
            .map(|i| (p[i], d[i][j] as f64 - d[j][i] as f64))
            .collect();
        pb.add_constraint(expr, ComparisonOp::Ge, 0.0);
        pb.add_constraint([(p[j], 1.0), (t, -1.0)], ComparisonOp::Ge, 0.0);
    }
    match pb.solve() {
        Ok(SolveOutcome::Solution(s)) => p.iter().map(|&v| if s[v] < 1e-9 { 0.0 } else { s[v].min(1.0) }).collect(),
        // ponytail: the LP is always feasible (a symmetric game has value 0); uniform is a safety net only.
        _ => vec![1.0 / n as f64; n],
    }
}

/// Picks the option whose cumulative probability interval contains `u` in [0, 1).
pub fn draw(probs: &[f64], u: f64) -> usize {
    let mut acc = 0.0;
    for (i, p) in probs.iter().enumerate() {
        acc += p;
        if u < acc {
            return i;
        }
    }
    probs.iter().rposition(|&p| p > 0.0).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: &[f64], b: &[f64]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6)
    }

    #[test]
    fn clear_winner() {
        let d = duels(3, &[vec![1, 2, 3], vec![1, 3, 2], vec![2, 1, 3]]);
        assert_eq!(condorcet_winner(&d), Some(0));
        assert!(close(&maximal_lottery(&d), &[1.0, 0.0, 0.0]));
    }

    #[test]
    fn rock_paper_scissors() {
        let d = duels(3, &[vec![1, 2, 3], vec![3, 1, 2], vec![2, 3, 1]]);
        assert_eq!(condorcet_winner(&d), None);
        let third = 1.0 / 3.0;
        assert!(close(&maximal_lottery(&d), &[third, third, third]));
    }

    #[test]
    fn ties_and_perfect_split() {
        // Equal ranks count for nobody.
        let d = duels(2, &[vec![1, 1]]);
        assert_eq!(d, vec![vec![0, 0], vec![0, 0]]);
        let d = duels(2, &[vec![1, 2], vec![2, 1]]);
        assert_eq!(condorcet_winner(&d), None);
        assert!(close(&maximal_lottery(&d), &[0.5, 0.5]));
    }

    #[test]
    fn cycle_with_unequal_strengths() {
        // A>B>C>A cycle plus a dominated option D: D must get 0.
        let b = vec![
            vec![1, 2, 3, 4],
            vec![1, 2, 3, 4],
            vec![2, 3, 1, 4],
            vec![3, 1, 2, 4],
            vec![3, 1, 2, 4],
        ];
        let d = duels(4, &b);
        assert_eq!(condorcet_winner(&d), None);
        let p = maximal_lottery(&d);
        assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-6);
        assert!(p[3].abs() < 1e-6);
        for j in 0..4 {
            let score: f64 = (0..4)
                .map(|i| p[i] * (d[i][j] as f64 - d[j][i] as f64))
                .sum();
            assert!(score > -1e-6);
        }
    }

    #[test]
    fn draw_intervals() {
        assert_eq!(draw(&[0.5, 0.5], 0.0), 0);
        assert_eq!(draw(&[0.5, 0.5], 0.7), 1);
        assert_eq!(draw(&[0.0, 1.0], 0.0), 1);
        assert_eq!(draw(&[0.3, 0.7 - 1e-12], 0.999999999999), 1);
    }
}
