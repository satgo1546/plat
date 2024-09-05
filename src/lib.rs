/// 两数之和
///
/// # 示例
///
/// ```
/// assert_eq!(frogicalc_max::add(1,1),2);
/// ```
pub fn add(left: u64, right: u64) -> u64 {
    //! 还有内容
    left + right
}

#[derive(Debug)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn can_hold(&self, other: &Rectangle) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

#[cfg(test)]
mod 我测 {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn hi() {
        assert!(true);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn fail() {
        let x = vec![1, 2, 3];
        x[114514];
    }

    #[test]
    fn 装得下() {
        let a = Rectangle {
            width: 8,
            height: 7,
        };
        let b = Rectangle {
            width: 5,
            height: 1,
        };
        assert!(a.can_hold(&b));
        assert!(!b.can_hold(&a));
    }
    #[test]
    fn it_works2() -> Result<(), String> {
        let result = add(2, 2);

        if result == 4 {
            Ok(())
        } else {
            Err(String::from("two plus two does not equal four"))
        }
    }
}
use std::{env, error::Error, fs, io};

mod front_of_house;
mod playground;

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<(String, String), io::Error> {
    args.next();
    let Some(a) = args.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not enough arguments",
        ));
    };
    let Some(b) = args.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not enough arguments",
        ));
    };
    Ok((a, b))
}

/// 主程序！
pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        return playground::main();
    }
    if args.len() < 3 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not enough arguments",
        )));
    }
    let args = parse_args(args.into_iter())?;
    grep(args.0, args.1, env::var("I").is_ok())
}

fn grep(query: String, filename: String, case_sensitive: bool) -> Result<(), Box<dyn Error>> {
    eprintln!("在{filename}中查找{query}");
    let contents = fs::read_to_string(filename)?;
    eprintln!("文件内容长度 = {}", contents.len());
    for line in (if case_sensitive { search_i } else { search })(&query, &contents) {
        println!("{}", line);
    }
    Ok(())
}

fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents.lines().filter(|l| l.contains(query)).collect()
}

fn search_i<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();
    contents
        .lines()
        .filter(|l| l.to_lowercase().contains(&query))
        .collect()
}

#[cfg(test)]
mod grep测试 {
    use super::*;
    #[test]
    fn a() {
        assert_eq!(
            vec!["鹅，鹅，鹅，"],
            search(
                "鹅",
                "\
鹅，鹅，鹅，
曲项向天歌。
白毛浮绿水，
红掌拨清波。"
            )
        );
    }
    #[test]
    fn b() {
        assert_eq!(
            search(
                "卧槽",
                "\
曲项向天歌。
白毛浮绿水，
红掌拨清波。"
            )
            .len(),
            0
        );
    }
    #[test]
    fn c() {
        assert_eq!(
            search_i(
                "aaa",
                "\
aaa
bbb
AAA"
            ),
            vec!["aaa", "AAA"]
        );
    }
}

pub trait Messenger {
    fn send(&mut self, msg: &str);
}

pub struct LimitTracker {
    value: usize,
    max: usize,
}

impl LimitTracker {
    pub fn new(max: usize) -> LimitTracker {
        LimitTracker { value: 0, max }
    }

    pub fn set_value<T: Messenger>(&mut self, value: usize, messenger: &mut T) {
        self.value = value;

        let percentage_of_max = self.value as f64 / self.max as f64;

        if percentage_of_max >= 1.0 {
            messenger.send("Error: You are over your quota!");
        } else if percentage_of_max >= 0.9 {
            messenger.send("Urgent warning: You've used up over 90% of your quota!");
        } else if percentage_of_max >= 0.75 {
            messenger.send("Warning: You've used up over 75% of your quota!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockMessenger {
        sent_messages: Vec<String>,
    }

    impl Messenger for MockMessenger {
        fn send(&mut self, message: &str) {
            self.sent_messages.push(String::from(message));
        }
    }

    #[test]
    fn it_sends_an_over_75_percent_warning_message() {
        let mut mock_messenger = MockMessenger {
            sent_messages: Vec::new(),
        };
        let mut limit_tracker = LimitTracker::new(100);

        limit_tracker.set_value(80, &mut mock_messenger);

        assert_eq!(mock_messenger.sent_messages.len(), 1);
    }
}
