# Examples

In this directory, we provide examples of expected extracted values and transformed values.

## Organization

Here we present the expected organization of the example data.

Each directory in this directory represents an unique test, identified by its directory name.  The test names should coincide with their purpose (i.e. Genesis for the Genesis transaction), and should be followed by the range that it includes (start and end inclusive)

Each test should have a `txs` directory containing a `#.pb` file, where the pound symbol represents the transaction version number.  Each directory should also have a `records` directory which should contain the serialized Record structure as the name `#.pb` matching the transaction from `txs`.

```text
tests/examples/
    |- genesis_0_0/
        |- txs/
            |- 0.pb
        |- records/
            |- 0.pb
    |- (other example test identifier)_1_3/
        |- txs/
            |- 1.pb
            |- 2.pb
            |- 3.pb
        |- records/
            |- 1.pb
            |- 2.pb
            |- 3.pb
    |- (other example test)_start_end/
        |- txs/
            |- start.pb
            |- ...
            |- end.pb
        |- records/
            |- start.pb
            |- ...
            |- end.pb
```
