# emergence

Emergence is a language for writing genetic circuits. The program is written in form of logic functions, which describe boolean logic between variables and produce an output. These functions together form a abstract logic circuts which is then converted to genetic circut that consists of genetic gates.

## Program

Emergence program can consist of 2 entities: **Functions** or **Genes**.

**Functions** describe interaction between abstract variables and produce abstract output. By abstract we mean that they can be replaced by any value, and that value is assigned by the compiler.

```rust
fn not a -> b {
    b = ~a;
}
```

Functions can use boolean operation or they can call other functions:
```rust
fn nor(a, b) -> c {
    let orab = a | b;
    c = not(orab);
}
```

**Genes** on the other hand for an input take **values**, and values can be proteins or signaling molecules. You can think of them as events. When **x** signal or protein concentration changes, the output protein **y** concentretation changes.

```rust
// the red fluorescent protein is synthesised when there is a low concentration of lactose
gene blue LacI -> RFP {
    RFP = not(LacI);
}
```

## Assigner

After parsing a program into a parse three, the compiler creates and abstract logic circut. E.g:

```rust
gene main (TetR, LacI) -> RFP {
	RFP = TetR ~| LacI;
}
```
compiles to:

<img src="./images/NOR.svg" width="300" />

which then gets converted to genetic gates:

<img src="./images/bio-gate-example.svg" width="600" />

Assigner is using KdTree search algorithm to find genetic gates with most similar response functions. Response function describes the expression of a protein based on concentration of inputs. There are two modes: normal and strict. Normal mode is optimized for assigning more gates, but with possibility that these gates are not the best solution. Strict mode will always assign the best gates, but with a change of failing to assign all of them if circuts are big.
