package main

import (
	"fmt"
	"io/ioutil"

	wasmer "github.com/wasmerio/wasmer-go/wasmer"
)

func main() {
	wasmBytes, _ := ioutil.ReadFile("../pest.wasm")

	engine := wasmer.NewEngine()
	store := wasmer.NewStore(engine)

	// Compiles the module
	module, _ := wasmer.NewModule(store, wasmBytes)

	// Instantiates the module
	importObject := wasmer.NewImportObject()
	instance, _ := wasmer.NewInstance(module, importObject)

	// Gets the `sum` exported function from the WebAssembly instance.
	compile_grammer, _ := instance.Exports.GetFunction("compile_grammer")

	// Calls that exported function with Go standard values. The WebAssembly
	// types are inferred and values are casted automatically.
	compile_grammer("alpha = { 'a'..'z' | 'A'..'Z' }")

	parse_input, _ := instance.Exports.GetFunction("parse_input")

	result, _ := parse_input("alpha", "a")
	fmt.Println(result) // 42!
}
