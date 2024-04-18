### Sierra Analyzer

Sierra analyzer is a security toolkit for analyzing Sierra files.

#### Project structure 

```
.
├── cairo              	         # Cairo repository
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
	<img src="/doc/images/images/decompiler-output.png"/></br>
</p>

For a colourless output : 

```
cargo run --bin sierra-decompiler <sierra file> --no-color
```

#### Use it as a library 

It is also possible to use the `sierra-analyzer-lib` library to decompile serialised or unserialised Sierra files.

Examples can be found [here](/lib/examples/).

#### Features

- [x] Decompiler
- [ ] Control-Flow Graph
- [ ] Call Graph
- [ ] Security detectors