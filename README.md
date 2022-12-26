JVM powered by Rust: experimental
======
**Important:** work in progress.

This repository contains code for an experimental implementation of the Java Virtual Machine 1.2 standard 
(referred to as JVM). This implementation is **not** entirely compliant, complete or efficient.

The goal of this project is simply to keep me entertained (these things relax
me outside of working hours) and to have some playground to test Rust.
The goal of this project is simply to keep me entertained 
(these things relax me outside of working hours) and to have some playground
to test Rust. Rust is still a great language, but I keep discovering more 
reasons to hate it or notice that C++ did a lot of things better 20 years ago.

Used specification [here](https://docs.oracle.com/javase/specs/jvms/se6/html/VMSpecTOC.doc.html).

Due to the choice of a very old standard, you need to pay special attention to whether they are actually compiled for 
this and not a higher version of the jvm.