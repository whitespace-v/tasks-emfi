### Task 3: Implement a Multi-threaded Matrix Multiplication

Description: Write a Rust program to multiply two matrices in parallel using multiple threads. This task is intended to introduce the intern to Rust's concurrency model and memory safety features.

Requirements:
- The program should take two matrices as input (you can hard-code them initially or read them from a file).
- Implement matrix multiplication using multiple threads, where each thread computes a part of the result matrix.
- Ensure the program is thread-safe and does not cause data races or memory leaks.
- Print the resulting matrix.

Learning Objectives:
- Understand Rust's concurrency model and thread safety.
- Learn about the `std::thread` module and how to use threads in Rust.
- Practice handling shared mutable state using `Arc` (Atomic Reference Counting) and `Mutex`.

---

### General Tips for Interns:
- Provide access to Rust's official documentation and the Rust book: [The Rust Programming Language](https://doc.rust-lang.org/book/).
- Encourage the interns to write unit tests for their code to understand Rust's testing framework.
- Suggest using `cargo` for project management, building, and running tests.
