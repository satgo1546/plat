#!/bin/bash
set -e

node() {
  deno -A $*
}

diff <(node main.ts 2>&1 <(cat <<INPUT
print "scone" + (-4 * 5 - 1);
INPUT
)) - <<OUTPUT
scone-21
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
var a = "global a";
var b = "global b";
var c = "global c";
{
  var a = "outer a";
  var b = "outer b";
  {
    var a = "inner a";
    print a;
    print b;
    print c;
  }
  print a;
  print b;
  print c;
}
print a;
print b;
print c;
INPUT
)) - <<OUTPUT
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
var a = 1;
{
  var a = a + 2;
  print a;
}
INPUT
)) - <<OUTPUT
Error: local variable initializer accesses itself (line 3)
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
if (1 < 2)
if (1 > 2) print "NG";
else print "OK";
INPUT
)) - <<OUTPUT
OK
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
print "hi" or 2; // "hi".
print nil or "yes"; // "yes".
print 1.5 > 2.1 or 1 == 1 and 1 + 1 >= 2;
INPUT
)) - <<OUTPUT
hi
yes
true
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
var a = 0;
var temp;

for (var b = 1; a < 10000; b = temp + b) {
  print a;
  temp = a;
  a = b;
}
INPUT
)) - <<OUTPUT
0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
610
987
1597
2584
4181
6765
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
fun sayHi(first, last) {
  print "Hi, " + first + " " + last + "!";
}

sayHi("Dear", "Reader");

fun add(a, b, c) {
  print a + b + c;
}

add(1, 2, 3);

fun add(a, b) {
  print a + b;
}

print add; // "<fn add>".

fun count(n) {
  if (n > 1) count(n - 1);
  print n;
}

count(3);

fun procedure() {
  print "don't return anything";
}

var result = procedure();
print result; // ?

fun fib(n) {
  if (n <= 1) return n;
  return fib(n - 2) + fib(n - 1);
}

for (var i = 0; i < 20; i = i + 1) {
  print fib(i);
}

fun makeCounter() {
  var i = 0;
  fun count() {
    i = i + 1;
    print i;
  }

  return count;
}

var counter = makeCounter();
print counter;
counter(); // "1".
counter(); // "2".
INPUT
)) - <<OUTPUT
Hi, Dear Reader!
6
<fn add>
1
2
3
don't return anything
nil
0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
610
987
1597
2584
4181
<fn count>
1
2
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
var a = "global";
{
  fun showA() {
    print a;
  }

  showA();
  var a = "block";
  showA();
}
INPUT
)) - <<OUTPUT
global
global
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
fun bad() {
  var a = "first";
  var a = "second";
}
INPUT
)) - <<OUTPUT
Error: variable already declared (line 3)
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
return "at top level";
INPUT
)) - <<OUTPUT
Error: nowhere to return (line 1)
OUTPUT

diff <(node main.ts 2>&1 <(cat <<INPUT
class DevonshireCream {
  serveOn() {
    return "Scones";
  }
}

print DevonshireCream; // Prints "DevonshireCream".
class Bagel {}
var bagel = Bagel();
print bagel; // Prints "Bagel instance".
bagel.e = 114514;
print bagel.e;
print bagel.i;
INPUT
)) - <<OUTPUT
DevonshireCream
Bagel instance
114514
Runtime error: undefined property at \`i\` (line 13)
OUTPUT
