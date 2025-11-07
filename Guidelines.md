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
- Make dependencies explicit.
- Errors should be returned and handled in a rust-specific manner.
### Easy to understand code
- Use explanatory naming.
- Avoid logical dependency: methods shouldn't work correctly depending on state somewhere else in the code.
- Avoid negative conditionals
- Use the simplest solution that could possibly work
### Way of working
- Use git
- generate git commits with meaningful messages from a functionality perspective. If it's simply a technical change, explain why it's needed.
- do the changes on the files and prepare a commit message for them. Present me with the commit message and I will review the changes and the messages. Ask me if it's ok to commit and only after i say yes you do the commit. 