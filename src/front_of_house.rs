use std::collections::HashMap;

pub mod hosting {
    pub fn add_to_waitlist() {
        seat_at_table();
        super::serving::take_order()
    }

    fn seat_at_table() {}
}

pub mod serving {
    use std::{collections::HashMap, ops::Add};
    pub fn take_order() {
        serve_order();
        take_payment();
        washing_room();
        super::solve();
    }

    fn serve_order() -> i32 {
        let mut v = Vec::new();
        v.push(114);
        v.push(514);
        v.push(1919);
        v.push(810);
        println!("两数{} {}", v[3], 1);
        for x in &mut v {
            print!("一数{x}");
            *x += 50;
        }
        println!("向量{v:?}");
        for x in &v {
            print!("二刷{x}");
        }
        println!("向量{v:?}");
        114514
    }

    fn take_payment() {
        let mut s = String::from("1234");
        s.push('x');
        let s2 = String::from("abc");
        s.push_str(&s2);
        s = s.add("追加");
        println!("字符串{s:?}，长度{}", s.len());
    }

    fn washing_room() {
        let mut m = HashMap::new();
        m.insert(String::from("red"), 50);
        m.insert(String::from("blue"), 10);
        m.insert(String::from("blue"), 30);
        for (key, value) in &m {
            print!("{key}={value} ")
        }
        let x = m.get("blue");
        println!("哈希表{m:?} 一个值{x:?}");

        let mut m2 = HashMap::new();
        m2.insert("x", "a");
        m2.insert("y", "b");
        let g = String::from("z");
        m2.insert(&g, "c");
        let z = m2.entry("a").or_insert("a");
        let znewval = format!("{z}={z}");
        *z = &znewval;
        println!("静态哈希表{m2:?}");
        println!(
            "单词分割{:?}",
            "a b aa bb ccc".split_ascii_whitespace().collect::<Vec<_>>()
        )
    }
}

fn solve() {
    fn median_and_mode(a: &[i32]) -> (i32, i32) {
        let mut a: Vec<i32> = a.into();
        a.sort();
        let mut m = HashMap::<i32, usize>::new();
        for &i in &a {
            *m.entry(i).or_insert(0) += 1
        }
        let mut maxi = (0, 0);
        for (&key, &value) in &m {
            if value > maxi.1 {
                maxi = (key, value)
            }
        }
        println!("计算过程 排序={a:?} 计数={m:?}");
        (a[a.len() / 2], maxi.0)
    }
    let array = [114, 514, 1919, 810, 114];
    let (median, mode) = median_and_mode(&array);
    println!("数组={array:?} 中位数={median} 众数={mode}");

    fn pig_latin(s: &str) -> String {
        let mut result = Vec::<String>::new();
        let vowels = b"aeiou";
        for w in s.split_whitespace() {
            if vowels.contains(&w.as_bytes()[0]) {
                result.push(format!("{}-hay", w))
            } else {
                result.push(format!("{}-{}ay", &w[1..], &w[..1]))
            }
        }
        result.join(" ")
    }
    println!(
        "pig latin = {}",
        pig_latin("first hello world apple banana")
    );

    println!("最大值={:?}", find_max(&['y', 'm', 'a', 'Z', 'q', '0']));
}

fn find_max<T: std::cmp::PartialOrd>(a: &[T]) -> &T {
    let mut max = &a[0];
    for x in a {
        if x > max {
            max = x
        }
    }
    max
}
