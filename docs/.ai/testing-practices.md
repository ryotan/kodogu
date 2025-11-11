# Testing Practices

## Universal Principles

### Tests as Documentation

- Tests describe how the system should behave
- Names explain the "what" and "why", not the "how"
- A failing test should clearly indicate what broke
- **Why**: Tests are living documentation that never goes out of date

### Fast Feedback

- Tests should run quickly
- Failures should be easy to diagnose
- Optimize for developer productivity
- **Why**: Slow tests don't get run, slowing down development

### Independent Tests

- Each test runs in isolation
- No shared state between tests
- Order of execution doesn't matter
- **Why**: Prevents cascading failures and flaky tests

## Test Structure

### AAA Pattern (Arrange-Act-Assert)

- **Arrange**: Set up test data and conditions
- **Act**: Execute the behavior being tested
- **Assert**: Verify the outcome
- **Why**: Consistent structure makes tests easier to read and maintain

### Single Responsibility

- One test verifies one behavior
- Focus on one assertion concept (may have multiple related assertions)
- Don't test multiple scenarios in one test
- **Why**: When a test fails, you immediately know what broke

### Clear Test Names

- Describe behavior, not implementation
- Be specific about the scenario
- **Why**: Test names serve as executable specifications

## Test Levels

### Unit Tests

- Test individual components in isolation
- Fast execution (milliseconds)
- No external dependencies
- **Why**: Rapid feedback on component correctness

### Integration Tests

- Test interactions between components
- May involve databases, files, or other systems
- Slower than unit tests, faster than E2E
- **Why**: Verify components work together correctly

### End-to-End Tests

- Test complete user scenarios
- Through the actual UI/API
- Slowest but most realistic
- **Why**: Verify the system works from user perspective

### Test Pyramid

- Many unit tests (base)
- Fewer integration tests (middle)
- Few E2E tests (top)
- **Why**: Balances coverage, speed, and maintenance cost

## Boundary Testing

### Test at Boundaries

- Valid inputs at edges (min, max)
- Invalid inputs just outside boundaries
- Null, empty, and special values
- **Why**: Bugs cluster at boundaries

### Equivalence Partitioning

- Divide input space into classes
- Test one value from each class
- Don't exhaustively test similar values
- **Why**: Efficient coverage without redundancy

## Test Data

### Meaningful Test Data

- Use realistic data that tells a story
- Avoid magic numbers without context
- Make relationships between data clear
- **Why**: Tests are easier to understand and maintain

### Minimal Test Data

- Include only what's necessary for the test
- Don't copy production data dumps
- Avoid irrelevant details
- **Why**: Reduces noise and highlights what matters

### Data Builders

- Use builders/factories/fixtures for complex objects
- Set only the fields relevant to the test
- Provide sensible defaults
- **Why**: Tests focus on what's being tested, not setup

## Invariant Testing

### Always Test Invariants

- Business rules that must never be violated
- Test both positive cases (allowed) and negative cases (rejected)
- Verify invariants are enforced at creation and modification
- **Why**: Invariants are the core of domain integrity

### Property-Based Testing

- Define properties that should always hold
- Generate random test cases
- Find edge cases you didn't think of
- **Why**: Discovers bugs that example-based tests miss

## Test Doubles

### Use Test Doubles Appropriately

- **Stub**: Provides canned answers (for queries)
- **Mock**: Verifies interactions (for commands)
- **Fake**: Working implementation (for complex dependencies)
- **Why**: Different needs require different approaches

### Don't Mock Value Objects

- Value objects are data, not behavior
- Use real instances in tests
- Only mock stateful dependencies
- **Why**: Over-mocking makes tests brittle

### Don't Mock What You Don't Own

- Don't mock external libraries or frameworks
- Use adapters and test the adapter
- **Why**: Tests shouldn't break when external APIs change

## Assertion Principles

### Clear Assertions

