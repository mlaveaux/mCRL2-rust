use lpsreach::Config;
use lpsreach::run;

// A test for one of the given models.
#[test]
fn test_anderson()
{
    let config = Config {
        filename: String::from("models/anderson.4.ldd")
    };

    if let Ok(result) = run(&config)
    {
        assert_eq!(result, 29641, "Number of states does not match expected amount.");
    }
}