# A DuckDB Extension To Launch A Web Server

## Build

```sh
make configure
make
```

On Windows, be aware that...

- This doesn't work on PowerShell or cmd.exe. Open WSL, Git Bash, or something.
- This assumes Python bin is `python3`. You'll probably need to specify `PYTHON_BIN`.

```sh
PYTHON_BIN=python make configure
PYTHON_BIN=python make
```

## Test

Launch DuckDB with `-unsigned` option.

```sh
duckdb -unsigned
```

Then, load the extension.

```sh
LOAD './build/debug/rusty_quack.duckdb_extension';
```

After that, you can use `rusty_quack()`, which launches a web server and open the URL with web browser.

```
D CALL rusty_quack();
┌───────────────────────────┐
│          column0          │
│          varchar          │
├───────────────────────────┤
│ URL http://127.0.0.1:3030 │
└───────────────────────────┘
```