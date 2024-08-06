### Sierra Analyzer

Sierra-Analyzer is a security toolkit for analyzing Sierra files.

1) [Project structure](#project-structure)
2) [Decompile a Sierra file](#decompile-a-sierra-file)
3) [Analyze a remote contract](#analyze-a-remote-contract)
4) [Print the contract's Control-Flow Graph](#print-the-contracts-control-flow-graph)
5) [Print the contract's Callgraph](#print-the-contracts-callgraph)
6) [Run the detectors](#print-the-contracts-callgraph)
7) [Use it as a library](#print-the-contracts-callgraph)
8) [Improve the decompiler output using LLMs](#print-the-contracts-callgraph)

#### Project structure 

```
.
├── doc               		 # Documentation files
├── examples                     # Sierra & Contrat class samples files
├── lib               	         # sierra-analyzer library
├── sierra-decompiler            # Sierra decompiler tool (based on sierra-analyzer library)
└── README.md
```

#### Decompile a Sierra file

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

#### Analyze a remote contract

Contracts can be fetched directly from Starknet (Mainnet & Sepolia) by specifying the contract class to analyze : 

```
# Fetch & decompile a contract from starknet mainnet 
cargo run -- --remote 0x07c43d18d37d66d7855dab8f21ebf9d554dd213c6307aacecaf2d595a53b3bbb

# Fetch & decompile a contract from Sepolia network
cargo run -- --network sepolia --remote 0x068377a89d64c0b16dc97c66933777bf4e9b050652c4fde2c59c8c4d755a163b
```

#### Print the contract's Control-Flow Graph

```
cargo run -- -f ./examples/sierra/fib_array.sierra --cfg  

# Output the Control-Flow Graph to a custom folder (default is ./output_cfg)
cargo run -- -f ./examples/sierra/fib_array.sierra --cfg --cfg-output ./test 
```

<p align="center">
	<img src="/doc/images/cfg-output.png" height="400px"/>
</p>

#### Print the contract's Callgraph

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

#### Run the detectors

```
cargo run -- -f ./examples/sierra/fib_array.sierra  -d
```

<p align="center">
	<img src="/doc/images/detectors-output.png" height="130px"/>
</p>

#### Use it as a library 

It is also possible to use the `sierra-analyzer-lib` library to decompile serialised or unserialised Sierra files.

Examples can be found [here](/lib/examples/).

#### Improve the decompiler output using LLMs

[Here](/doc/llm-decompilation.md) is a tutorial on how to improve the decompiler output using LLMs.

#### Features

- [x] Decompiler
- [x] Control-Flow Graph
- [x] Call Graph
- [X] Informational & Security detectors
- [x] Fetching contracts from Starknet
- [x] Symbolic execution
