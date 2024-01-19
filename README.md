# Polimorphism
A procedural macro to imitate ad hoc polimorphism (function overloading) which can be seen and found in many modern programming languages. Can be used similarly to an `fn` or `impl` declaration, but `polimorphism` allows for duplicate `fn` names with different signitures (types as parameters). This implementation of `polimorphism` bypasses the orphan rule with a `Local` type.
---
## Example
```
polimorphism!(
    pub fn func(n: i32, m: i32) -> i32 {
        n+m
    }
    pub fn func(n: f64, m: f64) -> f64 {
        n-m
    }
);

assert_eq!(polimorphism!(func(1,2)), 3);
assert_eq!(polimorphism!(func(1.0,2.0)), -1.0);
```
## Notes:
- This is a proof of concept, therefore it is REALLY unstable
- It is nearly untested and may break on edge cases
- Do NOT use it in production codebases at this time
- Feedback is appreciated :)