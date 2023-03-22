from wasmer import engine, Store, Module, Instance


def read_wasm_file(filename):
    with open(filename, 'rb') as file:
        wasm_bytes = file.read()
    return wasm_bytes


store = Store()

module = Module(store, read_wasm_file('../wasm_pest.wasm'))
instance = Instance(module)

# Call the exported `sum` function.
result = instance.exports.sum(5, 37)

print(result)  # 42!
