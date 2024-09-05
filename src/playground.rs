use std::{
    cell::RefCell,
    error::Error,
    f64::consts::PI,
    fmt::{Debug, Display},
    io::{self, BufRead, BufReader, Write},
    net::{IpAddr, Ipv4Addr, TcpListener},
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use rand::Rng;

fn read_number() -> i32 {
    loop {
        let mut x = String::new();
        io::stdin().read_line(&mut x).expect("读取失败");
        match x.trim().parse() {
            Ok(x) => return x,
            Err(_) => {
                println!("不是数字");
                continue;
            }
        };
    }
}

fn guess_number() {
    println!("猜数字（又来？）");
    let n = rand::thread_rng().gen_range(1..101);
    'big_loop: loop {
        println!("真实数据是{}，请输入", n);
        let x = read_number();
        println!("你猜了{}", x);
        match x.cmp(&n) {
            std::cmp::Ordering::Less => println!("小啊！"),
            std::cmp::Ordering::Equal => {
                println!("等啊！");
                break 'big_loop;
            }
            std::cmp::Ordering::Greater => println!("大啊！"),
        }
    }
}

fn fahrenheit_celsius() {
    let g: f64 = read_number().into();
    println!("{}", g * 9.0 / 5.0 + 32.0)
}

fn fibonacci(n: i32) -> i32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 1..=n {
        (a, b) = (b, a + b);
    }
    a
}

fn twelve_days() {
    for i in [("瓶", "啤酒"), ("罐", "可乐"), ("袋", "薯片")] {
        println!("隔壁超市有99{}子{}", i.0, i.1)
    }
}

const MY_ARRAY: [i32; 4] = [114, 514, 1919, 810];

#[derive(Debug)]
struct WTF;
impl WTF {
    fn namespaced_function() -> i32 {
        4
    }

    fn longest<'a>(&self, s: &'a str, t: &'a str) -> &'a str {
        if s.len() > t.len() {
            s
        } else {
            t
        }
    }
}

fn first_word(s: &str) -> &str {
    for (i, &c) in s.as_bytes().iter().enumerate() {
        if c == b' ' {
            return &s[0..i];
        }
    }
    &s[..]
}

#[derive(PartialEq, Debug)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(PartialEq, Debug)]
pub struct Vec2<T>(T, T);

impl<T> Vec2<T> {
    fn second(self) -> T {
        self.1
    }
}
impl Vec2<i16> {
    fn len(&self) -> f64 {
        f64::hypot(self.0.into(), self.1.into())
    }
}

impl<T> Area for Vec2<T> {
    fn area(&self) -> f64 {
        0.0
    }
}

trait Area {
    fn area(&self) -> f64;
}

impl Rect {
    fn bottom_right(&self) -> (i32, i32) {
        (self.x + self.width, self.y + self.height)
    }
    fn new(width: i32, height: i32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
    fn square(size: i32) -> Self {
        Self::new(size, size)
    }
}

impl Area for Rect {
    fn area(&self) -> f64 {
        f64::from(self.width) * f64::from(self.height)
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rect(({}, {}) +({}, {}) → ({}, {}))",
            self.x,
            self.y,
            self.width,
            self.height,
            self.x + self.width,
            self.y + self.height
        )
    }
}
#[derive(Debug)]
struct Circle {
    radius: f64,
}

impl Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circle(r={})", self.radius)
    }
}

impl Area for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}

