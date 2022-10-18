use std::convert::TryFrom;

use cosmwasm_std::{Decimal, Decimal256, StdError, StdResult, Uint128};

pub fn downcast(d: Decimal256) -> StdResult<Decimal> {
    let a = Uint128::try_from(d.atomics())?;
    let p = d.decimal_places();
    if let Ok(decimal) = Decimal::from_atomics(a, p) {
        Ok(decimal)
    } else {
        Err(StdError::generic_err("Decimal range exceeded"))
    }
}

pub fn upcast(d: Decimal) -> Decimal256 {
    let a = d.atomics();
    let p = d.decimal_places();
    // cannot fail
    Decimal256::from_atomics(a, p).unwrap()
}

/// return a * b
pub fn decimal_multiplication_in_256(a: Decimal, b: Decimal) -> StdResult<Decimal> {
    let a_u256: Decimal256 = upcast(a);
    let b_u256: Decimal256 = upcast(b);
    downcast(b_u256 * a_u256)
}

/// return a + b
pub fn decimal_summation_in_256(a: Decimal, b: Decimal) -> StdResult<Decimal> {
    let a_u256: Decimal256 = upcast(a);
    let b_u256: Decimal256 = upcast(b);
    downcast(b_u256 + a_u256)
}

/// return a - b
pub fn decimal_subtraction_in_256(a: Decimal, b: Decimal) -> StdResult<Decimal> {
    let a_u256: Decimal256 = upcast(a);
    let b_u256: Decimal256 = upcast(b);
    downcast(a_u256 - b_u256)
}

///
/// stolen from mirror
/// 
 
#[derive(PartialEq, Clone)]
pub enum Sign {
    Positive,
    Negative,
}

// return (sign, result)
pub fn signed_sum(
  sign_1: Sign,
  sign_2: Sign,
  num1: Decimal256,
  num2: Decimal256,
) -> StdResult<(Sign, Decimal256)> {
  if sign_1 == sign_2 {

    let val = num1 + num2;
    Ok((sign_1, val))

  } else if num1 > num2 {
    let val = num1 - num2;
    Ok((sign_1, val))
  } else {
    let val = num2 - num1;
    Ok((sign_2, val))
  }
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Decimal, Uint128};

    #[test]
    fn test_decimal_multiplication() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(1111111), Uint128::new(10000000));
        let multiplication =
            decimal_multiplication_in_256(Decimal::from_ratio(a, Uint128::new(1)), b).unwrap();
        assert_eq!(multiplication.to_string(), "11.11111");
    }

    #[test]
    fn test_decimal_sumation() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = decimal_summation_in_256(a, b).unwrap();
        assert_eq!(res.to_string(), "0.6");
    }

    #[test]
    fn test_decimal_subtraction() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = decimal_subtraction_in_256(a, b).unwrap();
        assert_eq!(res.to_string(), "0.2");
    }

    #[test]
    fn test_decimal_multiplication_in_256() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(1111111), Uint128::new(10000000));
        let multiplication =
            decimal_multiplication_in_256(Decimal::from_ratio(a, Uint128::new(1)), b).unwrap();
        assert_eq!(multiplication.to_string(), "11.11111");
    }

    #[test]
    fn test_decimal_sumation_in_256() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = decimal_summation_in_256(a, b).unwrap();
        assert_eq!(res.to_string(), "0.6");
    }

    #[test]
    fn test_decimal_subtraction_in_256() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = decimal_subtraction_in_256(a, b).unwrap();
        assert_eq!(res.to_string(), "0.2");
    }
}
