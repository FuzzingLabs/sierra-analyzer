<div align="center">

## Sierra Analyzer

Sierra Analyzer is a security toolkit designed for analyzing Sierra files. It includes: a decompiler, a call graph</br> generator, a control-flow graph generator, and various security detectors.
</div>

---

- [Project structure](#project-structure)
- [Decompile a Sierra file](#decompile-a-sierra-file)
- [Analyze a remote contract](#analyze-a-remote-contract)
- [Print the contract's Control-Flow Graph](#print-the-contracts-control-flow-graph)
- [Print the contract's Callgraph](#print-the-contracts-callgraph)
- [Run the detectors](#run-the-detectors)
- [Use the symbolic execution to generate unit tests](#use-the-symbolic-execution-to-generate-unit-tests)
- [Improve the decompiler output using LLMs](#print-the-contracts-callgraph)
- [Use it as a library](#print-the-contracts-callgraph)


### Project structure 

```
.
├── doc                  # Documentation files
├── examples             # Sierra & Contrat class samples files
├── lib                  # sierra-analyzer library
├── bin                  # Binaries directory containing Sierra decompiler tool (based on sierra-analyzer library) & Tests generator
└── README.md
```

### Decompile a Sierra file

```
cargo run -- -f <sierra file>
```

<p align="center">
	<b> Decompiler output  </b></br>
	<img height="400px" src="/doc/images/decompiler-output.png"/></br>
</p>

For a colourless output : 

```
cargo run -- -f <sierra file> --no-color
```

It it also possible to get a verbose output with more informations : 

```
cargo run -- -f <sierra file> --verbose
```

### Analyze a remote contract

Contracts can be fetched directly from Starknet (Mainnet & Sepolia) by specifying the contract class to analyze : 

```
# Fetch & decompile a contract from starknet mainnet 
cargo run -- --remote 0x07c43d18d37d66d7855dab8f21ebf9d554dd213c6307aacecaf2d595a53b3bbb

# Fetch & decompile a contract from Sepolia network
cargo run -- --network sepolia --remote 0x068377a89d64c0b16dc97c66933777bf4e9b050652c4fde2c59c8c4d755a163b
```

### Print the contract's Control-Flow Graph

```
cargo run -- -f ./examples/sierra/fib_array.sierra --cfg  

# Output the Control-Flow Graph to a custom folder (default is ./output_cfg)
cargo run -- -f ./examples/sierra/fib_array.sierra --cfg --cfg-output ./test 
```

<p align="center">
	<img src="/doc/images/cfg-output.png" height="400px"/>
</p>

### Print the contract's Callgraph

```
cargo run -- -f ./examples/sierra/fib_array.sierra --callgraph

# Output the Callgraph to a custom folder (default is ./output_callgraph)
cargo run -- -f ./examples/sierra/fib_array.sierra --callgraph --callgraph-output ./test 

# Get the Callgraph of a specific function
cargo run -- -f ./examples/sierra/fib_unary.sierra --callgraph --function 'examples::fib_unary::fib'
```

<p align="center">
	<img src="/doc/images/callgraph-output.png" height="400px"/>
</p>

### Run the detectors

```
cargo run -- -f ./examples/sierra/fib_array.sierra  -d
```

<p align="center">
	<img src="/doc/images/detectors-output.png" height="130px"/>
</p>

### Use the symbolic execution to generate unit tests

#### 1) Using the Tests generator detector

Symbolic execution can be used to generate unit tests for the functions that take `felt252` arguments as input. 

For example the file [symbolic_execution_test.sierra](https://github.com/FuzzingLabs/sierra-analyzer/blob/master/examples/sierra/symbolic_execution_test.sierra) contains a main function that takes four `felt252` arguments *v0*, *v1*, *v2* and *v3*. The function includes four conditions that check if `v0 == 102`, `v1 == 117`, `v2 == 122` and `v3 == 122` which correspond to the ASCII values for the letters *f*, *u*, *z*, and *z*, respectively.

When running the detectors we can generate test cases for each path in the function with the **Tests generator detector**:


```
cargo run -- -f ./examples/sierra/fib_array.sierra  -d

[...]

[Informational] Tests generator
        - symbolic::symbolic::symbolic_execution_test : 
        - v0: 102, v1: 0, v2: 0, v3: 0
        - v0: 103, v1: 0, v2: 0, v3: 0
        - v0: 102, v1: 117, v2: 0, v3: 0
        - v0: 0, v1: 118, v2: 0, v3: 0
        - v0: 102, v1: 117, v2: 122, v3: 0
        - v0: 0, v1: 0, v2: 123, v3: 0
        - v0: 102, v1: 117, v2: 122, v3: 122
        - v0: 0, v1: 0, v2: 0, v3: 123
```

#### 2) Using the library

The tests generator can also be used [with the library](https://github.com/FuzzingLabs/sierra-analyzer/blob/master/lib/examples/tests_generator.rs).

### Improve the decompiler output using LLMs

[Here](/doc/llm-decompilation.md) is a tutorial on how to improve the decompiler output using LLMs.

### Use it as a library 

It is also possible to use the `sierra-analyzer-lib` library to decompile serialised or unserialised Sierra files.

### Features

- [x] Decompiler
- [x] Control-Flow Graph
- [x] Call Graph
- [X] Informational & Security detectors
- [x] Fetching contracts from Starknet
- [x] Symbolic execution