fn front_and_back(shape: &impl Area) -> f64 {
    shape.area() * 2.0
}
fn front_and_back2<T: Area>(shape: &T) -> f64 {
    shape.area() * 2.0
}
fn get_areaable() -> impl Area + Debug + Display {
    if rand::thread_rng().gen_bool(0.5) {
        Rect::square(1919810)
    } else {
        Rect::square(114514)
    }
}
pub fn main() -> Result<(), Box<dyn Error>> {
    println!("斐(7) = {}", fibonacci(7));
    println!("常量 = {MY_ARRAY:?}，切片 = {:?}", &MY_ARRAY[1..3]);
    println!("字符串第一个词 = {:?}", first_word("many words"));
    twelve_days();
    let rect1 = Rect {
        x: 0,
        y: 0,
        width: 114,
        height: 514,
    };
    println!("矩形{rect1} {} {:?}", rect1.area(), rect1.bottom_right());
    let rect2 = Rect::square(1919);
    println!("矩形{rect2}");
    let circle = Circle { radius: 1919.810 };
    println!(
        "圆形{circle} 两倍面积{}={}",
        front_and_back(&circle),
        front_and_back2(&circle)
    );
    println!("圆形转换为字符串 {}", circle.to_string());
    println!("随机形状{}", get_areaable());
    println!("空结构体{:?}", WTF::namespaced_function());
    println!("IP地址{}", IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
    let mut val: Option<i32> = None;
    println!("空值{:?}", val);
    val = None;
    println!("还是空值{:?}", val);
    val = Some(114514);
    let Some(u) = val else { return Ok(()) };
    println!("有值{}", u);
    {
        use std::collections::BTreeMap;
        dbg!(BTreeMap::<u16, f64>::new());
        ()
    }
    eat();
    lifetime();
    closure();
    lisp();
    actor();
    polymorphism();
    dark_arts();
    server();

    println!("打开失败{:?}", read_username_from_file().expect_err("???"));
    read_username_from_file()?;

    println!(
        "二维向量长={:?} 第二维={:?}",
        Vec2(3, 4).len(),
        Vec2([114], [514]).second()
    );

    Vec2(3, 4);

    学生管理系统();
    fahrenheit_celsius();
    guess_number();
    Ok(())
}

fn eat() {
    let g = Guess(3);
    println!("Guess g = {} = {}", g.value(), g.0);
    let g = Guess::new(3);
    println!("Guess g2 = {} = {}", g.value(), g.0);
    let g = guess::Guess::new(3);
    println!("Guess g3 = {}", g.value());
    super::front_of_house::hosting::add_to_waitlist();
}

fn 学生管理系统() {
    panic!("做起来很麻烦！");
}
use std::fs::File;
use std::io::Read;

fn read_username_from_file() -> Result<String, io::Error> {
    let mut username = String::new();
    File::open("这个文件应该不存在")?.read_to_string(&mut username)?;
    Err(io::Error::from(io::Error::new(
        io::ErrorKind::BrokenPipe,
        "WWWWW",
    )))
}

#[derive(Debug)]
pub struct Guess(i32); // 这个就没用了，因为它是公开的
impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 || value > 100 {
            panic!("Guess value must be between 1 and 100, got {value}.");
        }
        Guess(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }
}

mod guess {
    #[derive(Debug)]
    pub struct Guess(i32);
    impl Guess {
        pub fn new(value: i32) -> Guess {
            if value < 1 || value > 100 {
                panic!("Guess value must be between 1 and 100, got {value}.");
            }
            Guess(value)
        }

        pub fn value(&self) -> i32 {
            self.0
        }
    }
}

fn longest<'a>(s: &'a str, t: &'a str) -> &'a str {
    if s.len() > t.len() {
        s
    } else {
        t
    }
}

#[derive(Debug)]
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {announcement}");
        self.part
    }
}

