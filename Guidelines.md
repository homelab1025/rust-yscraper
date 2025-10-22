## Guidelines
### Formatting
- Formatting and naming based on the regular conventions of the language.. For example for Java it's CamelCase, whereas for Rust it's snake_case.
### Structure
- Try to make the classes or functions generic if it makes sense to reuse them in various points of the code. Otherwise don't generalize so to avoid increase in complexity.
- Prefer composition over inheritance.
- Separate multi-threading code from the actual functionality.
- Avoid rigidity: a small change causes a cascade of changes.
- Single Responsibility principle: functions should only do one thing and avoid side-effects on the parameters.
- Use Dependency Inversion Principle
- User Interface Segregation Principle
- Make dependencies explicit
### Easy to understand code
- Use explanatory naming.
- Avoid logical dependency: methods shouldn't work correctly depending on state somewhere else in the code.
- Avoid negative conditionals
- Use the simplest solution that could possibly work
### Way of working
- Use git
- generate git commits with meaningful messages
- do a git commit after every meaningful change that adds value to the code