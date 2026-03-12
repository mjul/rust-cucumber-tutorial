# Cucumber Tutorial for Rust

This project shows you how to set up and use Cucumber for Rust.
It is based on my [Cucumber Tutorial for Clojure](https://github.com/mjul/cucumber-tutorial)

Cucumber is a language for writing executable specifications for software.
The `cucumber` crate provides a Rust library for running Cucumber specifications,
see the [code](https://github.com/cucumber-rs/cucumber)
and [documentation](https://cucumber-rs.github.io/cucumber/current/).

## Quick Start

Build:

```shell
    cargo build
```

Run tests:

```shell
   cargo test
```

Run Cucumber specifications:

```shell
cargo test --test cucumber_specs
```

## Usage

The project includes an example specification in the `features`
folder, and the step definition that binds the feature file to
executable test code in the `features/step_definitions` folder.

For example, to test that we can open a position in a currency trading
application, you could write a feature like this:

    Feature: Open Position
      In order to open a position
      As a trader
      I want to send a trade order
    
      Scenario: Market Order
        Given that my position in EURUSD is 0 at 1.34700
        And the market for EURUSD is at [1.34662;1.34714]
        When I submit an order to BUY 1000000 EURUSD at MKT
        Then a trade should be made at 1.34714
        And my position should show LONG 1000000 EURUSD at 1.34714

That is you specification. Now add step definitions to the
`tests/cucumber_specs.rs` file to connect the specification
mini-language defined by the above to code. We do this by matching
a regex to the "given" text with a function of the values matched by the regex,
_e.g._

```rust
#[given(regex = r"^that my position in (\w{6}) is (\d+) at ([\d.]+)$")]
fn my_initial_position_is_en(world: &mut TradingWorld, cross: String, qty: i64, price: String) {
    // set up the state accordingly...
    my_initial_position_is(world, cross, qty, parse_price_en(&price));
}
```

### Specifications in your own Language

You can define Cucumber features in many languages. Here is the Danish
version of the example above, from the file `features/open_position_da.feature`.
Note the `#language: da` declaration on the first line:

    #language: da
    Egenskab: Åbn position
      For at åbne en position
      Som en valutahandler
      Ønsker jeg at afgive en handelsordre
    
      Scenarie: Markedsordre
        Givet at min position i EURUSD er 0 købt til kurs 1,34700
        Og markedsprisen for EURUSD er [1,34662;1,34714]
        Når jeg afgiver en ordre om at KØBE 1000000 EURUSD til MARKEDSPRIS
        Så skal en handel ske til kurs 1,34714
        Og min position skal være LANG 1000000 EURUSD købt til kurs 1,34714

### Test the same Scenario with Multiple Examples

You can create a template, called a Scenario Outline, and have
Cucumber evaluate it with different sets of values substituted into
the template fields. The sets of values are called Examples.

For example, to evaluate selling euro-dollar at various price points
use the following Scenario Outline from the file
`features/open_position.feature`:

      Scenario Outline: Market Order SELL
        Given that my position in EURUSD is 0 at 1.34700
        And the market for EURUSD is at [<bid>;<ask>]
        When I submit an order to SELL <quantity> EURUSD at MKT
        Then a trade should be made at <bid>
        And my position should show SHORT <quantity> EURUSD at <bid>
    
        Examples:
          |  bid     | ask     | quantity |
          |  1.34662 | 1.34714 | 1000000  |
          |  1.40000 | 1.40050 | 1000000  |

### Use Tables of Values in Specifications

You can pass tabular data to your step definitions using a `cucumber::gherkin::Step`
parameter. This is useful for setting up context or verifying multiple correlated assertions.

For example, if we want to put conditional exits on a currency
position we can create two orders to take profit if the market rises
or limit the loss if the price falls respectively. These are called
LIMIT and STOP orders and they should be of the OCO-type, meaning that
one cancels the other: if either one is triggered the other one should
be cancelled.

See the file `features/conditional_order.feature` for an example:

    Feature: Conditional Order
      In order to guard my positions
      As a trader
      I want to send a trade order with conditional stop loss and take profit orders.
    
      Scenario: Market Order with Take Profit and Stop Loss guards
        Given that my position in EURUSD is 0 at 1.34700
        And the market for EURUSD is at [1.34662;1.34714]
        And I have no open orders in EURUSD
        When I submit an order to BUY 1000000 EURUSD at MKT with TARGET 1.3800 and STOP 1.3200
        Then a trade should be made at 1.34714
        And my position should show LONG 1000000 EURUSD at 1.34714
        And my open orders should contain these OCO-orders
          | Side | Quantity | Cross  | Type  | Price  | 
          | SELL | 1000000  | EURUSD | LIMIT | 1.3800 | 
          | SELL | 1000000  | EURUSD | STOP  | 1.3200 |

See the `tests/cucumber_specs.rs` file for an
example of how to use it to write the step definitions.

We use a helper function is useful for extracting the values from
the table into a sequence of maps:

```rust
/// Translate a table to a vector of `HashMap`, one for each row, where the keys are the
/// column names from the header row and the values are the values in the data row.
fn table_to_hash_maps(table: &Table) -> Vec<HashMap<String, String>> {
    match table.rows.as_slice() {
        [] => vec![],
        [_headers] => vec![],
        [headers, data @ ..] => {
            // The table is a list of rows, every row is a list of fields (strings)
            // Translate it to a list of keyed maps (one per row, excluding the header),
            // using the headline value as the key for each field
            data.iter()
                .map(|row| {
                    headers
                        .iter()
                        .cloned()
                        .zip(row.iter().cloned())
                        .collect::<HashMap<String, String>>()
                })
                .collect()
        }
    }
}
```

## Installation

Now, `cargo test --test cucumber_specs` will run the Cucumber tests.

    cargo test --test cucumber_specs

You should now see something like this:

```
```Feature: Open Position
Scenario: Market Order BUY
✔  Given that my position in EURUSD is 0 at 1.34700
✔  And the market for EURUSD is at [1.34662;1.34714]
✔  When I submit an order to BUY 1000000 EURUSD at MKT
✔  Then a trade should be made at 1.34714
✔  And my position should show LONG 1000000 EURUSD at 1.34714
Scenario Outline: Market Order SELL
✔  Given that my position in EURUSD is 0 at 1.34700
✔  And the market for EURUSD is at [1.34662;1.34714]
✔  When I submit an order to SELL 1000000 EURUSD at MKT
✔  Then a trade should be made at 1.34662
✔  And my position should show SHORT 1000000 EURUSD at 1.34662
Scenario Outline: Market Order SELL
✔  Given that my position in EURUSD is 0 at 1.34700
✔  And the market for EURUSD is at [1.40000;1.40050]
✔  When I submit an order to SELL 1000000 EURUSD at MKT
✔  Then a trade should be made at 1.40000
✔  And my position should show SHORT 1000000 EURUSD at 1.40000
[Summary]
1 feature
3 scenarios (3 passed)
15 steps (15 passed)
Egenskab: Åbn position
Scenarie: Markedsordre
✔  Givet at min position i EURUSD er 0 købt til kurs 1,34700
✔  Og markedsprisen for EURUSD er [1,34662;1,34714]
✔  Når jeg afgiver en ordre om at KØBE 1000000 EURUSD til MARKEDSPRIS
✔  Så skal en handel ske til kurs 1,34714
✔  Og min position skal være LANG 1000000 EURUSD købt til kurs 1,34714
[Summary]
1 feature
1 scenario (1 passed)
5 steps (5 passed)
Feature: Conditional Order
Scenario: Market Order with Take Profit and Stop Loss guards
✔  Given that my position in EURUSD is 0 at 1.34700
✔  And the market for EURUSD is at [1.34662;1.34714]
✔  And I have no open orders in EURUSD
✔  When I submit an order to BUY 1000000 EURUSD at MKT with TARGET 1.3800 and STOP 1.3200
✔  Then a trade should be made at 1.34714
✔  And my position should show LONG 1000000 EURUSD at 1.34714
✔  And my open orders should contain these OCO-orders
| Side | Quantity | Cross  | Type  | Price  |
| SELL | 1000000  | EURUSD | LIMIT | 1.3800 |
| SELL | 1000000  | EURUSD | STOP  | 1.3200 |
[Summary]
1 feature
1 scenario (1 passed)
7 steps (7 passed)
```

### Setting Up a Project for Cucumber

See the Rust Cucumber book for documentation: https://cucumber-rs.github.io/cucumber/current/introduction.html
First, edit the `Cargo.toml` file to include the Cucumber library and point to the top-level test file. The salient
parts are these:

```toml
[dev-dependencies]
cucumber = { version = "0.22.1" }
tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread"] }

[[test]]
name = "cucumber_specs" # this should be the same as the filename of your test target
harness = false  # allows Cucumber to print output instead of libtest
```

Here we use a `featurse` folder for the specifications in
the project root:

```shell
make -p features/
```    

As you write your code, put the feature definitions in the `features`
folder and the step definitions that link them to your code in the
`tests/cucumber_specs.rs` file.

On bigger projects, break out a `step_definitions` module to keep the step definitions
well organised.

## Further Research

For file system layout I followed the Clojure layout, it would be useful
to explore this further and find an idiomatic Rust layout that fits with the
configuration in `Cargo.toml` and the CLI options, _e.g._ you can use an input glob to
select which features to run, `-i`, but then we probably have to
write the `main` function in a way where it does not pull in everything

```shell
cargo test --test cucumber_specs -- -i 'features/open_position_da.feature' 
```

## License

Distributed under the MIT License. See the LICENSE file for details.