fn longest_with_an_announcement<'a, T: Display>(x: &'a str, y: &'a str, ann: T) -> &'a str {
    println!("现在有一条通知！{ann}");
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn lifetime() {
    let string1;
    {
        string1 = String::from("abcd");
    }
    let string2;
    {
        string2 = "xyz";
    }

    let result = longest(string1.as_str(), string2);
    let result2 = WTF {}.longest(string1.as_str(), string2);
    let result3 = longest_with_an_announcement(string1.as_str(), string2, "最长前播报");
    println!("最长的字符串是{result:?}、{result2:?}、{result3:?}");

    let i;
    let novel = String::from("请叫我伊莎玛拉");
    {
        let first_sentence = novel.split('.').next().unwrap();
        i = ImportantExcerpt {
            part: first_sentence,
        };
    }
    println!(
        "{}重要摘抄{:?} {} {}",
        i.level(),
        i,
        i.part,
        i.announce_and_return_part("啊啊")
    )
}

fn closure() {
    let mut list = vec![1, 2, 3];
    println!["闭包前{list:?}"];

    let mut borrows_mutably = || list.push(7);
    borrows_mutably();
    println! {"闭包后{list:?}"};
    let mut another = move || {
        list.push(-1);
        println!("向量移入闭包{list:?}")
    };
    another();
    thread::spawn(another).join().unwrap();

    {
        let o: Option<Vec<i32>> = None;
        // let g = o.unwrap_or_else(Vec::new);
        // println!("直接用函数名{:?}", g);
        let v = vec![9, 9, 9];
        let f = move || {
            println!("向量移入闭包{v:?}");
            vec![2, 3, 3]
        };
        println!("选项被消耗{:?}", o.unwrap_or_else(f.clone()));
        println!("复制捕获参数{:?}", f());
    }

    let mut list = [
        Circle { radius: 10.0 },
        Circle { radius: 3.3 },
        Circle { radius: 7.7 },
    ];

    let mut num_sort_operations = 0;
    list.sort_by_key(|r| {
        num_sort_operations += 1;
        (114, r.radius.to_bits(), 514)
    });
    println!("排序完成{list:?}，共用{num_sort_operations}次键");

    let mut nums = vec![5.5f64, 0.9, 1.1, 3.3, -19.19, 0.0, -8.10, 2.2];
    // nums.sort();
    nums.sort_by_key(|x| x.to_bits());
    println!("排序完成{nums:?}");
    nums.sort_by(f64::total_cmp);
    println!("排序完成{nums:?}");

    println!("接口中的类型常量{:?}", MyIterator1::next(&WTF {}));
    println!(
        "接口中的泛型参数{:?}",
        MyIterator2::<
            Result<
                Result<Result<WTF, WTF>, Result<WTF, WTF>>,
                Result<Result<WTF, WTF>, Result<WTF, WTF>>,
            >,
        >::next(&WTF {})
    );
    let mut v1 = vec![String::from("aaa"), String::from("bb"), String::new()];
    for x in v1.iter_mut() {
        println!("{:?}", x);
    }
}

trait MyIterator1 {
    type Item;
    fn next(&self) -> Option<Self::Item>;
}
trait MyIterator2<Item> {
    fn next(&self) -> Option<Item>;
}

impl MyIterator1 for WTF {
    type Item = WTF;
    fn next(&self) -> Option<Self::Item> {
        None
    }
}
impl<T> MyIterator2<T> for WTF {
    fn next(&self) -> Option<T> {
        None
    }
}

#[test]
fn iterator_demonstration() {
    let v1 = vec![1, 2, 3];

    let mut v1_iter = v1.iter();

    assert_eq!(v1_iter.next(), Some(&1));
    assert_eq!(v1_iter.next(), Some(&2));
    assert_eq!(v1_iter.next(), Some(&3));
    assert_eq!(v1_iter.next(), None);

    v1.iter().sum::<i32>();
}

struct 盒<型>(型);
impl<型> 盒<型> {
    fn new(x: 型) -> 盒<型> {
        盒(x)
    }
}
impl<型> Deref for 盒<型> {
    type Target = 型;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<型> Drop for 盒<型> {
    fn drop(&mut self) {
        println!("盒子被丢弃！");
    }
}
fn lisp() {
    #[derive(Debug)]
    enum List {
        Cons(i32, Rc<List>),
        Nil,
    }
    use List::{Cons, Nil};
    let list = Rc::new(Cons(1, Rc::new(Cons(2, Rc::new(Cons(3, Rc::new(Nil)))))));
    let Cons(head, ref tail) = *list else {
        panic!("什么玩意")
    };
    println!("列表{list:?} 头{head:?} 尾{tail:?}");
    {
        let grand = Cons(99, list.clone());
        println!("大列表{grand:?} 小列表{list:?}");
        println!("小列表引用计数{}", Rc::strong_count(&list));
    }
    println!("小列表引用计数{}", Rc::strong_count(&list));

    // Rc是不可线程间移动文物
    // thread::spawn(move || println!("引用计数指针移入闭包{list:?}"));

    let x = 5;
    let y = &x;
    let z = Box::new(x);
    let h = 盒::new(x);

    assert_eq!(5, x);
    assert_eq!(&5, y);
    assert_ne!(&4, y);
    assert_eq!(&&&&&&&5, &&&&&&y);
    assert_eq!(5, *y);
    assert_eq!(5, *z);
    assert_eq!(5, *h);
    assert_eq!(5, h.0);

    fn hello(name: &str) {
        println!("Hello, {name}!");
    }
    hello(&Box::new(String::from("Rust")));
    hello(盒::new(String::from("Rust")).deref().deref());
    hello(String::from("Rust").deref_mut());
    hello(String::from("Rust").deref());
    hello(&盒::new(String::from("Rust")));
    drop(h);

    let cell = RefCell::new(vec![]);
    cell.borrow_mut().push(114514);
    cell.borrow_mut().push(1919810);
    println!("引用单元{cell:?} 内部值{:?}", cell.borrow());

    #[derive(Debug)]
    enum ListCell {
        Cons(Rc<RefCell<i32>>, Rc<ListCell>),
        Nil,
    }
    impl Display for ListCell {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ListCell::Cons(x, y) => write!(f, "'({} {})", x.borrow(), y),
                ListCell::Nil => f.write_str("nil"),
            }
        }
    }

    let value = Rc::new(RefCell::new(5));

    let a = Rc::new(ListCell::Cons(Rc::clone(&value), Rc::new(ListCell::Nil)));

    let b = ListCell::Cons(Rc::new(RefCell::new(3)), Rc::clone(&a));
    let c = ListCell::Cons(Rc::new(RefCell::new(4)), Rc::clone(&a));

    *value.borrow_mut() += 10;

    println!("a after = {a:?}");
    println!("b after = {b}");
    println!("c after = {c}");
}

