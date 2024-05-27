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
cargo run --bin sierra-decompiler -f <sierra file>
```

<p align="center">
	<b> Decompiler output  </b></br>
	<img height="400px" src="/doc/images/decompiler-output.png"/></br>
</p>

For a colourless output : 

```
cargo run --bin sierra-decompiler -f <sierra file> --no-color
```

It it also possible to get a verbose output with more informations : 

```
cargo run --bin sierra-decompiler -f <sierra file> --verbose
```

#### Analyze a remote contract

Contracts can be fetched directly from the Starknet (Mainnet & Sepolia) by specifying the contract class to analyze : 

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
cargo run -- -f ./tests/sierra_files/fib_array.sierra --cfg --cfg-output ./test 
```

<p align="center">
	<img src="/doc/images/cfg-output.png" height="400px"/>
</p>

#### Use it as a library 

It is also possible to use the `sierra-analyzer-lib` library to decompile serialised or unserialised Sierra files.

Examples can be found [here](/lib/examples/).

#### Features

- [x] Decompiler
- [x] Control-Flow Graph
- [ ] Call Graph
- [ ] Security detectors
