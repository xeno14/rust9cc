#!/bin/bash

assert() {
  expected="$1"
  input="$2"

  ./target/debug/rust9cc "$input" > tmp.s
  cc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

cargo build || exit 1
cargo test || exit 1

assert 0 0
assert 42 42
assert 47 '5+6*7'
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'
assert 4 "5-(-1+2)"
assert 3 "+5+(-2)"

echo OK