- Assert on specific values, not generic conditions
- Use domain-meaningful comparisons
- Provide helpful failure messages
- **Why**: Failures should pinpoint the problem

### Avoid Assertion Roulette

- Don't have dozens of unrelated assertions
- Group related assertions logically
- Consider separate tests for unrelated checks
- **Why**: Know which assertion failed and why

### Test Outcomes, Not Implementation

- Assert on observable behavior
- Don't assert on internal state unless necessary
- Avoid testing private methods directly
- **Why**: Tests survive refactoring

## Error Testing

### Test Error Conditions

- Invalid inputs should be rejected
- Verify error messages are helpful
- Test error recovery mechanisms
- **Why**: Error handling is part of the behavior

### Expected Exceptions

- Use explicit exception assertions
- Verify exception type and message
- Don't catch-and-ignore
- **Why**: Proves the right error occurs for the right reason

## Test Maintainability

### DRY in Tests (Carefully)

- Extract common setup to helper methods
- Don't abstract away the story the test tells
- Balance readability vs repetition
- **Why**: Tests should be easy to understand even with some duplication

### Avoid Test Interdependence

- Don't rely on test execution order
- Each test creates its own world
- Clean up after yourself
- **Why**: Prevents cascading failures and flakiness

### Delete Obsolete Tests

- Remove tests for removed features
- Don't keep broken tests "for later"
- Refactor tests when code changes
- **Why**: Dead tests create maintenance burden

## Testing Strategy

### Prioritize Testing Pure Functions

- Unit test pure functions first and most thoroughly
- Pure functions are easiest to test
- Provide highest value-to-effort ratio
- **Why**: Maximum coverage with minimal setup

### Test Repositories with In-Memory Implementations

- Create in-memory versions of repositories for testing
- Test domain logic without database dependencies
- Fast, reliable, and isolated tests
- **Why**: Tests run quickly and don't require external services

### Build Testability into Design

- Consider testing needs during design phase
- Design interfaces that are easy to mock
- Inject dependencies explicitly
- **Why**: Testable design is good design

### Assert-First Development

- Start by writing the assertion
- Work backwards to determine setup
- Clarifies what you're actually testing
- **Why**: Forces clear thinking about expected outcomes

## Cross-Cutting Concerns

### Test Performance Requirements

- Set acceptable thresholds
- Test under realistic load
- Monitor resource usage
- **Why**: Performance is a feature, not an afterthought

### Test Security Requirements

- Verify authentication and authorization
- Test input validation and sanitization
- Check for common vulnerabilities
- **Why**: Security bugs are expensive

### Test Accessibility

- Verify keyboard navigation
- Check screen reader compatibility
- Test color contrast and text sizing
- **Why**: Accessibility is a requirement, not optional

## Test Evolution

### Refactor Tests

- Keep tests clean and maintainable
- Apply same standards as production code
- Improve test clarity over time
- **Why**: Tests are code too

### Test-Driven Development

- Write test before implementation
- See it fail, make it pass, refactor
- Design emerges from tests
- **Why**: Better design and guaranteed testability

### The Red-Green-Refactor Cycle

- **Red**: Write a failing test that describes desired behavior
- **Green**: Write minimal code to make the test pass
- **Refactor**: Improve code quality without changing behavior
- **Why**: Small iterations with continuous validation

### Treat Tests as Specifications

- Tests document what the code should do
- Test names describe behavior, not implementation
- Tests serve as living documentation
- **Why**: Tests communicate intent to future developers

### Iterate in Small Units

- Make one small change at a time
- Run tests frequently during development
- Keep each cycle short (minutes, not hours)
- **Why**: Fast feedback loop catches errors early

### Continuous Refactoring

- Improve design throughout development
- Refactor when tests are green
- Keep code clean from the start
- **Why**: Prevents technical debt accumulation

### Listen to Your Tests

- Hard-to-test code signals design problems
- Lots of mocks suggest tight coupling
- Slow tests suggest architectural issues
- **Why**: Tests provide feedback on design quality
