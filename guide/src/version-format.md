<!-- description: The API version syntax: version groups, major and minor numbers, and optional status. -->

# Version Format

Services are versioned using a version group (e.g. date) or major and minor version scheme with an optional status. The
version format has the following syntax:

```ebnf
letter = "A" | "B" | "C" | "D" | "E" | "F" | "G"
       | "H" | "I" | "J" | "K" | "L" | "M" | "N"
       | "O" | "P" | "Q" | "R" | "S" | "T" | "U"
       | "V" | "W" | "X" | "Y" | "Z" | "a" | "b"
       | "c" | "d" | "e" | "f" | "g" | "h" | "i"
       | "j" | "k" | "l" | "m" | "n" | "o" | "p"
       | "q" | "r" | "s" | "t" | "u" | "v" | "w"
       | "x" | "y" | "z" ;

positive = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;

digit = "0" | positive ;

day = ( [ "0" ] positive ) | ( "1" | "2" ) digit | ( "3" ( "0" | "1" ) ) ;

month = ( [ "0" ] positive ) | ( "1" ( "0" | "1" | "2" ) ) ;

year = 4 * digit ;

group = year "-" month "-" day ;

version = { digit } [ "." { digit } ] ;

status = letter [ { letter | digit | "." } { letter | digit } ] ;

api-version = ( group | version ) [ "-" status ] ;

```

The version status allows you to provide a condition to a version such as **alpha**, **beta**, **rc**, and
so on.  While the status is optional, either the version group or the major and minor versions must be specified.

## Versioned Request

By default, clients must explicitly request the version of a service via the **api-version** query string parameter or
URL path segment. It is possible to customize this behavior for legacy and other non-compliant services, which will be
covered in the **Advanced Versioning** topic.

> [!NOTE]
> When versioning by URL segment, the `v` prefix is neither required nor part of the API version.

## Versioned Request Examples

The following outlines examples of various service version formats:

- /api/foo?api-version=1.0
- /api/foo?api-version=2.0-alpha
- /api/foo?api-version=2015-05-01
- /api/v1/foo
- /api/v2.0-alpha/foo
- /api/v2015-05-01/foo

## Formatting

`ApiVersion` supports `Display`, which round trips back to the original text format. It also support an alternate format
that omits the minor version for numeric versions when the value is `0`.

```rust
println!("Welcome to version {:#}", api_version!(1.0));
println!("Welcome to version {}", api_version!(1.1-beta));
println!("Welcome to version {}", api_version!(2.0));

// Output: Welcome to version 1
// Output: Welcome to version 1.1-beta
// Output: Welcome to version 2.0
```