#[test]
#[should_panic(expected = "already borrowed")]
fn refcell_gg() {
    let cell = RefCell::new(vec![1, 2, 3]);
    let a = cell.borrow_mut();
    let b = cell.borrow_mut();
    println!("{a:?}{b:?}");
}

fn actor() {
    let (tx, rx) = mpsc::channel();
    tx.send(String::from("逸一时，误一世")).unwrap();
    println!("收到了{:?}", rx.try_recv());
    println!("收不到{:?}", rx.try_recv());
    let tx_in_thread = tx.clone();
    thread::spawn(move || {
        for x in vec!["逸一时", "误一世"] {
            println!("准备发{:?}", x);
            tx_in_thread.send(String::from(x)).expect("发不出");
            thread::sleep(Duration::from_millis(50));
        }
    });
    tx.send(String::from("主线程消息")).unwrap();
    drop(tx);
    for x in rx {
        println!("收到了{:?}", x);
    }

    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap();
        *num = 6;
    }
    println!("m={m:?}");

    let counter = Arc::new(Mutex::new(0));
    let handles = (0..10)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("总计：{}", *counter.lock().unwrap());
}

fn polymorphism() {
    let x = Rect::new(32, 64);
    let y = Rect::square(114);
    let mut a: Vec<&dyn Area> = vec![&x, &y];
    let z = Circle { radius: 19.19 };
    a.push(&z);
    println!("面积之和={}", a.iter().map(|x| x.area()).sum::<f64>());

    let x = 114.514f64;
    match x {
        v @ 114.0..=514.0 => println!("114~514，事{v}"),
        1919.0.. => println!("1919+"),
        ..=-810.0 => println!("<-810"),
        _ => println!("something else"),
    }
}

fn dark_arts() {
    let mut num = 5;
    let p = &mut num as *const i32;
    let q = &mut num as *mut i32;
    unsafe {
        println!("p[{p:?}]={} q[{q:?}]={}", *p, *q);
        *q = 6;
        println!("p[{p:?}]={} q[{q:?}]={}", *p, *q);
        println!("p+3={:?}", p.add(3));
    }

    let mut v = vec![1, 2, 3, 4, 5, 6];
    let (a, b) = (&v[..3], &v[3..]);
    println!("a={a:?} b={b:?}");
    let r = &mut v[..];
    let (a, b) = r.split_at_mut(3);
    println!("a={a:?} b={b:?}");

    macro_rules! hash_map {
        ($($x:expr),*) => {
            {
                let mut z = std::collections::HashMap::new();
                $(
                    let (k, v) = $x;
                    z.insert(k, v);
                )*
                z
            }
        };
    }

    let m = hash_map!((1, "one"), (2, "two"), (3, "three"));
    println!("通过宏构造的哈希表{m:?}");
    println!("#x {:?}", stringify!(x));
}

fn server() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let homepage = "<!DOCTYPE html>
<title>Hello Rust!</title>
<h1>你好，Rust！</h1>
<p>这是HTTP服务器";
    fn respond(mut stream: std::net::TcpStream, data: &str) {
        stream
            .write_all(
                format!(
                    "HTTP/1.1 418 I'm a teapot\r
Content-Length: {}\r
\r
{}",
                    data.len(),
                    data
                )
                .as_bytes(),
            )
            .unwrap();
    }
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("链接");
        let reader = BufReader::new(&mut stream);
        for line in reader.lines() {
            let line = line.unwrap();
            if line.is_empty() {
                break;
            }
            println!("line {line}");
        }
        respond(stream, &homepage);
    }
}
