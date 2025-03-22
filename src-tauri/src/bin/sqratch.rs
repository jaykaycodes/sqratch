use sqratch_lib;

// CLI entry point for the Sqratch application
// This binary will be installed and accessible as 'sqratch' from the terminal

fn main() {
    // Pass control to the main library code
    // This ensures that the CLI and GUI share the same codebase
    sqratch_lib::run();
}
