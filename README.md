# Terminal Input Handling (In-Keys)

This crate provides functions for handling input from the terminal, with a focus on providing
both synchronous and asynchronous options. It uses low-level system calls for efficient input
polling and processing. Minimal support for output streams exist as well. 

## Synchronous Input

Synchronous input functions allow for blocking operations where the program waits for user input.
These functions include reading a single key, processing escape sequences for special keys, and
reading strings of characters terminated by the Enter key.

## Asynchronous Input

Asynchronous input functions use a polling mechanism to check for available input without blocking
the program's execution. These functions can be useful in scenarios where the program needs to
remain responsive to other tasks or events while still monitoring for user input.

## Example Usage

```rust,ignore
use in_keys::Terminal;
use std::time::Duration;
use tokio::time;

let terminal = Terminal::new();

// Synchronous input
let key = terminal.read_key(); // Blocking, waits for user input

// Asynchronous input
let mut lock = terminal.lock_stdin().expect("Failed to connect with terminal");
let future_key = lock.read_key_future(); // Returns a future immediately
let result = time::timeout(Duration::from_secs(5), future_key).await;

match result {
    Ok(Ok(key)) => println!("Received key: {:?}", key),
    Ok(Err(error)) => eprintln!("Error reading key: {}", error),
    Err(_) => eprintln!("Timed out waiting for input"),
}
```

## Notes

- This module utilizes low-level system calls and may not be portable across all platforms.
  Only Linux is truly supported as of the current moment.
- Care should be taken when using asynchronous input, as it may introduce additional complexity
  and overhead.
