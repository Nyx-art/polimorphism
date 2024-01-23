# polymorphism
#### A procedural macro to imitate ad hoc polymorphism (function overloading) which can be seen and found in many modern programming languages. Can be used similarly to an `fn` or `impl` declaration, but `polymorphism` allows for duplicate `fn` names with different signitures (types as parameters). This implementation of `polymorphism` bypasses the orphan rule with a `Local` type.
---
## Example
```
polymorphism!(
    pub fn func(n: i32, m: i32) -> i32 {
        n+m
    }
    pub fn func(n: f64, m: f64) -> f64 {
        n-m
    }
);

assert_eq!(polymorphism!(func(1,2)), 3);
assert_eq!(polymorphism!(func(1.0,2.0)), -1.0);
```
## Notes:
- This is a proof of concept, therefore it is REALLY unstable
- It is nearly untested and may break on edge cases
- Do NOT use it in production codebases at this time
- Feedback is appreciated :)