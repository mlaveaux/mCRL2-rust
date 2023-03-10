#[cfg(test)]
mod tests {
    use mcrl2_benchmarks::load_case;

    // Compare the results of various rewriters.
    #[test]
    fn test_benchmark_case() {
        let (_, _) = load_case("cases/add16", 100);
    }
}
