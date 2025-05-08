# AI Behavior Guide for Hyperplane

## General
- Always consider the plan in PLAN.md
- make an integration test that tests the overall protocol as we develop it.

## Code Style
- Use `///` for public API docs, `//` for inline comments
- Use `thiserror` for error types, `anyhow` for application errors
- Derive `Debug`, `Clone`, `Serialize`/`Deserialize` where appropriate
- Use `async_trait` for async traits, document cancellation behavior

## Project Structure
- Keep modules focused and single-purpose
- Use `mod.rs` for module declarations
- Place tests in `#[cfg(test)]` modules
- Use `examples/` for example code

## Documentation
- Document all public APIs with examples
- Keep README.md up to date
- Document architecture decisions
- Include setup instructions

## Testing
- Write unit tests for all public APIs
- Above each test give a short description of what it is testing
- Test both success and failure cases
- Use `#[tokio::test]` for async tests
- Document test prerequisites

## Error Handling
- Use clear, actionable error messages
- Include relevant context in errors
- Log relevant state for debugging
- Use appropriate log levels

## Security
- Validate all inputs
- Use TLS for network connections
- Follow security best practices
- Document security considerations 