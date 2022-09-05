# Koncord

## Dependencies

* [serde]
* [csv]
* [rust_decimal]
  * Stable, acitively maintained, and uses #![forbid(unsafe_code)]
  * Why?
    * Money is not simple
    * Floating point is unsuitable for representing money
    * As fun as it would be to implement, need to keep solution "Simple" (and quick)
* [rust_fsm] only developed option, not updated for a year
