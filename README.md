### Sierra Analyzer

Sierra analyzer is a security toolkit for analyzing Sierra files.

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
cargo run --bin sierra-decompiler <sierra file>
```

<p align="center">
	<b> Decompiler output  </b></br>
	<img height="400px" src="/doc/images/decompiler-output.png"/></br>
</p>

For a colourless output : 

```
cargo run --bin sierra-decompiler <sierra file> --no-color
```

It it also possible to get a verbose output with more informations : 

```
cargo run --bin sierra-decompiler <sierra file> --verbose
```

#### Print the contract's Control-Flow Graph

```
cargo run ./examples/sierra/fib_array.sierra --cfg  

# Output the Control-Flow Graph to a custom folder (default is ./output_cfg)
cargo run ./tests/sierra_files/fib_array.sierra --cfg --cfg-output ./test 

# Get the CFG of a specific function
cargo run ./examples/sierra/fib_unary.sierra --cfg --function 'examples::fib_unary::fib'
```

<p align="center">
	<img src="/doc/images/cfg-output.png" height="400px"/>
</p>

#### Print the contract's Callgraph

```
cargo run ./examples/sierra/fib_array.sierra --callgraph

# Output the Callgraph to a custom folder (default is ./output_callgraph)
cargo run ./tests/sierra_files/fib_array.sierra --callgraph --callgraph-output ./test 

# Get the Callgraph of a specific function
cargo run ./examples/sierra/fib_unary.sierra --callgraph --function 'examples::fib_unary::fib'
```

<p align="center">
	<img src="/doc/images/callgraph-output.png" height="400px"/>
</p>

#### Use it as a library 

It is also possible to use the `sierra-analyzer-lib` library to decompile serialised or unserialised Sierra files.

Examples can be found [here](/lib/examples/).

#### Features

- [x] Decompiler
- [x] Control-Flow Graph
- [x] Call Graph
- [ ] Symbolic execution
- [ ] Security detectors
