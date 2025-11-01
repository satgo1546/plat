#!/bin/bash
set -e

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
3
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
