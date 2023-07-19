use reach::Config;
use reach::run;

// A test for one of the given models.
#[test]
fn test_anderson()
{
    let args = [
        String::from("path"),
        String::from("models/anderson.4.ldd"),
    ];

    let config = Config::new(args.iter().map(|s| s.to_string())).unwrap();

    if let Ok(result) = run(&config)
    {
        assert_eq!(result, 29641, "Number of states does not match expected amount.");
    }
}