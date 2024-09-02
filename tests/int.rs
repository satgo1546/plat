use frogicalc_max::{add, Rectangle};

#[test]
fn 加() {
    assert_eq!(add(233, 666), 899);
}

#[test]
fn rectify() {
    let x = Rectangle {
        width: 20,
        height: 44,
    };
    assert!(x.can_hold(&x));
}
