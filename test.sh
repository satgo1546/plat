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
