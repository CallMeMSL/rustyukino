pub mod registered;
pub mod unregistered;


async fn split_at_fist_space(command: &str) -> (String, String) {
    let mut operand = Vec::new();
    let mut argument = Vec::new();
    let mut take_operand = true;
    for c in command.chars() {
        if c == ' ' && take_operand {
            take_operand = false;
            continue;
        }
        if take_operand {
            operand.push(c);
        } else {
            argument.push(c);
        }
    }
    (operand.iter().collect(), argument.iter().collect())
}

#[tokio::test]
async fn test_split() {
    assert_eq!(split_at_fist_space("add link").await, ("add".to_string(), "link".to_string()));
    assert_eq!(split_at_fist_space("addlink").await, ("addlink".to_string(), "".to_string()));
    assert_eq!(split_at_fist_space("add link and so on").await,
               ("add".to_string(), "link and so on".to_string()));
}